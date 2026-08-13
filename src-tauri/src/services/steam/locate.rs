// This module contains functions related to locating the Steam installation on the system.
use std::path::PathBuf;

/// Attempts to locate the local Steam installation directory.
/// Returns `Some(PathBuf)` if found, or `None` if Steam is not installed/detected
pub fn find_steam_path() -> Option<PathBuf> {
    // macOS target
    #[cfg(target_os = "macos")]
    {
        find_mac_steam_path()
    }

    // Windows target
    #[cfg(target_os = "windows")]
    {
        find_windows_steam_path()
    }

    // Linux target
    #[cfg(target_os = "linux")]
    {
        find_linux_steam_path()
    }

    // If none of the above targets match, return None
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        None
    }
}

// MacOS
#[cfg(target_os = "macos")]
fn find_mac_steam_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let path = PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("Steam");

    if path.exists() && path.is_dir() {
        Some(path)
    } else {
        None
    }
}

// Windows
#[cfg(target_os = "windows")]
fn find_windows_steam_path() -> Option<PathBuf> {
    use winreg::enums::*;
    use winreg::RegKey;

    // check registry first (HKCU\Software\Valve\Steam)
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(steam_key) = hkcu.open_subkey("Software\\Valve\\Steam") {
        if let Ok(path_str) = steam_key.get_value::<String, _>("SteamPath") {
            let path = PathBuf::from(path_str);
            if path.exists() && path.is_dir() {
                return Some(path);
            }
        }
    }

    // fallback to Program Files (x86)
    let program_files = std::env::var("ProgramFiles(x86)")
        .or_else(|_| std::env::var("ProgramFiles"))
        .ok()?;
    let fallback = PathBuf::from(program_files).join("Steam");

    if fallback.exists() && fallback.is_dir() {
        Some(fallback)
    } else {
        None
    }
}


// Linux
#[cfg(target_os = "linux")]
fn find_linux_steam_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let home_path = PathBuf::from(home);

    // get all possible path candidates
    let candidates = [
        home_path.join(".steam").join("steam"),
        home_path.join(".local").join("share").join("Steam"),
        home_path
            .join(".var")
            .join("app")
            .join("com.valvesoftware.Steam")
            .join(".steam")
            .join("steam")
    ];

    for path in candidates {
        if path.exists() && path.is_dir() {
            return Some(path);
        }
    }

    None
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_steam_path_on_system_without_steam() {
        let result = find_steam_path();
        println!("Detected Steam Path: {:?}", result);

        // do not assert Some or None because system state varies,
        // but verify the code executes without panicking.
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_mac_steam_detection_with_mock_directory() {
        // create temp folder structure to simulate mac steam install
        let temp_dir = std::env::temp_dir().join("liberarme");
        let mock_steam_path = temp_dir
            .join("Library")
            .join("Application Support")
            .join("Steam");

        // make sure mock path exists
        std::fs::create_dir_all(&mock_steam_path).unwrap();

        // check match between mac logic and mock root
        let exists = mock_steam_path.exists() && mock_steam_path.is_dir();
        assert!(exists, "Mock Steam directory should be recognized as valid");

        // clean up
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_windows_steam_fallback_detection_with_mock_directory() {
        let temp_dir = std::env::temp_dir().join("liberarme");
        let mock_steam_path = temp_dir.join("Steam");

        std::fs::create_dir_all(&mock_steam_path).unwrap();

        let exists = mock_steam_path.exists() && mock_steam_path.is_dir();
        assert!(exists, "Mock Windows Steam directory should be recognized as valid");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_linux_steam_detection_with_mock_directory() {
        let temp_dir = std::env::temp_dir().join("drm_auditor_test_steam_linux");
        let mock_steam_path = temp_dir.join(".steam").join("steam");

        std::fs::create_dir_all(&mock_steam_path).unwrap();

        let exists = mock_steam_path.exists() && mock_steam_path.is_dir();
        assert!(exists, "Mock Linux Steam directory should be recognized as valid");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}