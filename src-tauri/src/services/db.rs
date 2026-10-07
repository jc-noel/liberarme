use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};

/// game record struct to store in sqlite db
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameRecord {
    pub id: String,
    pub steam_app_id: u32,
    pub title: String,
    pub normalized_title: String,
    pub is_owned: bool,
    pub is_installed: bool,
    pub install_path: Option<String>,
    pub install_size: Option<u64>,
    pub last_updated: Option<u64>,
    pub owned_synced_at: Option<u64>,
    pub synced_at: u64,
}

/// steam sync metadata (timestamps & status info)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SteamSyncMetadata {
    pub last_validated_at: Option<u64>,   // when creds last validated
    pub last_sync_at: Option<u64>,        // last successful owned games sync
    pub last_sync_status: Option<String>, // success or failed
    pub last_sync_error: Option<String>,  // error mss if last sync failed
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GameAssessmentRecord {
    pub game_id: String,
    pub status: String,
    pub launcher_requirement: Option<String>,
    pub internet_requirement: Option<String>,
    pub confidence: String,
    pub basis: Option<String>,
    pub updated_at: u64,
}

const CURRENT_SCHEMA_VERSION: i64 = 4;

/// inits sqlite db schema
/// creates `games` table if does not exist.
pub fn init_db(conn: &Connection) -> Result<()> {
    // app settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    let stored_version: i64 = get_setting(conn, "schema_version")?
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    
    if stored_version < 2 {
        // Legacy pre-v2 migration: the old games schema did not contain the
        // ownership/install-state columns introduced in schema version 2.
        //
        // Keep this destructive migration scoped to pre-v2 databases only.
        // Later schema bumps (including v3's evidence table) must not erase
        // an existing user's library.
        conn.execute("DROP TABLE IF EXISTS games", [])?;
    }
    // games table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS games (
            id TEXT PRIMARY KEY,
            steam_app_id INTEGER UNIQUE NOT NULL,
            title TEXT NOT NULL,
            normalized_title TEXT NOT NULL,
            is_owned INTEGER NOT NULL DEFAULT 0,
            is_installed INTEGER NOT NULL DEFAULT 0,
            install_path TEXT,
            install_size INTEGER,
            last_updated INTEGER,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            owned_synced_at INTEGER,
            synced_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        )",
        [],
    )?;

    // Evidence is append-only source material used by later assessment and
    // verification services. It intentionally lives beside the games table
    // rather than adding source-specific columns to GameRecord.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS evidence (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id TEXT NOT NULL,
            source_type TEXT NOT NULL,
            source_name TEXT NOT NULL,
            source_url TEXT,
            claim_type TEXT NOT NULL,
            claim TEXT NOT NULL,
            confidence TEXT,
            captured_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            metadata_json TEXT,
            FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_evidence_game_id
         ON evidence(game_id)",
        [],
    )?;


    // One current preservation assessment per game. Evidence and verification
    // history stay in their own append-only tables; this row is only the
    // current, explainable summary derived from those records.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS game_assessments (
            game_id TEXT PRIMARY KEY,
            status TEXT NOT NULL DEFAULT 'unknown',
            launcher_requirement TEXT,
            internet_requirement TEXT,
            confidence TEXT NOT NULL DEFAULT 'none',
            basis TEXT,
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
        )",
        [],
    )?;

    if stored_version < CURRENT_SCHEMA_VERSION {
        set_setting(conn, "schema_version", &CURRENT_SCHEMA_VERSION.to_string())?;
    }

    Ok(())
}

/// inserts new game or updates existing game based on `steam_app_id`
pub fn upsert_game(conn: &Connection, game: &GameRecord) -> Result<()> {
    conn.execute(
        "INSERT INTO games (
            id, steam_app_id, title, normalized_title, is_owned, is_installed,
            install_path, install_size, last_updated, owned_synced_at, synced_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, strftime('%s','now')
        )
        ON CONFLICT(steam_app_id) DO UPDATE SET
            title = excluded.title,
            normalized_title = excluded.normalized_title,
            is_owned = CASE WHEN games.is_owned = 1 THEN 1 ELSE excluded.is_owned END,
            is_installed = excluded.is_installed,
            install_path = excluded.install_path,
            install_size = excluded.install_size,
            last_updated = excluded.last_updated,
            owned_synced_at = games.owned_synced_at,
            synced_at = strftime('%s','now')",
        params![
            game.id,
            game.steam_app_id,
            game.title,
            game.normalized_title,
            game.is_owned,
            game.is_installed,
            game.install_path,
            game.install_size.map(|v| v as i64),
            game.last_updated.map(|v| v as i64),
            game.owned_synced_at.map(|v| v as i64),
        ],
    )?;

    Ok(())
}

/// Fetch all stored games from db, ordered by title
pub fn get_all_games(conn: &Connection) -> Result<Vec<GameRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, steam_app_id, title, normalized_title, is_owned, is_installed,
                install_path, install_size, last_updated, owned_synced_at, synced_at
             FROM games
             ORDER BY title ASC",
    )?;

    let game_iter = stmt.query_map([], |row| {
        let is_owned_i64: i64 = row.get(4)?;
        let is_installed_i64: i64 = row.get(5)?;
        let install_size_i64: Option<i64> = row.get(7)?;
        let last_updated_i64: Option<i64> = row.get(8)?;
        let owned_synced_at_i64: Option<i64> = row.get(9)?;
        let synced_at_i64: i64 = row.get(10)?;

        Ok(GameRecord {
            id: row.get(0)?,
            steam_app_id: row.get(1)?,
            title: row.get(2)?,
            normalized_title: row.get(3)?,
            is_owned: is_owned_i64 != 0,
            is_installed: is_installed_i64 != 0,
            install_path: row.get(6)?,
            install_size: install_size_i64.map(|v| v as u64),
            last_updated: last_updated_i64.map(|v| v as u64),
            owned_synced_at: owned_synced_at_i64.map(|v| v as u64),
            synced_at: synced_at_i64 as u64,
        })
    })?;

    let mut games = Vec::new();
    for game in game_iter {
        games.push(game?);
    }

    Ok(games)
}

/// all games that are marked as installed, ordered by title
pub fn get_installed_games_only(conn: &Connection) -> Result<Vec<GameRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, steam_app_id, title, normalized_title, is_owned, is_installed,
                install_path, install_size, last_updated, owned_synced_at, synced_at
             FROM games
             WHERE is_installed = 1
             ORDER BY title ASC",
    )?;

    let game_iter = stmt.query_map([], |row| {
        let is_owned_i64: i64 = row.get(4)?;
        let is_installed_i64: i64 = row.get(5)?;
        let install_size_i64: Option<i64> = row.get(7)?;
        let last_updated_i64: Option<i64> = row.get(8)?;
        let owned_synced_at_i64: Option<i64> = row.get(9)?;
        let synced_at_i64: i64 = row.get(10)?;

        Ok(GameRecord {
            id: row.get(0)?,
            steam_app_id: row.get(1)?,
            title: row.get(2)?,
            normalized_title: row.get(3)?,
            is_owned: is_owned_i64 != 0,
            is_installed: is_installed_i64 != 0,
            install_path: row.get(6)?,
            install_size: install_size_i64.map(|v| v as u64),
            last_updated: last_updated_i64.map(|v| v as u64),
            owned_synced_at: owned_synced_at_i64.map(|v| v as u64),
            synced_at: synced_at_i64 as u64,
        })
    })?;

    let mut games = Vec::new();
    for game in game_iter {
        games.push(game?);
    }

    Ok(games)
}

/// Insert or replace the current preservation assessment for a game.
///
/// The game_id primary key guarantees there is only one current assessment per
/// game. Historical evidence and verification records are intentionally kept
/// elsewhere.
pub fn upsert_game_assessment(
    conn: &Connection,
    assessment: &GameAssessmentRecord,
) -> Result<()> {
    conn.execute(
        "INSERT INTO game_assessments (
            game_id, status, launcher_requirement, internet_requirement,
            confidence, basis, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(game_id) DO UPDATE SET
            status = excluded.status,
            launcher_requirement = excluded.launcher_requirement,
            internet_requirement = excluded.internet_requirement,
            confidence = excluded.confidence,
            basis = excluded.basis,
            updated_at = excluded.updated_at",
        params![
            assessment.game_id,
            assessment.status,
            assessment.launcher_requirement,
            assessment.internet_requirement,
            assessment.confidence,
            assessment.basis,
            assessment.updated_at as i64,
        ],
    )?;

    Ok(())
}

pub fn get_game_assessment(
    conn: &Connection,
    game_id: &str,
) -> Result<Option<GameAssessmentRecord>> {
    conn.query_row(
        "SELECT game_id, status, launcher_requirement, internet_requirement,
                confidence, basis, updated_at
         FROM game_assessments
         WHERE game_id = ?1",
        params![game_id],
        |row| {
            let updated_at: i64 = row.get(6)?;
            Ok(GameAssessmentRecord {
                game_id: row.get(0)?,
                status: row.get(1)?,
                launcher_requirement: row.get(2)?,
                internet_requirement: row.get(3)?,
                confidence: row.get(4)?,
                basis: row.get(5)?,
                updated_at: updated_at as u64,
            })
        },
    )
    .optional()
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

/// gets steam sync metadata from settings
pub fn get_steam_sync_metadata(conn: &Connection) -> Result<SteamSyncMetadata> {
    let last_validated_at =
        get_setting(conn, "steam.last_validated_at")?.and_then(|s| s.parse::<u64>().ok());

    let last_sync_at = get_setting(conn, "steam.last_sync_at")?.and_then(|s| s.parse::<u64>().ok());

    let last_sync_status = get_setting(conn, "steam.last_sync_status")?;

    let last_sync_error = get_setting(conn, "steam.last_sync_error")?;

    Ok(SteamSyncMetadata {
        last_validated_at,
        last_sync_at,
        last_sync_status,
        last_sync_error,
    })
}

/// updates steam sync metadata after validation or sync attempt
pub fn update_steam_sync_metadata(
    conn: &Connection,
    status: &str,        // success or failed
    error: Option<&str>, // error msg if any
) -> Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // update last_sync_at only on success
    if status == "success" {
        set_setting(conn, "steam.last_sync_at", &now.to_string())?;
        set_setting(conn, "steam.last_sync_status", "success")?;
        set_setting(conn, "steam.last_sync_error", "")?; // clear error on success
    } else {
        // on failure, keep last_sync_at and keep update status/error
        set_setting(conn, "steam.last_sync_status", "failed")?;
        if let Some(err_msg) = error {
            set_setting(conn, "steam.last_sync_error", err_msg)?;
        }
    }

    Ok(())
}

/// updates timestamp when creds were last validated
pub fn update_steam_validated_at(conn: &Connection) -> Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    set_setting(conn, "steam.last_validated_at", &now.to_string())?;

    Ok(())
}

/// upserts a batch of owned games (from steam api) by steam_app_id.
/// - never overwrites install-related fields and set by a local scan
/// - marks is_owned = 1 and stamps owned_synced_at
pub fn upsert_owned_games(conn: &Connection, games: &[(u32, String)]) -> Result<()> {
    // games: vec of steam_app_id, name tuples
    for (app_id, name) in games {
        let normalized_title = name.to_lowercase();
        let id = format!("steam_{}", app_id);

        conn.execute(
            "INSERT INTO games (
                id, steam_app_id, title, normalized_title, is_owned, is_installed,
                install_path, install_size, last_updated, owned_synced_at, synced_at
            ) VALUES (
                ?1, ?2, ?3, ?4, 1, 0, NULL, NULL, NULL, strftime('%s','now'), strftime('%s','now')
            )
            ON CONFLICT(steam_app_id) DO UPDATE SET
                title = excluded.title,
                normalized_title = excluded.normalized_title,
                is_owned = 1,
                owned_synced_at = strftime('%s','now')",
            params![id, app_id, name, normalized_title],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evidence_table_stores_multiple_and_conflicting_entries() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        let game = GameRecord {
            id: "steam_400".to_string(),
            steam_app_id: 400,
            title: "Portal".to_string(),
            normalized_title: "portal".to_string(),
            is_owned: true,
            is_installed: true,
            install_path: Some("/path/to/Portal".to_string()),
            install_size: Some(1000),
            last_updated: Some(1625000000),
            owned_synced_at: Some(1625000000),
            synced_at: 0,
        };
        upsert_game(&conn, &game).unwrap();

        conn.execute(
            "INSERT INTO evidence (
                game_id, source_type, source_name, source_url,
                claim_type, claim, confidence, captured_at, metadata_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                "steam_400",
                "reference",
                "PCGamingWiki",
                Some("https://www.pcgamingwiki.com/wiki/Portal"),
                "launcher_requirement",
                "Steam is not required after installation",
                Some("medium"),
                1_700_000_000_i64,
                Some(r#"{"field":"DRM"}"#),
            ],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO evidence (
                game_id, source_type, source_name, source_url,
                claim_type, claim, confidence, captured_at, metadata_json
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                "steam_400",
                "local_verification",
                "User verification",
                Option::<String>::None,
                "launcher_requirement",
                "Launch failed while Steam was closed",
                Some("high"),
                1_700_000_100_i64,
                Option::<String>::None,
            ],
        )
        .unwrap();

        let mut stmt = conn
            .prepare(
                "SELECT source_type, source_name, source_url, claim_type, claim,
                        confidence, captured_at, metadata_json
                 FROM evidence
                 WHERE game_id = ?1
                 ORDER BY captured_at ASC",
            )
            .unwrap();

        let rows = stmt
            .query_map(params!["steam_400"], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, i64>(6)?,
                    row.get::<_, Option<String>>(7)?,
                ))
            })
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();

        assert_eq!(rows.len(), 2, "conflicting evidence should be retained, not overwritten");
        assert_eq!(rows[0].0, "reference");
        assert_eq!(rows[0].1, "PCGamingWiki");
        assert_eq!(
            rows[0].2.as_deref(),
            Some("https://www.pcgamingwiki.com/wiki/Portal")
        );
        assert_eq!(rows[0].3, "launcher_requirement");
        assert_eq!(rows[0].5.as_deref(), Some("medium"));
        assert_eq!(rows[0].7.as_deref(), Some(r#"{"field":"DRM"}"#));

        assert_eq!(rows[1].0, "local_verification");
        assert_eq!(rows[1].1, "User verification");
        assert_eq!(rows[1].2, None);
        assert_eq!(rows[1].3, "launcher_requirement");
        assert_eq!(rows[1].5.as_deref(), Some("high"));
        assert_eq!(rows[1].7, None);

        assert_ne!(rows[0].4, rows[1].4, "opposing claims should coexist");
    }

    #[test]
    fn test_schema_v3_upgrade_preserves_existing_games_and_adds_evidence() {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute(
            "CREATE TABLE app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE games (
                id TEXT PRIMARY KEY,
                steam_app_id INTEGER UNIQUE NOT NULL,
                title TEXT NOT NULL,
                normalized_title TEXT NOT NULL,
                is_owned INTEGER NOT NULL DEFAULT 0,
                is_installed INTEGER NOT NULL DEFAULT 0,
                install_path TEXT,
                install_size INTEGER,
                last_updated INTEGER,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
                owned_synced_at INTEGER,
                synced_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
            )",
            [],
        )
        .unwrap();

        set_setting(&conn, "schema_version", "2").unwrap();
        conn.execute(
            "INSERT INTO games (
                id, steam_app_id, title, normalized_title, is_owned, is_installed
             ) VALUES ('steam_400', 400, 'Portal', 'portal', 1, 1)",
            [],
        )
        .unwrap();

        init_db(&conn).unwrap();

        let games = get_all_games(&conn).unwrap();
        assert_eq!(games.len(), 1, "v2 -> v3 must preserve the existing library");
        assert_eq!(games[0].steam_app_id, 400);
        assert_eq!(
            get_setting(&conn, "schema_version").unwrap(),
            Some(CURRENT_SCHEMA_VERSION.to_string())
        );

        conn.execute(
            "INSERT INTO evidence (
                game_id, source_type, source_name, claim_type, claim
             ) VALUES ('steam_400', 'manual', 'Test', 'launcher_requirement', 'Unknown')",
            [],
        )
        .unwrap();

        let evidence_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM evidence WHERE game_id = 'steam_400'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(evidence_count, 1);
    }

    #[test]
    fn test_ownership_sync_survives_a_subsequent_local_rescan() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        // Step 1: local scan finds Portal installed (not yet known to be owned)
        let scanned = GameRecord {
            id: "steam_400".to_string(),
            steam_app_id: 400,
            title: "Portal".to_string(),
            normalized_title: "portal".to_string(),
            is_owned: false,
            is_installed: true,
            install_path: Some("/path/to/Portal".to_string()),
            install_size: Some(4294967296),
            last_updated: Some(1625000000),
            owned_synced_at: None,
            synced_at: 0,
        };
        upsert_game(&conn, &scanned).unwrap();

        // Step 2: ownership sync confirms Portal is owned
        let owned = vec![(400u32, "Portal".to_string())];
        upsert_owned_games(&conn, &owned).unwrap();

        let after_sync = get_all_games(&conn).unwrap();
        assert_eq!(after_sync.len(), 1);
        assert!(after_sync[0].is_owned, "ownership sync should have set is_owned");
        assert!(after_sync[0].owned_synced_at.is_some(), "ownership sync should stamp owned_synced_at");

        // Step 3: user re-runs local scan. The scanner has no concept of
        // Steam ownership, so it always builds a fresh GameRecord with
        // is_owned: false and owned_synced_at: None (see scanner.rs). This
        // rescan must NOT erase the ownership data set in step 2.
        let rescanned = GameRecord {
            id: "steam_400".to_string(),
            steam_app_id: 400,
            title: "Portal".to_string(),
            normalized_title: "portal".to_string(),
            is_owned: false,       // scanner never knows about ownership
            is_installed: true,
            install_path: Some("/path/to/Portal".to_string()),
            install_size: Some(4294967296),
            last_updated: Some(1625000500), // pretend Steam updated the game
            owned_synced_at: None, // scanner never sets this
            synced_at: 0,
        };
        upsert_game(&conn, &rescanned).unwrap();

        let after_rescan = get_all_games(&conn).unwrap();
        assert_eq!(after_rescan.len(), 1, "rescan should not create a duplicate row");

        let game = &after_rescan[0];
        assert!(
            game.is_owned,
            "a local rescan must not erase ownership data set by a prior Steam sync"
        );
        assert!(
            game.owned_synced_at.is_some(),
            "a local rescan must not erase the owned_synced_at timestamp set by a prior Steam sync"
        );

        // Sanity check: the rescan's own install-related updates should
        // still go through normally.
        assert_eq!(game.last_updated, Some(1625000500));
        assert!(game.is_installed);
    }

    #[test]
    fn test_get_installed_games_only_filters_out_owned_only() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        // insert an installed game
        let installed = GameRecord {
            id: "steam_1".to_string(),
            steam_app_id: 1,
            title: "Installed Game".to_string(),
            normalized_title: "installed game".to_string(),
            is_owned: false,
            is_installed: true,
            install_path: Some("/path/to/game".to_string()),
            install_size: Some(1000),
            last_updated: Some(1625000000),
            owned_synced_at: None,
            synced_at: 0,
        };
        upsert_game(&conn, &installed).unwrap();

        // insert an owned-only game (from API sync)
        let owned_only = vec![(2u32, "Owned Only Game".to_string())];
        upsert_owned_games(&conn, &owned_only).unwrap();

        // get_installed_games_only should return only the installed one
        let installed_games = get_installed_games_only(&conn).unwrap();
        assert_eq!(installed_games.len(), 1);
        assert_eq!(installed_games[0].steam_app_id, 1);
        assert!(installed_games[0].is_installed);

        // get_all_games should return both
        let all_games = get_all_games(&conn).unwrap();
        assert_eq!(all_games.len(), 2);
    }

    #[test]
    fn test_local_scan_survives_when_ownership_sync_would_fail() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        // simulate a local scan: insert 5 games as installed
        let scanned_games = vec![
            GameRecord {
                id: "steam_1".to_string(),
                steam_app_id: 1,
                title: "Game A".to_string(),
                normalized_title: "game a".to_string(),
                is_owned: false,
                is_installed: true,
                install_path: Some("/path/to/A".to_string()),
                install_size: Some(1000),
                last_updated: Some(1625000000),
                owned_synced_at: None,
                synced_at: 0,
            },
            GameRecord {
                id: "steam_2".to_string(),
                steam_app_id: 2,
                title: "Game B".to_string(),
                normalized_title: "game b".to_string(),
                is_owned: false,
                is_installed: true,
                install_path: Some("/path/to/B".to_string()),
                install_size: Some(2000),
                last_updated: Some(1625000001),
                owned_synced_at: None,
                synced_at: 0,
            },
            GameRecord {
                id: "steam_3".to_string(),
                steam_app_id: 3,
                title: "Game C".to_string(),
                normalized_title: "game c".to_string(),
                is_owned: false,
                is_installed: true,
                install_path: Some("/path/to/C".to_string()),
                install_size: Some(3000),
                last_updated: Some(1625000002),
                owned_synced_at: None,
                synced_at: 0,
            },
            GameRecord {
                id: "steam_4".to_string(),
                steam_app_id: 4,
                title: "Game D".to_string(),
                normalized_title: "game d".to_string(),
                is_owned: false,
                is_installed: true,
                install_path: Some("/path/to/D".to_string()),
                install_size: Some(4000),
                last_updated: Some(1625000003),
                owned_synced_at: None,
                synced_at: 0,
            },
            GameRecord {
                id: "steam_5".to_string(),
                steam_app_id: 5,
                title: "Game E".to_string(),
                normalized_title: "game e".to_string(),
                is_owned: false,
                is_installed: true,
                install_path: Some("/path/to/E".to_string()),
                install_size: Some(5000),
                last_updated: Some(1625000004),
                owned_synced_at: None,
                synced_at: 0,
            },
        ];

        for game in &scanned_games {
            upsert_game(&conn, game).unwrap();
        }

        let games_after_scan = get_all_games(&conn).unwrap();
        assert_eq!(games_after_scan.len(), 5);
        for game in &games_after_scan {
            assert!(game.is_installed, "all scanned games should be marked installed");
            assert!(!game.is_owned, "scanned games should not be marked owned yet");
        }

        // simulate ownership sync that partially succeeds
        // let's say the API returns 3 games, or returns garbage, or connection drops mid-syncing
        // in this case ownership sync would add 2 owned games that weren't installed
        let partial_owned = vec![
            (100u32, "Game X (not installed)".to_string()),
            (101u32, "Game Y (not installed)".to_string()),
        ];
        upsert_owned_games(&conn, &partial_owned).unwrap();

        // all 5 original scanned games should still exist
        // with is_installed = true and their install paths are the same
        let games_after_ownership = get_all_games(&conn).unwrap();
        assert_eq!(games_after_ownership.len(), 7, "should have 5 original + 2 new owned");

        // make sure the 5 original are untouched
        for original in &scanned_games {
            let found = games_after_ownership
                .iter()
                .find(|g| g.steam_app_id == original.steam_app_id);
            assert!(found.is_some(), "game {} should still exist", original.steam_app_id);

            let game = found.unwrap();
            assert!(
                game.is_installed,
                "game {} should still be marked installed",
                original.steam_app_id
            );
            assert_eq!(
                game.install_path, original.install_path,
                "game {} install path should be untouched",
                original.steam_app_id
            );
            assert_eq!(
                game.install_size, original.install_size,
                "game {} install size should be untouched",
                original.steam_app_id
            );
        }

        // 2 new owned-only games exist
        let owned_only = games_after_ownership
            .iter()
            .filter(|g| g.steam_app_id >= 100)
            .collect::<Vec<_>>();
        assert_eq!(owned_only.len(), 2, "should have 2 owned-only games");
        for game in owned_only {
            assert!(game.is_owned, "owned-only game should be marked owned");
            assert!(!game.is_installed, "owned-only game should not be marked installed");
            assert_eq!(game.install_path, None, "owned-only game should have no install path");
        }
    }

    #[test]
    fn test_init_db_migrates_when_schema_version_outdated() {
        let conn = Connection::open_in_memory().unwrap();

        // simulate a pre-alpha DB created before is_owned/is_installed existed,
        // with no schema_version tracked at all (old-old DB)
        conn.execute(
            "CREATE TABLE app_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE games (
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
            [],
        )
        .unwrap();

        // running init_db against this old schema should migrate cleanly
        init_db(&conn).unwrap();

        // schema_version should now be stamped
        let version = get_setting(&conn, "schema_version").unwrap();
        assert_eq!(version, Some(CURRENT_SCHEMA_VERSION.to_string()));

        // confirm new columns actually exist by inserting through them
        let game = GameRecord {
            id: "steam_1".to_string(),
            steam_app_id: 1,
            title: "Test".to_string(),
            normalized_title: "test".to_string(),
            is_owned: true,
            is_installed: false,
            install_path: None,
            install_size: None,
            last_updated: None,
            owned_synced_at: None,
            synced_at: 0,
        };
        upsert_game(&conn, &game).unwrap();
    }

    #[test]
    fn test_init_db_does_not_rebuild_games_when_schema_current() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap(); // fresh db, already current version

        // insert a game
        let game = GameRecord {
            id: "steam_1".to_string(),
            steam_app_id: 1,
            title: "Test".to_string(),
            normalized_title: "test".to_string(),
            is_owned: true,
            is_installed: false,
            install_path: None,
            install_size: None,
            last_updated: None,
            owned_synced_at: None,
            synced_at: 0,
        };
        upsert_game(&conn, &game).unwrap();

        // re-running init_db should NOT wipe existing data
        init_db(&conn).unwrap();

        let games = get_all_games(&conn).unwrap();
        assert_eq!(games.len(), 1, "data should survive when schema is already current");
    }

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
            is_owned: false,
            is_installed: true,
            install_path: Some("/path/to/Portal".to_string()),
            install_size: Some(4294967296),
            last_updated: Some(1625000000),
            owned_synced_at: None,
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
        assert_eq!(games[0].is_installed, portal.is_installed);
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
            is_owned: false,
            is_installed: true,
            install_path: Some("/new/path/to/Portal".to_string()),
            install_size: Some(5000000000),
            last_updated: Some(1630000000),
            owned_synced_at: None,
            synced_at: 0, // ignored on insert/update; db always sets the real value via strftime
        };

        upsert_game(&conn, &portal_updated).unwrap();

        let updated_games = get_all_games(&conn).unwrap();
        assert_eq!(updated_games.len(), 1); // Still 1 record
        assert_eq!(updated_games[0].title, "Portal (Updated)");
        assert_eq!(updated_games[0].install_size, Some(5000000000));
    }

    #[test]
    fn test_game_assessment_insert_read_and_update_keeps_one_current_row() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        let game = GameRecord {
            id: "steam_400".to_string(),
            steam_app_id: 400,
            title: "Portal".to_string(),
            normalized_title: "portal".to_string(),
            is_owned: true,
            is_installed: true,
            install_path: Some("/path/to/Portal".to_string()),
            install_size: Some(1000),
            last_updated: Some(1_625_000_000),
            owned_synced_at: Some(1_625_000_000),
            synced_at: 0,
        };
        upsert_game(&conn, &game).unwrap();

        let initial = GameAssessmentRecord {
            game_id: game.id.clone(),
            status: "unknown".to_string(),
            launcher_requirement: None,
            internet_requirement: None,
            confidence: "none".to_string(),
            basis: None,
            updated_at: 1_800_000_000,
        };
        upsert_game_assessment(&conn, &initial).unwrap();

        assert_eq!(
            get_game_assessment(&conn, &game.id).unwrap(),
            Some(initial.clone())
        );

        let updated = GameAssessmentRecord {
            game_id: game.id.clone(),
            status: "needs_verification".to_string(),
            launcher_requirement: Some("unknown".to_string()),
            internet_requirement: Some("online_at_launch".to_string()),
            confidence: "medium".to_string(),
            basis: Some("External evidence requires local confirmation.".to_string()),
            updated_at: 1_800_000_100,
        };
        upsert_game_assessment(&conn, &updated).unwrap();

        assert_eq!(
            get_game_assessment(&conn, &game.id).unwrap(),
            Some(updated)
        );

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM game_assessments WHERE game_id = ?1",
                params![game.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "only one current assessment should exist per game");
    }

    #[test]
    fn test_get_game_assessment_missing_returns_none() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        assert_eq!(
            get_game_assessment(&conn, "steam_missing").unwrap(),
            None
        );
    }

    #[test]
    fn test_schema_v4_upgrade_preserves_games_and_evidence_and_adds_assessments() {
        let conn = Connection::open_in_memory().unwrap();

        // Build a schema-v3 database with an existing game and evidence row.
        init_db(&conn).unwrap();
        set_setting(&conn, "schema_version", "3").unwrap();

        let game = GameRecord {
            id: "steam_400".to_string(),
            steam_app_id: 400,
            title: "Portal".to_string(),
            normalized_title: "portal".to_string(),
            is_owned: true,
            is_installed: true,
            install_path: Some("/path/to/Portal".to_string()),
            install_size: Some(1000),
            last_updated: Some(1_625_000_000),
            owned_synced_at: Some(1_625_000_000),
            synced_at: 0,
        };
        upsert_game(&conn, &game).unwrap();
        conn.execute(
            "INSERT INTO evidence (
                game_id, source_type, source_name, claim_type, claim, captured_at
             ) VALUES (?1, 'reference', 'Fixture', 'steam_drm', 'DRM-free', ?2)",
            params![game.id, 1_700_000_000_i64],
        )
        .unwrap();

        conn.execute("DROP TABLE game_assessments", []).unwrap();

        init_db(&conn).unwrap();

        assert_eq!(
            get_setting(&conn, "schema_version").unwrap(),
            Some(CURRENT_SCHEMA_VERSION.to_string())
        );
        assert_eq!(get_all_games(&conn).unwrap().len(), 1);

        let evidence_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM evidence", [], |row| row.get(0))
            .unwrap();
        assert_eq!(evidence_count, 1);

        let assessment_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM game_assessments", [], |row| row.get(0))
            .unwrap();
        assert_eq!(assessment_count, 0);
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

    #[test]
    fn test_get_steam_sync_metadata_empty() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        let metadata = get_steam_sync_metadata(&conn).unwrap();
        assert_eq!(metadata.last_validated_at, None);
        assert_eq!(metadata.last_sync_at, None);
        assert_eq!(metadata.last_sync_status, None);
        assert_eq!(metadata.last_sync_error, None);
    }

    #[test]
    fn test_update_steam_sync_metadata_success() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        update_steam_sync_metadata(&conn, "success", None).unwrap();

        let metadata = get_steam_sync_metadata(&conn).unwrap();
        assert_eq!(metadata.last_sync_status, Some("success".to_string()));
        assert!(metadata.last_sync_at.is_some());
        assert_eq!(metadata.last_sync_error, Some("".to_string())); // Empty on success
    }

    #[test]
    fn test_update_steam_sync_metadata_failed() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        let error_msg = "Invalid API key";
        update_steam_sync_metadata(&conn, "failed", Some(error_msg)).unwrap();

        let metadata = get_steam_sync_metadata(&conn).unwrap();
        assert_eq!(metadata.last_sync_status, Some("failed".to_string()));
        assert_eq!(metadata.last_sync_error, Some(error_msg.to_string()));
        assert_eq!(metadata.last_sync_at, None); // Not updated on failure
    }

    #[test]
    fn test_update_steam_validated_at() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        update_steam_validated_at(&conn).unwrap();

        let metadata = get_steam_sync_metadata(&conn).unwrap();
        assert!(metadata.last_validated_at.is_some());
        assert!(metadata.last_validated_at.unwrap() > 0);
    }

    #[test]
    fn test_steam_sync_metadata_preserves_previous_sync_on_failure() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        // First: successful sync
        update_steam_sync_metadata(&conn, "success", None).unwrap();
        let first_sync = get_steam_sync_metadata(&conn).unwrap();
        let first_sync_time = first_sync.last_sync_at;
        assert!(first_sync_time.is_some());

        // Second: failed sync (should keep first sync time)
        update_steam_sync_metadata(&conn, "failed", Some("Network error")).unwrap();
        let metadata = get_steam_sync_metadata(&conn).unwrap();

        assert_eq!(metadata.last_sync_status, Some("failed".to_string()));
        assert_eq!(metadata.last_sync_at, first_sync_time); // Preserved!
        assert_eq!(metadata.last_sync_error, Some("Network error".to_string()));
    }

    #[test]
    fn test_upsert_owned_games_new_owned_only_game() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        let owned = vec![(400u32, "Portal".to_string())];
        upsert_owned_games(&conn, &owned).unwrap();

        let games = get_all_games(&conn).unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].steam_app_id, 400);
        assert_eq!(games[0].title, "Portal");
        assert!(games[0].is_owned);
        assert!(!games[0].is_installed);
        assert_eq!(games[0].install_path, None);
        assert_eq!(games[0].install_size, None);
        assert!(games[0].owned_synced_at.is_some());
    }

    #[test]
    fn test_upsert_owned_games_preserves_existing_install_data() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        // simulate a prior local scan finding Portal installed
        let scanned = GameRecord {
            id: "steam_400".to_string(),
            steam_app_id: 400,
            title: "Portal".to_string(),
            normalized_title: "portal".to_string(),
            is_owned: false,
            is_installed: true,
            install_path: Some("/path/to/Portal".to_string()),
            install_size: Some(4294967296),
            last_updated: Some(1625000000),
            owned_synced_at: None,
            synced_at: 0,
        };
        upsert_game(&conn, &scanned).unwrap();

        // now sync ownership for the same steam_app_id
        let owned = vec![(400u32, "Portal".to_string())];
        upsert_owned_games(&conn, &owned).unwrap();

        let games = get_all_games(&conn).unwrap();
        assert_eq!(games.len(), 1); // reconciled into same row, not duplicated
        assert!(games[0].is_owned); // ownership flag now set
        assert!(games[0].is_installed); // install flag untouched
        assert_eq!(games[0].install_path, scanned.install_path); // untouched
        assert_eq!(games[0].install_size, scanned.install_size); // untouched
        assert!(games[0].owned_synced_at.is_some());
    }

    #[test]
    fn test_upsert_owned_games_resync_keeps_timestamp() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        let owned = vec![(400u32, "Portal".to_string())];
        upsert_owned_games(&conn, &owned).unwrap();
        upsert_owned_games(&conn, &owned).unwrap(); // sync again

        let games = get_all_games(&conn).unwrap();
        assert_eq!(games.len(), 1); // still one row, no duplicate
        assert!(games[0].owned_synced_at.is_some());
    }
}
