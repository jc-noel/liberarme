// discovers installed games within library folders
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum AppManifestReadError {
    Io(std::io::Error),
    Parse(keyvalues_parser::error::Error),
}

impl From<std::io::Error> for AppManifestReadError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<keyvalues_parser::error::Error> for AppManifestReadError {
    fn from(error: keyvalues_parser::error::Error) -> Self {
        Self::Parse(error)
    }
}

// structure for serde to parse into
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct AppManifest {
    pub app_id: u32,
    pub name: String,
    pub install_dir: PathBuf,
    pub install_size: u64,
    pub last_updated: Option<u64>,
}

/// reads `appmanifest_*.acf` and parses it
pub fn read_app_manifest(path: &Path) -> Result<Option<AppManifest>, AppManifestReadError> {
    let content = std::fs::read_to_string(path)?;
    Ok(parse_app_manifest(&content)?)
}

/// parses vdf content of the steam appmanifest file
pub fn parse_app_manifest(
    vdf_content: &str,
) -> Result<Option<AppManifest>, keyvalues_parser::error::Error> {
    let vdf = keyvalues_parser::Parser::new()
        .literal_special_chars(true)
        .parse(vdf_content)?;

    if vdf.key != "AppState" {
        return Ok(None);
    }

    let manifest = (|| {
        let app_state = vdf.value.get_obj()?;

        let app_id = app_state
            .get("appid")?
            .first()?
            .get_str()?
            .parse()
            .ok()?;

        let name = app_state
            .get("name")?
            .first()?
            .get_str()?
            .to_string();

        let install_dir = PathBuf::from(
            app_state
                .get("installdir")?
                .first()?
                .get_str()?
        );

        // parse SizeOnDisk, default to 0 if missing
        let install_size = app_state
            .get("SizeOnDisk")
            .and_then(|vals| vals.first())
            .and_then(|v| v.get_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // parse lastupdated timestamp, None if steam doesn't report one
        let last_updated = app_state
            .get("LastUpdated")
            .and_then(|vals| vals.first())
            .and_then(|v| v.get_str())
            .and_then(|s| s.parse().ok());

        Some(AppManifest {
            app_id,
            name,
            install_dir,
            install_size,
            last_updated,
        })
    })();

    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_app_manifest() {
        let manifest_content = r#"
"AppState"
{
	"appid"		"400"
	"name"		"Portal"
	"installdir"	"Portal"
	"SizeOnDisk"	"4294967296"
	"LastUpdated"	"1625000000"
}
"#;

        let parsed = parse_app_manifest(manifest_content).unwrap();
        assert_eq!(
            parsed,
            Some(AppManifest {
                app_id: 400,
                name: "Portal".to_string(),
                install_dir: PathBuf::from("Portal"),
                install_size: 4294967296,
                last_updated: Some(1625000000),
            })
        );
    }

    #[test]
    fn test_parse_app_manifest_with_missing_optional_fields() {
        let manifest_content = r#"
"AppState"
{
	"appid"		"730"
	"name"		"Counter-Strike 2"
	"installdir"	"Counter-Strike Global Offensive"
}
"#;

        let parsed = parse_app_manifest(manifest_content).unwrap();
        assert_eq!(
            parsed,
            Some(AppManifest {
                app_id: 730,
                name: "Counter-Strike 2".to_string(),
                install_dir: PathBuf::from("Counter-Strike Global Offensive"),
                install_size: 0,
                last_updated: None,
            })
        );
    }

    #[test]
    fn test_parse_invalid_app_manifest_returns_error() {
        assert!(parse_app_manifest("not a vdf").is_err());
    }

    #[test]
    fn test_parse_non_app_manifest_returns_none() {
        assert_eq!(parse_app_manifest("\"NotAppState\" {}").unwrap(), None);
    }
}