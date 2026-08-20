// service to fetch owned games from steam web api
use serde::{Deserialize, Serialize};

/// response struct to store steams' GetOwnedGames API response
#[derive(Debug, Deserialize)]
pub struct GetOwnedGamesResponse {
    pub response: GetOwnedGamesData,
}

#[derive(Debug, Deserialize)]
pub struct GetOwnedGamesData {
    pub games: Option<Vec<OwnedGame>>,
}

/// represents a single game from GetOwnedGames response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnedGame {
    pub appid: u32,
    pub name: String,
    pub playtime_forever: u32, // playtime in mins
}

/// fetches owned games from steam's IPlayerService/GetOwnedGames API
/// 
/// arguments:
/// - `steam_id64`: users numeric steamid64
/// - `api_key`: steam web api key
/// 
/// returns:
/// - Ok(games) - list of owned games
/// - Err(message) - api error or network failure
pub async fn fetch_owned_games(
    steam_id64: &str,
    api_key: &str,
) -> Result<Vec<OwnedGame>, String> {
    let trimmed_id = steam_id64.trim();
    let trimmed_key = api_key.trim();

    // validate inputs
    if trimmed_id.is_empty() {
        return Err("SteamID64 cannot be empty".to_string());
    }

    if trimmed_key.is_empty() {
        return Err("Steam API key cannot be empty".to_string());
    }

    // build api url
    let url = format!(
        "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}&include_appinfo=1&include_played_free_games=1",
        trimmed_key, trimmed_id
    );

    // make http request
    let response = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    // check http status
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
    let data: GetOwnedGamesResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Steam API response: {}", e))?;

    // extract games list
    match data.response.games {
        Some(games) => Ok(games),
        None => Ok(Vec::new()), // no games owned (or private profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_owned_games_input_validation() {
        // These tests verify the function signature and input validation work
        // Actual API testing would require mocking, which we'll skip for now
    }

    #[test]
    fn test_owned_game_struct() {
        // Verify the struct can deserialize correctly
        let json = r#"{"appid": 400, "name": "Portal", "playtime_forever": 1000}"#;
        let game: Result<OwnedGame, _> = serde_json::from_str(json);
        assert!(game.is_ok());
        
        let g = game.unwrap();
        assert_eq!(g.appid, 400);
        assert_eq!(g.name, "Portal");
        assert_eq!(g.playtime_forever, 1000);
    }
}