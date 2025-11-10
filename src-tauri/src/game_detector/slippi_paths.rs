use std::path::PathBuf;

/// Get the default Slippi replay folder path based on the operating system
pub fn get_default_slippi_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| String::from("C:\\"));
        PathBuf::from(appdata)
            .join("Slippi Launcher")
            .join("netplay")
    }

    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/"));
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Slippi Launcher")
            .join("netplay")
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/"));
        PathBuf::from(home)
            .join(".config")
            .join("Slippi Launcher")
            .join("netplay")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_path_returns_valid_path() {
        let path = get_default_slippi_path();
        assert!(path.to_str().is_some());
    }
}

