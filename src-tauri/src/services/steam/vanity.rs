// helper service that resolves Steam vanity URLS or profile names to SteamID64.
// examples: "steamcommunity.com/id/username", "username", "76561198123456789"

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
}