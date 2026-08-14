// Discovers steam library folders
use std::path::PathBuf;

/// Parses VDF text and returns list of directory paths
pub fn parse_library_folders(vdf_content: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    match keyvalues_parser::parse(vdf_content) {
        Ok(vdf) => {
            if let Some(root_obj) = vdf.value.get_obj() {
                for (_key, values) in root_obj.iter() {
                    for value in values {
                        if let Some(folder_obj) = value.get_obj() {
                            if let Some(path_values) = folder_obj.get("path") {
                                if let Some(path_str) = path_values.first().and_then(|v| v.get_str()) {
                                    paths.push(PathBuf::from(path_str));
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Failed to parse libraryfolders.vdf: {:?}", err);
        }
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_library_vdf() {
        let vdf = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"/Users/james/Library/Application Support/Steam"
		"label"		""
	}
}
"#;

        let paths = parse_library_folders(vdf);
        assert_eq!(paths.len(), 1);
        assert_eq!(
            paths[0],
            PathBuf::from("/Users/james/Library/Application Support/Steam")
        );
    }

    #[test]
    fn test_parse_multi_library_vdf() {
        let vdf = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"label"		""
	}
	"1"
	{
		"path"		"D:\\Games\\SteamLibrary"
		"label"		"Secondary SSD"
	}
}
"#;

        let paths = parse_library_folders(vdf);
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0], PathBuf::from("C:\\Program Files (x86)\\Steam"));
        assert_eq!(paths[1], PathBuf::from("D:\\Games\\SteamLibrary"));
    }

    #[test]
    fn test_parse_invalid_vdf_returns_empty_vec() {
        let bad_vdf = "this is not a valid vdf file";
        let paths = parse_library_folders(bad_vdf);
        assert!(paths.is_empty());
    }
}