//! Recording file scanning and matching

use crate::app_state::SlpCacheEntry;
use crate::commands::errors::Error;
use crate::slippi::{self, RecordingSession, SlippiMetadata};
use std::path::Path;
use std::time::SystemTime;
use tauri::Manager;
use walkdir::WalkDir;

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

/// Scan for all recordings in the recording directory
pub async fn scan_recordings(
    recording_dir: &str,
    slippi_dir: &str,
    slp_cache: &std::sync::Mutex<std::collections::HashMap<String, SlpCacheEntry>>,
) -> Vec<RecordingSession> {
    let mut recordings = Vec::new();
    
    for entry in WalkDir::new(recording_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("mp4") {
            if let Ok(session) = create_recording_session(path, slippi_dir, slp_cache).await {
                recordings.push(session);
            } else {
                log::warn!("⚠️ Failed to load recording metadata for {:?}", path);
            }
        }
    }
    
    // Sort by start time (newest first)
    recordings.sort_by(|a, b| b.start_time.cmp(&a.start_time));
    recordings
}

/// Create a recording session from a video file path
pub async fn create_recording_session(
    video_path: &Path,
    slippi_dir: &str,
    slp_cache: &std::sync::Mutex<std::collections::HashMap<String, SlpCacheEntry>>,
) -> Result<RecordingSession, Error> {
    let video_path_str = video_path.to_string_lossy().to_string();
    
    // Get file metadata
    let metadata = std::fs::metadata(video_path)
        .map_err(|e| Error::InvalidPath(format!("Failed to read file metadata: {}", e)))?;
    
    let file_size = metadata.len();
    let start_time = metadata
        .created()
        .or_else(|_| metadata.modified())
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| {
            chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                .unwrap_or_default()
                .to_rfc3339()
        })
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    
    // Try to find matching .slp file
    let video_filename = video_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let slp_path = find_matching_slp(video_filename, slippi_dir).await;
    
    // Parse .slp file if found (with caching)
    let (slippi_metadata, duration, end_time) = if let Some(ref slp) = slp_path {
        parse_slp_file_cached(slp, slp_cache).await
    } else {
        (None, None, None)
    };
    
    // Generate ID from filename
    let id = video_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    // Generate thumbnail path
    let thumbnail_path = super::thumbnails::generate_thumbnail_if_missing(video_path, &id);
    
    Ok(RecordingSession {
        id,
        start_time,
        end_time,
        slp_path: slp_path.unwrap_or_default(),
        video_path: Some(video_path_str),
        thumbnail_path,
        duration,
        file_size: Some(file_size),
        slippi_metadata,
    })
}

/// Find a matching .slp file for a video filename
pub async fn find_matching_slp(video_filename: &str, slippi_dir: &str) -> Option<String> {
    if !video_filename.starts_with("Game_") {
        log::debug!(
            "⏭️ Skipping .slp lookup for non-Slippi recording: {}",
            video_filename
        );
        return None;
    }
    
    log::debug!("🔍 Looking for .slp file matching: {}", video_filename);
    
    // Build expected .slp path
    let slp_filename = format!("{}.slp", video_filename);
    
    // Search for exact match
    for entry in WalkDir::new(slippi_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            if filename == slp_filename {
                log::debug!("✅ Found exact match: {:?}", path);
                return Some(path.to_string_lossy().to_string());
            }
        }
    }
    
    log::warn!("⚠️ No matching .slp file found for: {}", video_filename);
    None
}

/// Parse a .slp file with caching
async fn parse_slp_file_cached(
    slp_path: &str,
    cache: &std::sync::Mutex<std::collections::HashMap<String, SlpCacheEntry>>,
) -> (Option<SlippiMetadata>, Option<u64>, Option<String>) {
    // Check cache first
    let file_modified = match std::fs::metadata(slp_path) {
        Ok(meta) => meta.modified().ok(),
        Err(_) => None,
    };
    
    if let (Some(modified_time), Ok(cache_guard)) = (file_modified, cache.lock()) {
        if let Some(entry) = cache_guard.get(slp_path) {
            if entry.modified_time == modified_time {
                log::debug!("✅ Using cached .slp data for: {}", slp_path);
                let metadata = serde_json::from_value(entry.metadata.clone()).ok();
                return (metadata, entry.duration, entry.end_time.clone());
            }
        }
    }
    
    // Parse the file
    let game = match slippi::parse_slp_file(slp_path) {
        Ok(game) => game,
        Err(e) => {
            log::error!("Failed to parse .slp file: {:?}", e);
            return (None, None, None);
        }
    };
    
    let metadata = slippi::extract_metadata(&game);
    let duration_secs = slippi::frames_to_seconds(metadata.game_duration);
    let start_time = metadata.start_time.clone();
    
    // Cache the result
    if let Some(modified_time) = file_modified {
        if let Ok(mut cache_guard) = cache.lock() {
            if let Ok(metadata_json) = serde_json::to_value(&metadata) {
                cache_guard.insert(
                    slp_path.to_string(),
                    SlpCacheEntry {
                        metadata: metadata_json,
                        duration: Some(duration_secs),
                        end_time: Some(start_time.clone()),
                        modified_time,
                    },
                );
                log::debug!("💾 Cached .slp data for: {}", slp_path);
            }
        }
    }
    
    (Some(metadata), Some(duration_secs), Some(start_time))
}

