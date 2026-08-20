use crate::services::db::{self, GameRecord};
use crate::services::steam::{library, locate, manifest};
use rusqlite::Connection;
use std::fs;

/// orchestrates full steam scan:
/// 1. finds steam path
/// 2. parses libraryfolders.vdf
/// 3. scans library for appmanifest_*.acf files
/// 4. parses games & upserts into sqlite
/// 5. returns all discovered games
pub fn run_steam_scan(conn: &Connection) -> Result<Vec<GameRecord>, String> {
    // locate steam
    let steam_path = match locate::find_steam_path() {
        Some(path) => path,
        None => return Ok(Vec::new()), // steam wasn't found
    };

    // locate libraryfolders.vdf
    let library_vdf_path = steam_path.join("steamapps").join("libraryfolders.vdf");
    let mut library_paths = Vec::new();

    if library_vdf_path.exists() {
        let vdf_content = fs::read_to_string(&library_vdf_path)
            .map_err(|error| format!("Failed to read libraryfolders.vdf: {error}"))?;

        library_paths = library::parse_library_folders(&vdf_content)
            .map_err(|error| format!("Failed to parse libraryfolders.vdf: {error}"))?;
    }

    // include default steam install path as fallback
    if !library_paths.contains(&steam_path) {
        library_paths.push(steam_path.clone());
    }

    // scan each library for appmanifest_*.acf
    for lib_path in library_paths {
        let steamapps_dir = lib_path.join("steamapps");
        if !steamapps_dir.exists() || !steamapps_dir.is_dir() {
            continue;
        }

        // read all files in steamapps/
        if let Ok(entries) = fs::read_dir(&steamapps_dir) {
            for entry in entries.flatten() {
                let file_path = entry.path();

                // look for files named appmanifest_*.acf
                if file_path.is_file() {
                    if let Some(file_name) = file_path.file_name().and_then(|s| s.to_str()) {
                        if file_name.starts_with("appmanifest_") && file_name.ends_with(".acf") {
                            // parse manifest
                            let parsed_manifest = manifest::read_app_manifest(&file_path)
                                .map_err(|error| match error {
                                        manifest::AppManifestReadError::Io(source) => {
                                            format!("Failed to read {}: {source}", file_path.display())
                                        }
                                        manifest::AppManifestReadError::Parse(source) => {
                                            format!("Failed to parse {}: {source}", file_path.display())
                                        }
                                    },
                                )?;
                            if let Some(parsed_manifest) = parsed_manifest {
                                // build absolute install path
                                let full_install_path = steamapps_dir
                                    .join("common")
                                    .join(&parsed_manifest.install_dir);

                                let record = GameRecord {
                                    id: format!("steam_{}", parsed_manifest.app_id),
                                    steam_app_id: parsed_manifest.app_id,
                                    title: parsed_manifest.name.clone(),
                                    normalized_title: parsed_manifest.name.to_lowercase(),
                                    is_owned: false,       // this scan path only confirms local install, not Steam ownership
                                    is_installed: true,    // we found an appmanifest, so it's installed
                                    install_path: Some(full_install_path.to_string_lossy().to_string()),
                                    install_size: Some(parsed_manifest.install_size),
                                    last_updated: parsed_manifest.last_updated,
                                    owned_synced_at: None, // not touched by this sync path
                                    synced_at: 0,           // ignored on insert; db sets the real value via strftime
                                };

                                // save to sqlite
                                let _ = db::upsert_game(conn, &record);
                            }
                        }
                    }
                }
            }
        }
    }

    // return all games from db
    db::get_installed_games_only(conn).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_steam_scan_on_system_without_steam() {
        let conn = Connection::open_in_memory().unwrap();
        db::init_db(&conn).unwrap();

        let games = run_steam_scan(&conn).unwrap();
        println!("Scan result count: {}", games.len());
    }
}
