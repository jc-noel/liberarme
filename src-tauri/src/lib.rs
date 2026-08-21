mod services;

use rusqlite::Connection;
use serde::Serialize;
use services::db::{self, GameRecord, SteamSyncMetadata};
use services::steam::scanner;
use services::steam::vanity;
use services::steam::owned_games;
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
    db::get_installed_games_only(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_games(state: State<AppState>) -> Result<Vec<GameRecord>, String> {
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

    if !steam_id_trimmed.is_empty() {
        if steam_id_trimmed.len() != 17 || !steam_id_trimmed.chars().all(|c| c.is_ascii_digit()) {
            return Err("SteamID64 must be exactly 17 numeric digits".to_string());
        }
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

/// syncs owned games from steam using stored creds
/// reads api key and steamid64 from sqlite settings
/// returns list of owned games or error message
#[tauri::command]
async fn sync_owned_games(state: State<'_, AppState>) -> Result<Vec<owned_games::OwnedGame>, String> {
    // get stored creds, then drop lock before awaiting
    let (api_key, steam_id64) = {
        let conn = state.db_conn.lock().map_err(|e| e.to_string())?;

        let api_key = db::get_setting(&conn, "steam.api_key")
            .map_err(|e| e.to_string())?
            .ok_or("Steam API key not configured. Please configure settings first.")?;

        let steam_id64 = db::get_setting(&conn, "steam.steam_id64")
            .map_err(|e| e.to_string())?
            .ok_or("SteamID64 not configured. Please configure settings first.")?;
        (api_key, steam_id64)
    };

    // validate creds
    if api_key.trim().is_empty() {
        return Err("Steam API key is empty".to_string());
    }

    if !vanity::validate_steamid64(&steam_id64) {
        return Err("SteamID64 is invalid format".to_string());
    }

    // call steam api
    match owned_games::fetch_owned_games(&steam_id64, &api_key).await {
        Ok(games) => {
            // map OwnedGame -> (steam_app_id, name) shape db.rs expects,
            // keeping db.rs decoupled from the Steam API response struct
            let games_for_db: Vec<(u32, String)> = games
                .iter()
                .map(|g| (g.appid, g.name.clone()))
                .collect();

            let conn = state.db_conn.lock().map_err(|e| e.to_string())?;

            // persist ownership before reporting success, so a sync that
            // "succeeds" always means the data actually made it to sqlite
            db::upsert_owned_games(&conn, &games_for_db).map_err(|e| e.to_string())?;

            db::update_steam_sync_metadata(&conn, "success", None)
                .map_err(|e| e.to_string())?;

            Ok(games)
        }
        Err(api_error) => {
            let conn = state.db_conn.lock().map_err(|e| e.to_string())?;
            db::update_steam_sync_metadata(&conn, "failed", Some(&api_error))
                .map_err(|e| e.to_string())?;

            Err(api_error)
        }
    }
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
            get_all_games,
            set_steam_settings,
            get_steam_settings,
            resolve_steam_id,
            validate_steam_credentials,
            sync_owned_games
        ])
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

    // Proves that a failure in one Steam workflow (ownership sync) cannot
    // take down the local scan workflow, at the shared-state level.
    //
    // AppState holds a single Mutex<Connection> used by every command.
    // A Mutex only becomes "poisoned" if a thread panics while holding
    // the lock - a normal Err(...) return does NOT poison it. This test
    // simulates the worst case (a panic while the lock is held, as if a
    // future bug crashed mid-sync) and confirms that a subsequent
    // db-locking command (standing in for scan_steam_games) still
    // returns a clean, recoverable Err instead of taking down the app.
    #[test]
    fn local_scan_path_survives_a_poisoned_db_mutex() {
        let conn = Connection::open_in_memory().unwrap();
        db::init_db(&conn).unwrap();

        let state = AppState {
            db_conn: Mutex::new(conn),
        };

        // Simulate a hypothetical bug: something panics while holding
        // the db lock during an ownership sync. We deliberately swallow
        // the panic's default stderr output so the test log stays clean.
        let previous_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));

        let sync_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            let _conn_guard = state.db_conn.lock().unwrap();
            panic!("simulated failure mid-sync while holding the db lock");
        }));

        panic::set_hook(previous_hook);
        assert!(sync_result.is_err(), "expected the simulated sync panic to unwind");

        // The mutex is now poisoned. Confirm that code shaped like our
        // real commands (state.db_conn.lock().map_err(|e| e.to_string()))
        // degrades to a clean, recoverable error instead of panicking
        // the whole app - this is exactly how scan_steam_games,
        // get_installed_games, etc. all acquire the lock today.
        let scan_like_result: Result<(), String> = (|| {
            let conn = state.db_conn.lock().map_err(|e| e.to_string())?;
            db::get_installed_games_only(&conn).map_err(|e| e.to_string())?;
            Ok(())
        })();

        assert!(
            scan_like_result.is_err(),
            "a poisoned mutex should surface as a normal Err, not a second panic"
        );
        assert!(
            scan_like_result.unwrap_err().contains("poisoned"),
            "error message should clearly indicate the lock was poisoned"
        );
    }
}