use rusqlite::{params, Connection, Result};
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
    pub last_updated: u64
}

/// inits sqlite db schema
/// creates `games` table if does not exist.
pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS games (
            id TEXT PRIMARY KEY,
            steam_app_id INTEGER UNIQUE NOT NULL,
            title TEXT NOT NULL,
            normalized_title TEXT NOT NULL,
            install_path TEXT NOT NULL,
            install_size INTEGER NOT NULL,
            last_updated INTEGER NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )", 
        []
    )?;

    Ok(())
}

/// inserts new game or updates existing game based on `steam_app_id`
pub fn upsert_game(conn: &Connection, game: &GameRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO games (
            id, steam_app_id, title, normalized_title, install_path, install_size, last_updated, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP
        )
        ON CONFLICT(steam_app_id) DO UPDATE SET
            title = excluded.title,
            normalized_title = excluded.normalized_title,
            install_path = excluded.install_path,
            install_size = excluded.install_size,
            last_updated = excluded.last_updated,
            updated_at = CURRENT_TIMESTAMP", 
        params![
            game.id,
            game.steam_app_id,
            game.title,
            game.normalized_title,
            game.install_path,
            game.install_size as i64,
            game.last_updated as i64,
        ]
    )?;

    Ok(())
}

/// Fetch all stored games from db, ordered by title
pub fn get_all_games(conn: &Connection) -> Result<Vec<GameRecord>> {
    // get all sql statement
    let mut stmt = conn.prepare(
        "SELECT id, steam_app_id, title, normalized_title, install_path, install_size, last_updated
             FROM games
             ORDER BY title ASC",
    )?;

    let game_iter = stmt.query_map([], |row| {
        let install_size_i64: i64 = row.get(5)?;
        let last_updated_i64: i64 = row.get(6)?;

        Ok(GameRecord {
            id: row.get(0)?,
            steam_app_id: row.get(1)?,
            title: row.get(2)?,
            normalized_title: row.get(3)?,
            install_path: row.get(4)?,
            install_size: install_size_i64 as u64,
            last_updated: last_updated_i64 as u64
        })
    })?;

    let mut games = Vec::new();
    for game in game_iter {
        games.push(game?);
    }

    Ok(games)
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
            last_updated: 1625000000,
        };

        // insert Portal
        upsert_game(&conn, &portal).unwrap();

        let games = get_all_games(&conn).unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0], portal);

        // test upsert (update existing game without error)
        let portal_updated = GameRecord {
            id: "steam_400".to_string(),
            steam_app_id: 400,
            title: "Portal (Updated)".to_string(),
            normalized_title: "portal updated".to_string(),
            install_path: "/new/path/to/Portal".to_string(),
            install_size: 5000000000,
            last_updated: 1630000000,
        };

        upsert_game(&conn, &portal_updated).unwrap();

        let updated_games = get_all_games(&conn).unwrap();
        assert_eq!(updated_games.len(), 1); // Still 1 record
        assert_eq!(updated_games[0].title, "Portal (Updated)");
        assert_eq!(updated_games[0].install_size, 5000000000);
    }
}