mod services;

use rusqlite::Connection;
use serde::Serialize;
use services::db::{self, GameRecord, SteamSyncMetadata};
use services::steam::scanner;
use services::steam::vanity;
use std::sync::Mutex;
use tauri::{Manager, State};

// appstate hold db conn
pub struct AppState {
    pub db_conn: Mutex<Connection>,
}

// steam settings struct
#[derive(Debug, Serialize)]
struct SteamSettings {
    api_key: Option<String>,
    steam_id64: Option<String>,
}

// struct for SteamID result
#[derive(Debug, Serialize)]
struct ResolveSteamIdResult {
    success: bool,
    steam_id64: Option<String>,
    error: Option<String>,
}

/// scans the user's Steam library for installed games and list of game records.
#[tauri::command]
fn scan_steam_games(state: State<AppState>) -> Result<Vec<GameRecord>, String> {
    let conn = state.db_conn.lock().map_err(|e| e.to_string())?;
    scanner::run_steam_scan(&conn)
}

/// gets a list of all installed games from the database.
#[tauri::command]
fn get_installed_games(state: State<AppState>) -> Result<Vec<GameRecord>, String> {
    let conn = state.db_conn.lock().map_err(|e| e.to_string())?;
    db::get_all_games(&conn).map_err(|e| e.to_string())
}

/// sets/saves api key and steam id to settings
#[tauri::command]
fn set_steam_settings(
    state: State<AppState>,
    api_key: String,
    steam_id64: String,
) -> Result<(), String> {
    // validate input before saving to db
    validate_steam_settings_input(&api_key, &steam_id64)?;

    let conn = state.db_conn.lock().map_err(|e| e.to_string())?;

    db::set_setting(&conn, "steam.api_key", &api_key).map_err(|e| e.to_string())?;
    db::set_setting(&conn, "steam.steam_id64", &steam_id64).map_err(|e| e.to_string())?;

    Ok(())
}

/// get steam setting values
#[tauri::command]
fn get_steam_settings(state: State<AppState>) -> Result<SteamSettings, String> {
    let conn = state.db_conn.lock().map_err(|e| e.to_string())?;

    let api_key = db::get_setting(&conn, "steam.api_key").map_err(|e| e.to_string())?;
    let steam_id64 = db::get_setting(&conn, "steam.steam_id64").map_err(|e| e.to_string())?;

    Ok(SteamSettings { api_key, steam_id64 })
}

/// function to validate input before saving steam settings
fn validate_steam_settings_input(api_key: &str, steam_id64: &str) -> Result<(), String> {
    let api_key_trimmed = api_key.trim();
    let steam_id_trimmed = steam_id64.trim();

    if api_key_trimmed.is_empty() {
        return Err("Steam API key is required.".to_string());
    }

    if api_key_trimmed.len() < 8 {
        return Err("Steam API key looks too short".to_string());
    }

    if steam_id_trimmed.is_empty() {
        return Err("SteamID64 is required".to_string());
    }

    if steam_id_trimmed.len() != 17 || !steam_id_trimmed.chars().all(|c| c.is_ascii_digit()) {
        return Err("SteamID64 must be exactly 17 numeric digits".to_string());
    }

    Ok(())
}

/// resolve Steam vanity name or URL to SteamID64
/// inputs can be:
/// - vanity name: "myusername"
/// - full url: "https://steamcommunity.com/id/myusername"
/// - numeric: "76561198123456789"
#[tauri::command]
async fn resolve_steam_id(
    input: String,
    state: State<'_, AppState>,
) -> Result<ResolveSteamIdResult, String> {
    // extract vanity name (or return None if already numeric)
    match vanity::extract_vanity_name(&input) {
        Some(vanity_name) => {
            // get key and drop lock
            let api_key = {
                let conn = state.db_conn.lock().map_err(|e| e.to_string())?;
                db::get_setting(&conn, "steam.api_key")
                    .map_err(|e| e.to_string())?
                    .ok_or("Steam API key not configured.")?
            }; 

            // call the Steam API
            match vanity::resolve_vanity_url(&vanity_name, &api_key).await {
                Ok(Some(steamid64)) => {
                    // Successfully resolved
                    Ok(ResolveSteamIdResult {
                        success: true,
                        steam_id64: Some(steamid64),
                        error: None,
                    })
                }
                Ok(None) => {
                    // vanity doesn't exist
                    Ok(ResolveSteamIdResult {
                        success: false,
                        steam_id64: None,
                        error: Some(
                            "Steam profile not found. Check the vanity URL and try again.".to_string()
                        ),
                    })
                }
                Err(api_error) => {
                    // call failed
                    Ok(ResolveSteamIdResult {
                        success: false,
                        steam_id64: None,
                        error: Some(format!("Failed to resolve: {}", api_error)),
                    })
                }
            }
        }
        None => {
            // input was treated as numeric SteamID64
            if vanity::validate_steamid64(&input) {
                Ok(ResolveSteamIdResult {
                    success: true,
                    steam_id64: Some(input.trim().to_string()),
                    error: None,
                })
            } else {
                Ok(ResolveSteamIdResult {
                    success: false,
                    steam_id64: None,
                    error: Some("Invalid SteamID64 format. Must be exactly 17 numeric digits.".to_string()),
                })
            }
        }
    }
}

/// validates stored steam creds by checking if non-empty
/// updates last_validated_at timestamp when sucessfull
/// returns metadata with validation status
#[tauri::command]
fn validate_steam_credentials(state: State<AppState>) -> Result<SteamSyncMetadata, String> {
    let conn = state.db_conn.lock().map_err(|e| e.to_string())?;

    // get current settings
    let api_key = db::get_setting(&conn, "steam.api_key")
        .map_err(|e| e.to_string())?;
    let steam_id64 = db::get_setting(&conn, "steam.steam_id64")
        .map_err(|e| e.to_string())?;

    // check if both present
    let api_key_valid = api_key.as_ref()
        .map(|k| !k.trim().is_empty()).unwrap_or(false);
    let steam_id_valid = steam_id64.as_ref()
        .map(|id| vanity::validate_steamid64(id)).unwrap_or(false);

    if !api_key_valid || !steam_id_valid {
        let error_msg = if !api_key_valid {
            "Steam API key is missing or invalid."
        } else {
            "SteamID64 is missing or invalid."
        };

        // record validation failure
        db::update_steam_sync_metadata(&conn, "failed", Some(error_msg))
            .map_err(|e| e.to_string())?;

        return Err(error_msg.to_string());
    }

    // both valid
    db::update_steam_validated_at(&conn).map_err(|e| e.to_string())?;

    // return metadata
    db::get_steam_sync_metadata(&conn).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // get os app dir
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");

            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
            let db_path = app_dir.join("app.db");

            // open sqlite conn & run schema migration
            let conn = Connection::open(&db_path).expect("failed to open database");
            db::init_db(&conn).expect("failed to init db schema");

            // manage db conn in tauri state
            app.manage(AppState {
                db_conn: Mutex::new(conn),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_steam_games,
            get_installed_games,
            set_steam_settings,
            get_steam_settings,
            resolve_steam_id,
            validate_steam_credentials
        ])
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
