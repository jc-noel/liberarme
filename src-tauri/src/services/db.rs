use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};

/// game record struct to store in sqlite db
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameRecord {
    pub id: String,
    pub steam_app_id: u32,
    pub title: String,
    pub normalized_title: String,
    pub install_path: String,
    pub install_size: u64,
    pub last_updated: Option<u64>,
    pub synced_at: u64
}

/// inits sqlite db schema
/// creates `games` table if does not exist.
pub fn init_db(conn: &Connection) -> Result<()> {

    // games table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS games (
            id TEXT PRIMARY KEY,
            steam_app_id INTEGER UNIQUE NOT NULL,
            title TEXT NOT NULL,
            normalized_title TEXT NOT NULL,
            install_path TEXT NOT NULL,
            install_size INTEGER NOT NULL,
            last_updated INTEGER,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            synced_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        )", 
        []
    )?;

    // app settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    Ok(())
}

/// inserts new game or updates existing game based on `steam_app_id`
pub fn upsert_game(conn: &Connection, game: &GameRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO games (
            id, steam_app_id, title, normalized_title, install_path, install_size, last_updated, synced_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, strftime('%s','now')
        )
        ON CONFLICT(steam_app_id) DO UPDATE SET
            title = excluded.title,
            normalized_title = excluded.normalized_title,
            install_path = excluded.install_path,
            install_size = excluded.install_size,
            last_updated = excluded.last_updated,
            synced_at = strftime('%s','now')", 
        params![
            game.id,
            game.steam_app_id,
            game.title,
            game.normalized_title,
            game.install_path,
            game.install_size as i64,
            game.last_updated.map(|v| v as i64),
        ]
    )?;

    Ok(())
}

/// Fetch all stored games from db, ordered by title
pub fn get_all_games(conn: &Connection) -> Result<Vec<GameRecord>> {
    // get all sql statement
    let mut stmt = conn.prepare(
        "SELECT id, steam_app_id, title, normalized_title, install_path, install_size, last_updated, synced_at
             FROM games
             ORDER BY title ASC",
    )?;

    let game_iter = stmt.query_map([], |row| {
        let install_size_i64: i64 = row.get(5)?;
        let last_updated_i64: Option<i64> = row.get(6)?;
        let synced_at_i64: i64 = row.get(7)?;

        Ok(GameRecord {
            id: row.get(0)?,
            steam_app_id: row.get(1)?,
            title: row.get(2)?,
            normalized_title: row.get(3)?,
            install_path: row.get(4)?,
            install_size: install_size_i64 as u64,
            last_updated: last_updated_i64.map(|v| v as u64),
            synced_at: synced_at_i64 as u64
        })
    })?;

    let mut games = Vec::new();
    for game in game_iter {
        games.push(game?);
    }

    Ok(games)
}

/// inserts or updates a setting value by key
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value, updated_at)
         VALUES (?1, ?2, CURRENT_TIMESTAMP)
         ON CONFLICT(key) DO UPDATE SET
             value = excluded.value,
             updated_at = CURRENT_TIMESTAMP",
        params![key, value],
    )?;

    Ok(())
}

/// fetches a setting by key
pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_db_and_upsert_game() {
        // Create an in-memory database for testing
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        let portal = GameRecord {
            id: "steam_400".to_string(),
            steam_app_id: 400,
            title: "Portal".to_string(),
            normalized_title: "portal".to_string(),
            install_path: "/path/to/Portal".to_string(),
            install_size: 4294967296,
            last_updated: Some(1625000000),
            synced_at: 0, // ignored on insert/update; db always sets the real value via strftime
        };

        // insert Portal
        upsert_game(&conn, &portal).unwrap();

        let games = get_all_games(&conn).unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, portal.id);
        assert_eq!(games[0].steam_app_id, portal.steam_app_id);
        assert_eq!(games[0].title, portal.title);
        assert_eq!(games[0].normalized_title, portal.normalized_title);
        assert_eq!(games[0].install_path, portal.install_path);
        assert_eq!(games[0].install_size, portal.install_size);
        assert_eq!(games[0].last_updated, portal.last_updated);
        assert!(games[0].synced_at > 0);

        // test upsert (update existing game without error)
        let portal_updated = GameRecord {
            id: "steam_400".to_string(),
            steam_app_id: 400,
            title: "Portal (Updated)".to_string(),
            normalized_title: "portal updated".to_string(),
            install_path: "/new/path/to/Portal".to_string(),
            install_size: 5000000000,
            last_updated: Some(1630000000),
            synced_at: 0, // ignored on insert/update; db always sets the real value via strftime
        };

        upsert_game(&conn, &portal_updated).unwrap();

        let updated_games = get_all_games(&conn).unwrap();
        assert_eq!(updated_games.len(), 1); // Still 1 record
        assert_eq!(updated_games[0].title, "Portal (Updated)");
        assert_eq!(updated_games[0].install_size, 5000000000);
    }

    #[test]
    fn test_set_and_get_setting() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        set_setting(&conn, "steam.api_key", "test_key_123").unwrap();

        let value = get_setting(&conn, "steam.api_key").unwrap();
        assert_eq!(value, Some("test_key_123".to_string()));
    }

    #[test]
    fn test_set_setting_overwrites_existing_key() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        set_setting(&conn, "steam.steam_id64", "11111111111111111").unwrap();
        set_setting(&conn, "steam.steam_id64", "76561198000000000").unwrap();

        let value = get_setting(&conn, "steam.steam_id64").unwrap();
        assert_eq!(value, Some("76561198000000000".to_string()));
    }

    #[test]
    fn test_get_setting_missing_key_returns_none() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        let value = get_setting(&conn, "steam.missing").unwrap();
        assert_eq!(value, None);
    }
}