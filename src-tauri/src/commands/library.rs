//! Library management commands
//!
//! Commands for browsing recordings, clips, and managing files.

use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::game_detector::slippi_paths;
use crate::library;
use crate::slippi::RecordingSession;
use std::path::Path;
use tauri::State;
use tauri_plugin_store::StoreExt;
use walkdir::WalkDir;

/// Get list of all recorded sessions
#[tauri::command]
pub async fn get_recordings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<RecordingSession>, Error> {
    log::debug!("📂 Scanning for recordings...");
    
    // Get recording directory
    let recording_dir = match library::get_recording_directory(&app).await {
        Ok(dir) => dir,
        Err(e) => {
            log::error!("Failed to get recording directory: {:?}", e);
            return Ok(Vec::new());
        }
    };
    
    log::debug!("📁 Recording directory: {}", recording_dir);
    
    // Get Slippi directory
    let slippi_dir = get_slippi_directory(&app)?;
    log::debug!("📁 Slippi directory: {}", slippi_dir);
    
    // Scan for recordings
    let recordings = library::scan_recordings(&recording_dir, &slippi_dir, &state.slp_cache).await;
    
    log::info!("✅ Found {} recording(s)", recordings.len());
    Ok(recordings)
}

/// Get list of all clips
#[tauri::command]
pub async fn get_clips(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<RecordingSession>, Error> {
    log::debug!("📂 Scanning for clips...");
    
    // Get clips directory (sibling to recordings)
    let recording_dir = match library::get_recording_directory(&app).await {
        Ok(dir) => dir,
        Err(e) => {
            log::error!("Failed to get recording directory: {:?}", e);
            return Ok(Vec::new());
        }
    };
    
    let recording_dir_path = Path::new(&recording_dir);
    let clips_parent_dir = recording_dir_path.parent().unwrap_or(recording_dir_path);
    let clips_dir_path = clips_parent_dir.join("Clips");
    
    let clips_dir = match clips_dir_path.to_str() {
        Some(path) => path.to_string(),
        None => {
            log::error!("❌ Failed to determine clips directory path");
            return Err(Error::InvalidPath(
                "Failed to determine clips directory path".to_string(),
            ));
        }
    };
    
    log::debug!("📁 Clips directory: {}", clips_dir);
    
    // Check if clips directory exists
    if !clips_dir_path.exists() {
        log::debug!("Clips directory doesn't exist yet");
        return Ok(Vec::new());
    }
    
    // Get Slippi directory
    let slippi_dir = get_slippi_directory(&app)?;
    
    // Scan for clips
    let mut clips = Vec::new();
    
    for entry in WalkDir::new(&clips_dir_path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("mp4") {
            if let Ok(session) = library::create_recording_session(path, &slippi_dir, &state.slp_cache).await {
                clips.push(session);
            } else {
                log::warn!("⚠️ Failed to load clip metadata for {:?}", path);
            }
        }
    }
    
    // Sort by start time (newest first)
    clips.sort_by(|a, b| b.start_time.cmp(&a.start_time));
    
    log::info!("✅ Found {} clip(s)", clips.len());
    Ok(clips)
}

/// Delete a recording (video file)
#[tauri::command]
pub async fn delete_recording(video_path: Option<String>, _slp_path: String) -> Result<(), Error> {
    if let Some(video) = video_path {
        if !video.is_empty() && std::path::Path::new(&video).exists() {
            std::fs::remove_file(&video)
                .map_err(|e| Error::RecordingFailed(format!("Failed to delete video: {}", e)))?;
            log::info!("✅ Deleted video: {}", video);
        }
    }
    Ok(())
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

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn get_slippi_directory(app: &tauri::AppHandle) -> Result<String, Error> {
    let store = app.store("settings.json").map_err(|e| {
        Error::InitializationError(format!("Failed to open settings store: {}", e))
    })?;
    
    if let Some(value) = store.get("slippiPath") {
        if let Some(path) = value.as_str() {
            if !path.is_empty() {
                return Ok(path.to_string());
            }
        }
    }
    
    Ok(slippi_paths::get_default_slippi_path()
        .to_str()
        .unwrap_or("")
        .to_string())
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

