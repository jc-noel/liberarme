mod services;

use rusqlite::Connection;
use serde::Serialize;
use services::db::{self, GameRecord};
use services::steam::scanner;
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
            get_steam_settings
        ])
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
