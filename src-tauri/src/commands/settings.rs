use tauri::{AppHandle, Manager};
use std::path::PathBuf;

#[tauri::command]
pub fn get_settings_path(app: AppHandle) -> Result<String, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    let settings_path = app_data_dir.join("settings.json");
    
    Ok(settings_path
        .to_str()
        .ok_or("Invalid path encoding")?
        .to_string())
}

#[tauri::command]
pub fn open_settings_folder(app: AppHandle) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("open")
        .arg(&app_data_dir)
        .spawn()
        .map_err(|e| format!("Failed to open folder: {}", e))?;

    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer")
        .arg(&app_data_dir)
        .spawn()
        .map_err(|e| format!("Failed to open folder: {}", e))?;

    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open")
        .arg(&app_data_dir)
        .spawn()
        .map_err(|e| format!("Failed to open folder: {}", e))?;

    Ok(())
}

/// Get a setting value from the settings store
/// Returns the value as a string, or None if the setting doesn't exist
#[tauri::command]
pub async fn get_setting(app: AppHandle, key: String) -> Result<Option<String>, String> {
    let path = app.path().app_data_dir().map_err(|e| format!("Failed to get app data directory: {}", e))?;
    let store_path = path.join("settings.json");
    
    // Try to read setting from settings file
    if store_path.exists() {
        if let Ok(contents) = std::fs::read_to_string(&store_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
                if let Some(value) = json.get(&key) {
                    // Return as string if it's a string, or serialize to string if it's another type
                    if let Some(str_val) = value.as_str() {
                        return Ok(Some(str_val.to_string()));
                    } else if let Some(bool_val) = value.as_bool() {
                        return Ok(Some(bool_val.to_string()));
                    } else if let Some(num_val) = value.as_number() {
                        return Ok(Some(num_val.to_string()));
                    }
                }
            }
        }
    }
    
    Ok(None)
}

/// Get the recording output directory, resolving defaults and ensuring it exists
/// Returns the directory path (not a file path)
#[tauri::command]
pub async fn get_recording_directory(app: AppHandle) -> Result<String, String> {
    // Get recordingPath from settings
    let recording_path = get_setting(app.clone(), "recordingPath".to_string()).await?;
    
    // Determine the final path
    let final_path = if let Some(path) = recording_path {
        if path.trim().is_empty() {
            // Use default: ~/Movies/Bunbun Recordings
            if let Some(home) = std::env::var_os("HOME") {
                PathBuf::from(home).join("Movies").join("Bunbun Recordings")
            } else {
                app.path().app_data_dir()
                    .map_err(|e| format!("Failed to get app data directory: {}", e))?
                    .join("Recordings")
            }
        } else {
            // Expand ~ if present
            let expanded = if path.starts_with("~/") {
                if let Some(home) = std::env::var_os("HOME") {
                    PathBuf::from(home).join(&path[2..])
                } else {
                    return Err("HOME environment variable not set".to_string());
                }
            } else {
                PathBuf::from(path)
            };
            expanded
        }
    } else {
        // No setting at all, use default
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join("Movies").join("Bunbun Recordings")
        } else {
            app.path().app_data_dir()
                .map_err(|e| format!("Failed to get app data directory: {}", e))?
                .join("Recordings")
        }
    };
    
    // Ensure the directory exists
    if let Err(e) = std::fs::create_dir_all(&final_path) {
        return Err(format!("Failed to create recording directory: {}", e));
    }
    
    Ok(final_path
        .to_str()
        .ok_or("Invalid path encoding")?
        .to_string())
}


