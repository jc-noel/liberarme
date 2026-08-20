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
// PartialEq is added here so tests can compare two OwnedGame values
// (or Vecs of them) directly with assert_eq!, e.g. comparing a full
// games list returned from a mocked API call against what we expect.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    // delegate to the base-url-aware version, pointed at the real Steam API.
    // splitting it this way means tests can call fetch_owned_games_at()
    // with a fake local server url instead of hitting the real internet.
    fetch_owned_games_at("https://api.steampowered.com", steam_id64, api_key).await
}

/// same as fetch_owned_games, but lets the caller choose which server to
/// hit. production code should always use fetch_owned_games() above -
/// this version only exists so tests can point it at a mock server.
async fn fetch_owned_games_at(
    base_url: &str,
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
        "{}/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}&include_appinfo=1&include_played_free_games=1",
        base_url, trimmed_key, trimmed_id
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

    // --- fetch_owned_games_at tests below use mockito to fake the Steam API ---
    // mockito::Server::new_async() spins up a real local HTTP server that only
    // our test process can see. We tell it exactly what request to expect and
    // what response to send back, so we can test our error-handling code
    // without ever calling the real Steam API.

    #[tokio::test]
    async fn test_fetch_owned_games_empty_steam_id_returns_error() {
        // this hits our own input validation, so it never even makes a
        // network call - base_url can be nonsense here.
        let result = fetch_owned_games_at("http://example.invalid", "", "some_api_key").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "SteamID64 cannot be empty");
    }

    #[tokio::test]
    async fn test_fetch_owned_games_empty_api_key_returns_error() {
        let result = fetch_owned_games_at("http://example.invalid", "76561198123456789", "").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Steam API key cannot be empty");
    }

    #[tokio::test]
    async fn test_fetch_owned_games_success_returns_games_list() {
        let mut server = mockito::Server::new_async().await;

        // tell the fake server: when someone GETs this path, respond with
        // a successful Steam-shaped JSON body containing two games.
        let _mock = server
            .mock("GET", mockito::Matcher::Regex(r"^/IPlayerService/GetOwnedGames/v1/.*".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"response":{"games":[
                    {"appid":400,"name":"Portal","playtime_forever":1000},
                    {"appid":620,"name":"Portal 2","playtime_forever":500}
                ]}}"#,
            )
            .create_async()
            .await;

        let result = fetch_owned_games_at(&server.url(), "76561198123456789", "fake_key").await;

        let games = result.expect("expected Ok result with a games list");
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].appid, 400);
        assert_eq!(games[0].name, "Portal");
        assert_eq!(games[1].appid, 620);
    }

    #[tokio::test]
    async fn test_fetch_owned_games_private_profile_returns_empty_list() {
        let mut server = mockito::Server::new_async().await;

        // Steam returns a 200 with no "games" key at all when the profile
        // is private or the user owns nothing visible - this should NOT
        // be treated as an error, just an empty list.
        let _mock = server
            .mock("GET", mockito::Matcher::Regex(r"^/IPlayerService/GetOwnedGames/v1/.*".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"response":{}}"#)
            .create_async()
            .await;

        let result = fetch_owned_games_at(&server.url(), "76561198123456789", "fake_key").await;

        // now that OwnedGame derives PartialEq, we can compare the whole
        // Result<Vec<OwnedGame>, String> directly against an empty Ok Vec.
        assert_eq!(result, Ok(Vec::new()));
    }

    #[tokio::test]
    async fn test_fetch_owned_games_401_returns_invalid_key_message() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("GET", mockito::Matcher::Regex(r"^/IPlayerService/GetOwnedGames/v1/.*".to_string()))
            .with_status(401)
            .create_async()
            .await;

        let result = fetch_owned_games_at(&server.url(), "76561198123456789", "bad_key").await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Invalid API key. Check your Steam Web API key"
        );
    }

    #[tokio::test]
    async fn test_fetch_owned_games_403_returns_privacy_message() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("GET", mockito::Matcher::Regex(r"^/IPlayerService/GetOwnedGames/v1/.*".to_string()))
            .with_status(403)
            .create_async()
            .await;

        let result = fetch_owned_games_at(&server.url(), "76561198123456789", "fake_key").await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Access denied. Check your Steam privacy settings"
        );
    }

    #[tokio::test]
    async fn test_fetch_owned_games_other_error_status_returns_generic_message() {
        let mut server = mockito::Server::new_async().await;

        // 500 is just a stand-in for "some other error we don't have a
        // specific message for" - the exact code doesn't matter here.
        let _mock = server
            .mock("GET", mockito::Matcher::Regex(r"^/IPlayerService/GetOwnedGames/v1/.*".to_string()))
            .with_status(500)
            .create_async()
            .await;

        let result = fetch_owned_games_at(&server.url(), "76561198123456789", "fake_key").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Steam API returned status 500");
    }

    #[tokio::test]
    async fn test_fetch_owned_games_bad_json_returns_parse_error() {
        let mut server = mockito::Server::new_async().await;

        // valid HTTP 200, but the body isn't JSON Steam would ever send us
        let _mock = server
            .mock("GET", mockito::Matcher::Regex(r"^/IPlayerService/GetOwnedGames/v1/.*".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("this is not json")
            .create_async()
            .await;

        let result = fetch_owned_games_at(&server.url(), "76561198123456789", "fake_key").await;

        assert!(result.is_err());
        assert!(result.unwrap_err().starts_with("Failed to parse Steam API response"));
    }
}
