//! Storage and file operations

use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::database;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

/// Storage usage information
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageUsage {
    pub total_bytes: i64,
    pub recording_count: i32,
}

/// Get current storage usage for recordings (excluding clips)
#[tauri::command]
pub async fn get_storage_usage(state: State<'_, AppState>) -> Result<StorageUsage, Error> {
    let db = state.database.clone();
    let conn = db.connection();

    let total_bytes = database::get_total_storage_bytes(&conn)
        .map_err(|e| Error::RecordingFailed(format!("Failed to get storage bytes: {}", e)))?;

    let recording_count = database::get_recording_count(&conn)
        .map_err(|e| Error::RecordingFailed(format!("Failed to get recording count: {}", e)))?;

    Ok(StorageUsage {
        total_bytes,
        recording_count,
    })
}

/// Open a video file in the default player
#[tauri::command]
pub async fn open_video(video_path: String) -> Result<(), Error> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &video_path])
            .spawn()
            .map_err(|e| Error::RecordingFailed(format!("Failed to open video: {}", e)))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&video_path)
            .spawn()
            .map_err(|e| Error::RecordingFailed(format!("Failed to open video: {}", e)))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&video_path)
            .spawn()
            .map_err(|e| Error::RecordingFailed(format!("Failed to open video: {}", e)))?;
    }

    Ok(())
}

/// Open the folder containing a video file
#[tauri::command]
pub async fn open_recording_folder(video_path: String) -> Result<(), Error> {
    let path = std::path::Path::new(&video_path);
    let folder = path
        .parent()
        .ok_or_else(|| Error::InvalidPath("Failed to get parent directory".to_string()))?;

    open_folder(folder)
}

/// Open a file's location in the system file explorer
#[tauri::command]
pub fn open_file_location(path: String) -> Result<(), Error> {
    let file_path = Path::new(&path);
    let dir_path = if file_path.is_file() {
        file_path
            .parent()
            .ok_or_else(|| Error::InvalidPath("Could not get parent directory".to_string()))?
    } else {
        file_path
    };

    open_folder(dir_path)
}

fn open_folder(folder: &Path) -> Result<(), Error> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(folder)
            .spawn()
            .map_err(|e| Error::RecordingFailed(format!("Failed to open folder: {}", e)))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(folder)
            .spawn()
            .map_err(|e| Error::RecordingFailed(format!("Failed to open folder: {}", e)))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(folder)
            .spawn()
            .map_err(|e| Error::RecordingFailed(format!("Failed to open folder: {}", e)))?;
    }

    Ok(())
}
