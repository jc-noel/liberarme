use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceInput {
    pub game_id: String,
    pub source_type: String,
    pub source_name: String,
    pub source_url: Option<String>,
    pub claim_type: String,
    pub claim: String,
    pub confidence: Option<String>,
    pub captured_at: Option<u64>,
    pub metadata_json: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceRecord {
    pub id: i64,
    pub game_id: String,
    pub source_type: String,
    pub source_name: String,
    pub source_url: Option<String>,
    pub claim_type: String,
    pub claim: String,
    pub confidence: Option<String>,
    pub captured_at: u64,
    pub metadata_json: Option<String>,
}

/// Append one evidence item for a game.
///
/// Evidence is intentionally append-only at this layer: new observations should
/// be recorded as new rows so conflicting and historical claims remain visible.
pub fn record_evidence(conn: &Connection, evidence: &EvidenceInput) -> Result<EvidenceRecord> {
    let captured_at = evidence.captured_at.map(|value| value as i64);

    conn.execute(
        "INSERT INTO evidence (
            game_id, source_type, source_name, source_url,
            claim_type, claim, confidence, captured_at, metadata_json
         ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7,
            COALESCE(?8, strftime('%s','now')),
            ?9
         )",
        params![
            evidence.game_id,
            evidence.source_type,
            evidence.source_name,
            evidence.source_url,
            evidence.claim_type,
            evidence.claim,
            evidence.confidence,
            captured_at,
            evidence.metadata_json,
        ],
    )?;

    let id = conn.last_insert_rowid();
    get_evidence_by_id(conn, id)
}

pub fn get_evidence_for_game(conn: &Connection, game_id: &str) -> Result<Vec<EvidenceRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, game_id, source_type, source_name, source_url,
                claim_type, claim, confidence, captured_at, metadata_json
         FROM evidence
         WHERE game_id = ?1
         ORDER BY captured_at ASC, id ASC",
    )?;

    let rows = stmt.query_map(params![game_id], map_evidence_row)?;

    rows.collect()
}

fn get_evidence_by_id(conn: &Connection, id: i64) -> Result<EvidenceRecord> {
    conn.query_row(
        "SELECT id, game_id, source_type, source_name, source_url,
                claim_type, claim, confidence, captured_at, metadata_json
         FROM evidence
         WHERE id = ?1",
        params![id],
        map_evidence_row,
    )
}

fn map_evidence_row(row: &rusqlite::Row<'_>) -> Result<EvidenceRecord> {
    let captured_at: i64 = row.get(8)?;

    Ok(EvidenceRecord {
        id: row.get(0)?,
        game_id: row.get(1)?,
        source_type: row.get(2)?,
        source_name: row.get(3)?,
        source_url: row.get(4)?,
        claim_type: row.get(5)?,
        claim: row.get(6)?,
        confidence: row.get(7)?,
        captured_at: captured_at as u64,
        metadata_json: row.get(9)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::db::{init_db, upsert_game, GameRecord};

    fn insert_test_game(conn: &Connection) {
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

        upsert_game(conn, &game).unwrap();
    }

    #[test]
    fn record_and_fetch_external_evidence() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        insert_test_game(&conn);

        let input = EvidenceInput {
            game_id: "steam_400".to_string(),
            source_type: "reference".to_string(),
            source_name: "PCGamingWiki".to_string(),
            source_url: Some("https://www.pcgamingwiki.com/wiki/Portal".to_string()),
            claim_type: "launcher_requirement".to_string(),
            claim: "Steam is not required after installation".to_string(),
            confidence: Some("medium".to_string()),
            captured_at: Some(1_700_000_000),
            metadata_json: Some(r#"{"field":"DRM"}"#.to_string()),
        };

        let saved = record_evidence(&conn, &input).unwrap();
        assert!(saved.id > 0);
        assert_eq!(saved.game_id, input.game_id);
        assert_eq!(saved.source_name, "PCGamingWiki");
        assert_eq!(saved.captured_at, 1_700_000_000);

        let rows = get_evidence_for_game(&conn, "steam_400").unwrap();
        assert_eq!(rows, vec![saved]);
    }

    #[test]
    fn record_and_fetch_local_verification_evidence() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        insert_test_game(&conn);

        let input = EvidenceInput {
            game_id: "steam_400".to_string(),
            source_type: "local_verification".to_string(),
            source_name: "User verification".to_string(),
            source_url: None,
            claim_type: "launcher_requirement".to_string(),
            claim: "Reached the main menu while Steam was closed".to_string(),
            confidence: Some("high".to_string()),
            captured_at: Some(1_700_000_100),
            metadata_json: Some(r#"{"executable":"portal2.exe"}"#.to_string()),
        };

        let saved = record_evidence(&conn, &input).unwrap();
        assert_eq!(saved.source_type, "local_verification");
        assert_eq!(saved.source_url, None);

        let rows = get_evidence_for_game(&conn, "steam_400").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].claim, input.claim);
    }

    #[test]
    fn conflicting_evidence_is_preserved_in_stable_order() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        insert_test_game(&conn);

        let external = EvidenceInput {
            game_id: "steam_400".to_string(),
            source_type: "reference".to_string(),
            source_name: "PCGamingWiki".to_string(),
            source_url: Some("https://www.pcgamingwiki.com/wiki/Portal".to_string()),
            claim_type: "launcher_requirement".to_string(),
            claim: "Steam is not required after installation".to_string(),
            confidence: Some("medium".to_string()),
            captured_at: Some(1_700_000_000),
            metadata_json: None,
        };

        let local = EvidenceInput {
            game_id: "steam_400".to_string(),
            source_type: "local_verification".to_string(),
            source_name: "User verification".to_string(),
            source_url: None,
            claim_type: "launcher_requirement".to_string(),
            claim: "Launch failed while Steam was closed".to_string(),
            confidence: Some("high".to_string()),
            captured_at: Some(1_700_000_100),
            metadata_json: None,
        };

        record_evidence(&conn, &local).unwrap();
        record_evidence(&conn, &external).unwrap();

        let rows = get_evidence_for_game(&conn, "steam_400").unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].source_name, "PCGamingWiki");
        assert_eq!(rows[1].source_name, "User verification");
        assert_ne!(rows[0].claim, rows[1].claim);
    }

    #[test]
    fn evidence_with_same_timestamp_orders_by_id() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        insert_test_game(&conn);

        for claim in ["first claim", "second claim"] {
            record_evidence(
                &conn,
                &EvidenceInput {
                    game_id: "steam_400".to_string(),
                    source_type: "manual".to_string(),
                    source_name: "Test".to_string(),
                    source_url: None,
                    claim_type: "note".to_string(),
                    claim: claim.to_string(),
                    confidence: None,
                    captured_at: Some(1_700_000_000),
                    metadata_json: None,
                },
            )
            .unwrap();
        }

        let rows = get_evidence_for_game(&conn, "steam_400").unwrap();
        assert_eq!(rows[0].claim, "first claim");
        assert_eq!(rows[1].claim, "second claim");
        assert!(rows[0].id < rows[1].id);
    }

    #[test]
    fn missing_game_returns_empty_evidence_list() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        let rows = get_evidence_for_game(&conn, "steam_missing").unwrap();
        assert!(rows.is_empty());
    }
}
