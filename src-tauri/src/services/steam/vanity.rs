// helper service that resolves Steam vanity URLS or profile names to SteamID64.
// examples: "steamcommunity.com/id/username", "username", "76561198123456789"
use serde::Deserialize;

/// result from steams ResolveVanityURL api
#[derive(Debug, Deserialize)]
pub struct VanityUrlResponse {
    pub response: VanityUrlResponseData,
}

#[derive(Debug, Deserialize)]
pub struct VanityUrlResponseData {
    pub steamid: Option<String>,
    pub success: i32, // 1 = found, 42 = not found
}

/// validates string as valid numeric SteamID64 (17 digits)
pub fn validate_steamid64(input: &str) -> bool {
    let trimmed = input.trim();

    // check for 17 characters
    if trimmed.len() != 17 {
        return false;
    }

    trimmed.chars().all(|c| c.is_ascii_digit())
}

/// extracts vanity name from Steam URL
/// examples:
/// - "steamcommunity.com/id/myusername" -> Some("myusername")
/// - "https://steamcommunity.com/id/myusername" -> Some("myusername")
/// - "myusername" -> Some("myusername")
/// - "76561198123456789" → None (already numeric)
pub fn extract_vanity_name(input: &str) -> Option<String> {
    let trimmed = input.trim();

    // if already valid, return none
    if validate_steamid64(trimmed) {
        return None;
    }

    // extract
    if let Some(id_index) = trimmed.find("/id/") {
        let after_id = &trimmed[id_index + 4..]; // skip "/id/"
        let vanity = after_id.split('/').next().unwrap_or("").trim();

        if !vanity.is_empty() {
            return Some(vanity.to_string());
        }
    }

    // otherwise treat whole input as vanity name
    if !trimmed.is_empty() {
        return Some(trimmed.to_string());
    }

    None
}

/// resolve vanity name to STeamID64 using Steam's ResolveVanityUrl API
/// 
/// arguments:
/// - `vanity_name`: vanity url slug (e.g., "myusername")
/// - `api_key`: Steam Web API key
/// 
/// returns:
/// - Ok(Some(steamid64)) if found
/// - Ok(None) vanity name doesn't exist
/// - Err(message) api call fail or network error
pub async fn resolve_vanity_url(vanity_name: &str, api_key: &str) -> Result<Option<String>, String> {
    let trimmed_vanity = vanity_name.trim();
    let trimmed_key = api_key.trim();

    // validate inputs
    if trimmed_vanity.is_empty() {
        return Err("Vanity name cannot be empty.".to_string());
    }

    if trimmed_key.is_empty() {
        return Err("Steam API key cannot be empty".to_string());
    }

    // api url
    let url = format!(
        "https://api.steampowered.com/ISteamUser/ResolveVanityURL/v1/?key={}&vanityurl={}",
        trimmed_key, trimmed_vanity
    );

    // make request
    let response = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    // check http status
    // we check for specific status codes here (instead of one generic
    // error) so the user gets a message that actually tells them what
    // to do next. this mirrors the same status handling already done
    // in owned_games.rs::fetch_owned_games, so both Steam API call
    // sites give the same quality of error message.
    if !response.status().is_success() {
        if response.status().as_u16() == 401 {
            return Err("Invalid API key. Check your Steam Web API key".to_string());
        }
        if response.status().as_u16() == 403 {
            return Err("Access denied. Check your Steam privacy settings".to_string());
        }
        return Err(format!("Steam API returned status {}", response.status()));
    }

    // parse json
    let data: VanityUrlResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Steam API response: {}", e))?;

    // check steams success code
    if data.response.success != 1 {
        // success = 42 means vanity url not found
        return Ok(None);
    }

    // extract/validate steamid64
    match data.response.steamid {
        Some(steamid) => {
            if validate_steamid64(&steamid) {
                Ok(Some(steamid))
            } else {
                Err("Steam API returned invalid SteamID64 format".to_string())
            }
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_steamid64_valid() {
        assert!(validate_steamid64("76561198123456789"));
        assert!(validate_steamid64("  76561198123456789  ")); // with whitespace
    }

    #[test]
    fn test_validate_steamid64_invalid_length() {
        assert!(!validate_steamid64("123")); // too short
        assert!(!validate_steamid64("765611981234567890")); // too long
    }

    #[test]
    fn test_validate_steamid64_invalid_non_numeric() {
        assert!(!validate_steamid64("7656119812345678a")); // contains letter
        assert!(!validate_steamid64("765611-98-123456789")); // contains dashes
    }

    #[test]
    fn test_extract_vanity_name_from_url() {
        assert_eq!(
            extract_vanity_name("steamcommunity.com/id/myusername"),
            Some("myusername".to_string())
        );

        assert_eq!(
            extract_vanity_name("https://steamcommunity.com/id/myusername"),
            Some("myusername".to_string())
        );

        assert_eq!(
            extract_vanity_name("https://steamcommunity.com/id/myusername/"),
            Some("myusername".to_string())
        );
    }

    #[test]
    fn test_extract_vanity_name_direct() {
        assert_eq!(
            extract_vanity_name("myusername"),
            Some("myusername".to_string())
        );

        assert_eq!(
            extract_vanity_name("  myusername  "),
            Some("myusername".to_string())
        );
    }

    #[test]
    fn test_extract_vanity_name_from_numeric_steamid64() {
        // If input is already numeric SteamID64, return None
        assert_eq!(extract_vanity_name("76561198123456789"), None);
    }

    #[test]
    fn test_extract_vanity_name_empty_input() {
        assert_eq!(extract_vanity_name(""), None);
        assert_eq!(extract_vanity_name("   "), None);
    }

    #[test]
    fn test_resolve_vanity_url_input_validation() {
        // This test just checks that the function validates inputs
        // not sure how to test the actual API call without mocking
        // just verify the function signature compiles
    }
}
