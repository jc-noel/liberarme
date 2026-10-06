use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::header::LOCATION;
use reqwest::redirect::Policy;
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::json;

use crate::services::evidence::{
    get_evidence_for_game, record_evidence, EvidenceInput, EvidenceRecord,
};

const PCGW_REDIRECT_BASE: &str = "https://www.pcgamingwiki.com";
const PCGW_API_BASE: &str = "https://www.pcgamingwiki.com";
const PCGW_SOURCE_NAME: &str = "PCGamingWiki";
const PCGW_CACHE_TTL_SECS: u64 = 7 * 24 * 60 * 60;
const PCGW_USER_AGENT: &str =
    "Liberarme/0.1 (+https://github.com/jc-noel/liberarme; preservation evidence lookup)";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcgwSyncResult {
    pub evidence: Vec<EvidenceRecord>,
    pub from_cache: bool,
}

#[derive(Debug, Deserialize)]
struct ParseResponse {
    parse: Option<ParseData>,
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
struct ParseData {
    title: String,
    wikitext: Wikitext,
}

#[derive(Debug, Deserialize)]
struct Wikitext {
    #[serde(rename = "*")]
    text: String,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    code: String,
    info: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SteamAvailability {
    drm: Option<String>,
    notes: Option<String>,
}

pub async fn sync_pcgamingwiki_evidence(
    conn: &Connection,
    game_id: &str,
    steam_app_id: u32,
    force_refresh: bool,
) -> Result<PcgwSyncResult, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("System clock error: {e}"))?
        .as_secs();

    sync_pcgamingwiki_evidence_at(
        conn,
        game_id,
        steam_app_id,
        force_refresh,
        now,
        PCGW_REDIRECT_BASE,
        PCGW_API_BASE,
    )
    .await
}

async fn sync_pcgamingwiki_evidence_at(
    conn: &Connection,
    game_id: &str,
    steam_app_id: u32,
    force_refresh: bool,
    now: u64,
    redirect_base: &str,
    api_base: &str,
) -> Result<PcgwSyncResult, String> {
    let cached: Vec<EvidenceRecord> = get_evidence_for_game(conn, game_id)
        .map_err(|e| format!("Failed to read cached PCGamingWiki evidence: {e}"))?
        .into_iter()
        .filter(|row| row.source_name == PCGW_SOURCE_NAME)
        .collect();

    let newest_cached = cached.iter().map(|row| row.captured_at).max();

    if !force_refresh {
        if let Some(captured_at) = newest_cached {
            if now.saturating_sub(captured_at) < PCGW_CACHE_TTL_SECS {
                return Ok(PcgwSyncResult {
                    evidence: cached,
                    from_cache: true,
                });
            }
        }
    }

    let inputs = fetch_pcgamingwiki_evidence_at(
        game_id,
        steam_app_id,
        now,
        redirect_base,
        api_base,
    )
    .await?;

    let mut saved = Vec::with_capacity(inputs.len());
    for input in inputs {
        saved.push(
            record_evidence(conn, &input)
                .map_err(|e| format!("Failed to store PCGamingWiki evidence: {e}"))?,
        );
    }

    Ok(PcgwSyncResult {
        evidence: saved,
        from_cache: false,
    })
}

async fn fetch_pcgamingwiki_evidence_at(
    game_id: &str,
    steam_app_id: u32,
    captured_at: u64,
    redirect_base: &str,
    api_base: &str,
) -> Result<Vec<EvidenceInput>, String> {
    let client = reqwest::Client::builder()
        .user_agent(PCGW_USER_AGENT)
        .redirect(Policy::none())
        .build()
        .map_err(|e| format!("Failed to create PCGamingWiki HTTP client: {e}"))?;

    let mut redirect_url = reqwest::Url::parse(&format!(
        "{}/api/appid.php",
        redirect_base.trim_end_matches('/')
    ))
    .map_err(|e| format!("Invalid PCGamingWiki redirect URL: {e}"))?;
    redirect_url
        .query_pairs_mut()
        .append_pair("appid", &steam_app_id.to_string());

    let response = client
        .get(redirect_url)
        .send()
        .await
        .map_err(|e| format!("PCGamingWiki network error: {e}"))?;

    if response.status().as_u16() == 404 {
        return Ok(vec![lookup_evidence(
            game_id,
            steam_app_id,
            captured_at,
            None,
            &format!(
                "No PCGamingWiki page matched Steam AppID {steam_app_id} at lookup time."
            ),
        )]);
    }

    if response.status().as_u16() == 429 {
        return Err("PCGamingWiki rate limit reached. Try again later.".to_string());
    }

    if !response.status().is_redirection() {
        return Err(format!(
            "PCGamingWiki AppID lookup returned status {}",
            response.status()
        ));
    }

    let location = response
        .headers()
        .get(LOCATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| "PCGamingWiki AppID lookup did not return a redirect target".to_string())?;

    let page_title = page_title_from_location(location)?;
    let source_url = canonical_page_url(location, redirect_base);

    let mut parse_url = reqwest::Url::parse(&format!(
        "{}/w/api.php",
        api_base.trim_end_matches('/')
    ))
    .map_err(|e| format!("Invalid PCGamingWiki API URL: {e}"))?;
    {
        let mut pairs = parse_url.query_pairs_mut();
        pairs
            .append_pair("action", "parse")
            .append_pair("redirects", "1")
            .append_pair("prop", "wikitext")
            .append_pair("page", &page_title)
            .append_pair("format", "json");
    }

    let parse_response = client
        .get(parse_url)
        .send()
        .await
        .map_err(|e| format!("PCGamingWiki network error: {e}"))?;

    if parse_response.status().as_u16() == 429 {
        return Err("PCGamingWiki rate limit reached. Try again later.".to_string());
    }

    if !parse_response.status().is_success() {
        return Err(format!(
            "PCGamingWiki MediaWiki API returned status {}",
            parse_response.status()
        ));
    }

    let payload: ParseResponse = parse_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse PCGamingWiki API response: {e}"))?;

    if let Some(error) = payload.error {
        return Err(format!(
            "PCGamingWiki API error {}: {}",
            error.code, error.info
        ));
    }

    let parsed = payload
        .parse
        .ok_or_else(|| "PCGamingWiki API response did not include parsed page data".to_string())?;

    let page_url = if source_url.is_empty() {
        format!(
            "{}/wiki/{}",
            redirect_base.trim_end_matches('/'),
            parsed.title.replace(' ', "_")
        )
    } else {
        source_url
    };

    let mut evidence = vec![lookup_evidence(
        game_id,
        steam_app_id,
        captured_at,
        Some(page_url.clone()),
        &format!(
            "PCGamingWiki matched Steam AppID {steam_app_id} to '{}'.",
            parsed.title
        ),
    )];

    match extract_steam_availability(&parsed.wikitext.text) {
        Some(availability) => {
            if let Some(drm) = availability.drm.as_deref() {
                let trimmed = drm.trim();

                if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("unknown") {
                    let claim = if trimmed.eq_ignore_ascii_case("drm-free") {
                        "PCGamingWiki lists the Steam release as DRM-free.".to_string()
                    } else {
                        format!("PCGamingWiki lists Steam release DRM as: {trimmed}")
                    };

                    evidence.push(EvidenceInput {
                        game_id: game_id.to_string(),
                        source_type: "reference".to_string(),
                        source_name: PCGW_SOURCE_NAME.to_string(),
                        source_url: Some(page_url.clone()),
                        claim_type: "steam_drm".to_string(),
                        claim,
                        confidence: Some("medium".to_string()),
                        captured_at: Some(captured_at),
                        metadata_json: Some(
                            json!({
                                "steam_app_id": steam_app_id,
                                "page_title": parsed.title,
                                "raw_drm": trimmed,
                                "notes": availability.notes,
                            })
                            .to_string(),
                        ),
                    });

                    if let Some(offline_claim) = offline_requirement_claim(trimmed) {
                        evidence.push(EvidenceInput {
                            game_id: game_id.to_string(),
                            source_type: "reference".to_string(),
                            source_name: PCGW_SOURCE_NAME.to_string(),
                            source_url: Some(page_url.clone()),
                            claim_type: "internet_requirement".to_string(),
                            claim: offline_claim,
                            confidence: Some("medium".to_string()),
                            captured_at: Some(captured_at),
                            metadata_json: Some(
                                json!({
                                    "steam_app_id": steam_app_id,
                                    "page_title": parsed.title,
                                    "raw_drm": trimmed,
                                })
                                .to_string(),
                            ),
                        });
                    }
                } else {
                    evidence.push(incomplete_evidence(
                        game_id,
                        steam_app_id,
                        captured_at,
                        &page_url,
                        &parsed.title,
                    ));
                }
            } else {
                evidence.push(incomplete_evidence(
                    game_id,
                    steam_app_id,
                    captured_at,
                    &page_url,
                    &parsed.title,
                ));
            }
        }
        None => evidence.push(incomplete_evidence(
            game_id,
            steam_app_id,
            captured_at,
            &page_url,
            &parsed.title,
        )),
    }

    Ok(evidence)
}

fn lookup_evidence(
    game_id: &str,
    steam_app_id: u32,
    captured_at: u64,
    source_url: Option<String>,
    claim: &str,
) -> EvidenceInput {
    EvidenceInput {
        game_id: game_id.to_string(),
        source_type: "reference_lookup".to_string(),
        source_name: PCGW_SOURCE_NAME.to_string(),
        source_url,
        claim_type: "source_lookup".to_string(),
        claim: claim.to_string(),
        confidence: None,
        captured_at: Some(captured_at),
        metadata_json: Some(json!({ "steam_app_id": steam_app_id }).to_string()),
    }
}

fn incomplete_evidence(
    game_id: &str,
    steam_app_id: u32,
    captured_at: u64,
    source_url: &str,
    page_title: &str,
) -> EvidenceInput {
    EvidenceInput {
        game_id: game_id.to_string(),
        source_type: "reference".to_string(),
        source_name: PCGW_SOURCE_NAME.to_string(),
        source_url: Some(source_url.to_string()),
        claim_type: "source_incomplete".to_string(),
        claim: "PCGamingWiki page matched, but no usable Steam DRM/offline fact was found."
            .to_string(),
        confidence: None,
        captured_at: Some(captured_at),
        metadata_json: Some(
            json!({
                "steam_app_id": steam_app_id,
                "page_title": page_title,
            })
            .to_string(),
        ),
    }
}

fn offline_requirement_claim(raw_drm: &str) -> Option<String> {
    let normalized = raw_drm.to_ascii_lowercase().replace(' ', "");

    if normalized.contains("offline=no") || normalized.contains("offline=false") {
        return Some(
            "PCGamingWiki indicates this Steam release requires a constant internet connection."
                .to_string(),
        );
    }

    if normalized.contains("offline=postlaunch") {
        return Some(
            "PCGamingWiki indicates this Steam release requires an internet connection at launch."
                .to_string(),
        );
    }

    None
}

fn page_title_from_location(location: &str) -> Result<String, String> {
    let path = if let Ok(url) = reqwest::Url::parse(location) {
        url.path().to_string()
    } else {
        location.to_string()
    };

    let slug = path
        .split("/wiki/")
        .nth(1)
        .ok_or_else(|| format!("Unexpected PCGamingWiki redirect target: {location}"))?
        .split(|ch| ch == '?' || ch == '#')
        .next()
        .unwrap_or_default();

    let decoded = percent_decode(slug)?;
    if decoded.trim().is_empty() {
        return Err("PCGamingWiki redirect target did not contain a page title".to_string());
    }

    Ok(decoded.replace('_', " "))
}

fn canonical_page_url(location: &str, redirect_base: &str) -> String {
    if reqwest::Url::parse(location).is_ok() {
        location.to_string()
    } else {
        format!(
            "{}{}",
            redirect_base.trim_end_matches('/'),
            if location.starts_with('/') {
                location.to_string()
            } else {
                format!("/{location}")
            }
        )
    }
}

fn percent_decode(input: &str) -> Result<String, String> {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len() {
                return Err("Invalid percent-encoding in PCGamingWiki redirect".to_string());
            }

            let hi = hex_value(bytes[i + 1])
                .ok_or_else(|| "Invalid percent-encoding in PCGamingWiki redirect".to_string())?;
            let lo = hex_value(bytes[i + 2])
                .ok_or_else(|| "Invalid percent-encoding in PCGamingWiki redirect".to_string())?;
            output.push((hi << 4) | lo);
            i += 3;
        } else {
            output.push(bytes[i]);
            i += 1;
        }
    }

    String::from_utf8(output)
        .map_err(|_| "PCGamingWiki redirect contained invalid UTF-8".to_string())
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn extract_steam_availability(wikitext: &str) -> Option<SteamAvailability> {
    let lower = wikitext.to_ascii_lowercase();
    let needle = "{{availability/row";
    let mut offset = 0;

    while let Some(relative) = lower[offset..].find(needle) {
        let start = offset + relative;
        let template = extract_balanced_template(wikitext, start)?;
        let params = parse_template_params(template);

        if params
            .get(&1)
            .map(|value| value.trim().eq_ignore_ascii_case("steam"))
            .unwrap_or(false)
        {
            return Some(SteamAvailability {
                drm: params.get(&3).cloned().filter(|value| !value.trim().is_empty()),
                notes: params.get(&4).cloned().filter(|value| !value.trim().is_empty()),
            });
        }

        offset = start + template.len();
    }

    None
}

fn extract_balanced_template(text: &str, start: usize) -> Option<&str> {
    let bytes = text.as_bytes();
    let mut i = start;
    let mut depth = 0_i32;

    while i + 1 < bytes.len() {
        if bytes[i] == b'{' && bytes[i + 1] == b'{' {
            depth += 1;
            i += 2;
            continue;
        }

        if bytes[i] == b'}' && bytes[i + 1] == b'}' {
            depth -= 1;
            i += 2;
            if depth == 0 {
                return text.get(start..i);
            }
            continue;
        }

        i += 1;
    }

    None
}

fn parse_template_params(template: &str) -> HashMap<usize, String> {
    let inner = template
        .trim()
        .trim_start_matches("{{")
        .trim_end_matches("}}");

    let parts = split_top_level(inner, '|');
    let mut params = HashMap::new();
    let mut positional = 1_usize;

    for part in parts.into_iter().skip(1) {
        let trimmed = part.trim();

        if let Some((key, value)) = trimmed.split_once('=') {
            if let Ok(index) = key.trim().parse::<usize>() {
                params.insert(index, value.trim().to_string());
                continue;
            }
        }

        while params.contains_key(&positional) {
            positional += 1;
        }
        params.insert(positional, trimmed.to_string());
        positional += 1;
    }

    params
}

fn split_top_level(text: &str, delimiter: char) -> Vec<&str> {
    let bytes = text.as_bytes();
    let mut parts = Vec::new();
    let mut start = 0_usize;
    let mut depth = 0_i32;
    let delimiter = delimiter as u8;
    let mut i = 0_usize;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            depth += 1;
            i += 2;
            continue;
        }

        if i + 1 < bytes.len() && bytes[i] == b'}' && bytes[i + 1] == b'}' {
            depth -= 1;
            i += 2;
            continue;
        }

        if bytes[i] == delimiter && depth == 0 {
            if let Some(part) = text.get(start..i) {
                parts.push(part);
            }
            start = i + 1;
        }

        i += 1;
    }

    if let Some(part) = text.get(start..) {
        parts.push(part);
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::db::{init_db, upsert_game, GameRecord};

    fn insert_test_game(conn: &Connection) {
        upsert_game(
            conn,
            &GameRecord {
                id: "steam_400".to_string(),
                steam_app_id: 400,
                title: "Portal".to_string(),
                normalized_title: "portal".to_string(),
                is_owned: true,
                is_installed: true,
                install_path: Some("/games/Portal".to_string()),
                install_size: Some(123),
                last_updated: None,
                owned_synced_at: None,
                synced_at: 0,
            },
        )
        .unwrap();
    }

    #[test]
    fn parses_positional_and_nested_steam_availability_row() {
        let wiki = r#"
            {{Availability|
            {{Availability/row| GOG.com | 123 | DRM-free | | | Windows }}
            {{Availability/row| Steam | 400 | {{DRM|Steam|offline=postlaunch}} | Requires first launch online. | | Windows }}
            }}
        "#;

        let row = extract_steam_availability(wiki).unwrap();
        assert_eq!(
            row.drm.as_deref(),
            Some("{{DRM|Steam|offline=postlaunch}}")
        );
        assert_eq!(
            row.notes.as_deref(),
            Some("Requires first launch online.")
        );
    }

    #[test]
    fn parses_named_steam_availability_row() {
        let wiki = r#"
            {{Availability/row|1=Steam|2=400|3=DRM-free|4=Portable copy confirmed|5=|6=Windows}}
        "#;

        let row = extract_steam_availability(wiki).unwrap();
        assert_eq!(row.drm.as_deref(), Some("DRM-free"));
        assert_eq!(row.notes.as_deref(), Some("Portable copy confirmed"));
    }

    #[tokio::test]
    async fn known_game_records_relevant_pcgw_evidence() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        insert_test_game(&conn);

        let mut server = mockito::Server::new_async().await;

        let redirect = server
            .mock("GET", "/api/appid.php")
            .match_query(mockito::Matcher::UrlEncoded(
                "appid".into(),
                "400".into(),
            ))
            .with_status(302)
            .with_header("location", "/wiki/Portal")
            .create_async()
            .await;

        let parse = server
            .mock("GET", "/w/api.php")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("action".into(), "parse".into()),
                mockito::Matcher::UrlEncoded("redirects".into(), "1".into()),
                mockito::Matcher::UrlEncoded("prop".into(), "wikitext".into()),
                mockito::Matcher::UrlEncoded("page".into(), "Portal".into()),
                mockito::Matcher::UrlEncoded("format".into(), "json".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"parse":{"title":"Portal","wikitext":{"*":"{{Availability|\n{{Availability/row| Steam | 400 | DRM-free | | | Windows }}\n}}"}}}"#,
            )
            .create_async()
            .await;

        let result = sync_pcgamingwiki_evidence_at(
            &conn,
            "steam_400",
            400,
            true,
            1_800_000_000,
            &server.url(),
            &server.url(),
        )
        .await
        .unwrap();

        redirect.assert_async().await;
        parse.assert_async().await;

        assert!(!result.from_cache);
        assert_eq!(result.evidence.len(), 2);
        assert_eq!(result.evidence[0].claim_type, "source_lookup");
        assert_eq!(result.evidence[1].claim_type, "steam_drm");
        assert!(result.evidence[1].claim.contains("DRM-free"));
    }

    #[tokio::test]
    async fn incomplete_pcgw_page_records_uncertainty_not_a_drm_claim() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        insert_test_game(&conn);

        let mut server = mockito::Server::new_async().await;

        server
            .mock("GET", "/api/appid.php")
            .match_query(mockito::Matcher::UrlEncoded(
                "appid".into(),
                "400".into(),
            ))
            .with_status(302)
            .with_header("location", "/wiki/Portal")
            .create_async()
            .await;

        server
            .mock("GET", "/w/api.php")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("action".into(), "parse".into()),
                mockito::Matcher::UrlEncoded("redirects".into(), "1".into()),
                mockito::Matcher::UrlEncoded("prop".into(), "wikitext".into()),
                mockito::Matcher::UrlEncoded("page".into(), "Portal".into()),
                mockito::Matcher::UrlEncoded("format".into(), "json".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"parse":{"title":"Portal","wikitext":{"*":"{{Availability|{{Availability/row| Steam | 400 | | | | Windows }}}}"}}}"#,
            )
            .create_async()
            .await;

        let result = sync_pcgamingwiki_evidence_at(
            &conn,
            "steam_400",
            400,
            true,
            1_800_000_000,
            &server.url(),
            &server.url(),
        )
        .await
        .unwrap();

        assert_eq!(result.evidence.len(), 2);
        assert_eq!(result.evidence[1].claim_type, "source_incomplete");
        assert!(result
            .evidence
            .iter()
            .all(|row| row.claim_type != "steam_drm"));
    }

    #[tokio::test]
    async fn missing_pcgw_page_is_cached_as_lookup_evidence() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        insert_test_game(&conn);

        let mut server = mockito::Server::new_async().await;
        let lookup = server
            .mock("GET", "/api/appid.php")
            .match_query(mockito::Matcher::UrlEncoded(
                "appid".into(),
                "400".into(),
            ))
            .with_status(404)
            .create_async()
            .await;

        let result = sync_pcgamingwiki_evidence_at(
            &conn,
            "steam_400",
            400,
            false,
            1_800_000_000,
            &server.url(),
            &server.url(),
        )
        .await
        .unwrap();

        lookup.assert_async().await;
        assert_eq!(result.evidence.len(), 1);
        assert_eq!(result.evidence[0].claim_type, "source_lookup");
        assert!(result.evidence[0].claim.starts_with("No PCGamingWiki page"));

        let cached = sync_pcgamingwiki_evidence_at(
            &conn,
            "steam_400",
            400,
            false,
            1_800_000_100,
            "http://127.0.0.1:9",
            "http://127.0.0.1:9",
        )
        .await
        .unwrap();

        assert!(cached.from_cache);
        assert_eq!(cached.evidence.len(), 1);
    }

    #[tokio::test]
    async fn malformed_pcgw_response_returns_parse_error_without_writing_evidence() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        insert_test_game(&conn);

        let mut server = mockito::Server::new_async().await;

        server
            .mock("GET", "/api/appid.php")
            .match_query(mockito::Matcher::UrlEncoded(
                "appid".into(),
                "400".into(),
            ))
            .with_status(302)
            .with_header("location", "/wiki/Portal")
            .create_async()
            .await;

        server
            .mock("GET", "/w/api.php")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("this is not json")
            .create_async()
            .await;

        let error = sync_pcgamingwiki_evidence_at(
            &conn,
            "steam_400",
            400,
            true,
            1_800_000_000,
            &server.url(),
            &server.url(),
        )
        .await
        .unwrap_err();

        assert!(
            error.starts_with("Failed to parse PCGamingWiki API response"),
            "unexpected error: {error}"
        );

        let rows = get_evidence_for_game(&conn, "steam_400").unwrap();
        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn pcgw_network_failure_returns_error_without_writing_evidence() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        insert_test_game(&conn);

        let error = sync_pcgamingwiki_evidence_at(
            &conn,
            "steam_400",
            400,
            true,
            1_800_000_000,
            "http://127.0.0.1:9",
            "http://127.0.0.1:9",
        )
        .await
        .unwrap_err();

        assert!(error.starts_with("PCGamingWiki network error:"));

        let rows = get_evidence_for_game(&conn, "steam_400").unwrap();
        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn pcgw_rate_limit_returns_specific_error() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        insert_test_game(&conn);

        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/api/appid.php")
            .match_query(mockito::Matcher::UrlEncoded(
                "appid".into(),
                "400".into(),
            ))
            .with_status(429)
            .create_async()
            .await;

        let error = sync_pcgamingwiki_evidence_at(
            &conn,
            "steam_400",
            400,
            true,
            1_800_000_000,
            &server.url(),
            &server.url(),
        )
        .await
        .unwrap_err();

        assert_eq!(error, "PCGamingWiki rate limit reached. Try again later.");
    }
}
