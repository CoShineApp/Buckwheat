//! Recording file scanning and matching

use crate::commands::errors::Error;
use tauri::Manager;

/// Get the recording directory from settings or use default
pub async fn get_recording_directory(app: &tauri::AppHandle) -> Result<String, Error> {
    use tauri_plugin_store::StoreExt;
    
    let store = app
        .store("settings.json")
        .map_err(|e| Error::InitializationError(format!("Failed to open settings store: {}", e)))?;
    
    if let Some(value) = store.get("recordingPath") {
        if let Some(path) = value.as_str() {
            if !path.is_empty() {
                let path_string = path.to_string();
                std::fs::create_dir_all(&path_string).map_err(|e| {
                    Error::RecordingFailed(format!("Failed to create directory: {}", e))
                })?;
                return Ok(path_string);
            }
        }
    }
    
    // Use default: Videos/Buckwheat
    let default_dir = app
        .path()
        .video_dir()
        .map_err(|e| Error::InitializationError(format!("Failed to get videos directory: {}", e)))?
        .join("Buckwheat");
    
    std::fs::create_dir_all(&default_dir).map_err(|e| {
        Error::RecordingFailed(format!("Failed to create default directory: {}", e))
    })?;
    
    default_dir
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| Error::InvalidPath("Failed to convert path to string".to_string()))
}
