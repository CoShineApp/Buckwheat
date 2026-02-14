//! Storage management for recordings
//!
//! This module handles automatic cleanup of old recordings when storage
//! limits are exceeded.

use std::path::Path;
use tauri::{AppHandle, Manager};

use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::database;

/// Storage cleanup result containing deleted paths
#[derive(Debug, Clone, serde::Serialize)]
pub struct StorageCleanupResult {
    /// Paths of deleted video files
    pub deleted_paths: Vec<String>,
    /// Total bytes freed
    pub bytes_freed: i64,
}

/// Read the storage limit setting from the settings store
/// Returns the limit in bytes, or 0 for unlimited
fn get_storage_limit_bytes(app: &AppHandle) -> Result<i64, Error> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| Error::RecordingFailed(format!("Failed to get app data directory: {}", e)))?;
    let store_path = path.join("settings.json");

    if store_path.exists() {
        if let Ok(contents) = std::fs::read_to_string(&store_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
                if let Some(limit) = json.get("storageLimit") {
                    if let Some(gb) = limit.as_f64() {
                        // Convert GB to bytes
                        return Ok((gb * 1024.0 * 1024.0 * 1024.0) as i64);
                    }
                }
            }
        }
    }

    Ok(0) // Default: unlimited
}

/// Enforce storage limit by deleting oldest recordings until under the limit
/// 
/// This function:
/// 1. Reads the storage limit from settings
/// 2. Queries total storage used (excluding clips)
/// 3. While over limit: deletes oldest recording (video, thumbnail, DB entry)
/// 4. Returns list of deleted video paths
/// 
/// Note: Clips are excluded from both the storage calculation and deletion
pub async fn enforce_storage_limit(app: &AppHandle) -> Result<StorageCleanupResult, Error> {
    let limit_bytes = get_storage_limit_bytes(app)?;
    
    // If limit is 0, skip enforcement (unlimited)
    if limit_bytes == 0 {
        log::debug!("[Storage] No storage limit set, skipping enforcement");
        return Ok(StorageCleanupResult {
            deleted_paths: vec![],
            bytes_freed: 0,
        });
    }

    let state = app.state::<AppState>();
    let db = state.database.clone();
    let conn = db.connection();

    let mut deleted_paths: Vec<String> = vec![];
    let mut bytes_freed: i64 = 0;

    loop {
        // Check current storage usage
        let current_bytes = database::get_total_storage_bytes(&conn)
            .map_err(|e| Error::RecordingFailed(format!("Failed to get storage usage: {}", e)))?;

        if current_bytes <= limit_bytes {
            log::debug!(
                "[Storage] Under limit: {} bytes used, {} bytes limit",
                current_bytes,
                limit_bytes
            );
            break;
        }

        log::info!(
            "[Storage] Over limit: {} bytes used, {} bytes limit. Deleting oldest recording...",
            current_bytes,
            limit_bytes
        );

        // Get the oldest recording
        let oldest = database::get_oldest_recordings(&conn, 1)
            .map_err(|e| Error::RecordingFailed(format!("Failed to get oldest recordings: {}", e)))?;

        if oldest.is_empty() {
            log::warn!("[Storage] No recordings to delete, but still over limit");
            break;
        }

        let recording = &oldest[0];
        let recording_id = recording.id.clone();
        let video_path = recording.video_path.clone();
        let file_size = recording.file_size.unwrap_or(0);

        // Delete video file
        if Path::new(&video_path).exists() {
            if let Err(e) = std::fs::remove_file(&video_path) {
                log::error!("[Storage] Failed to delete video file {}: {}", video_path, e);
            } else {
                log::info!("[Storage] Deleted video file: {}", video_path);
            }
        }

        // Delete thumbnail if exists
        if let Some(ref thumb_path) = recording.thumbnail_path {
            if Path::new(thumb_path).exists() {
                if let Err(e) = std::fs::remove_file(thumb_path) {
                    log::error!("[Storage] Failed to delete thumbnail {}: {}", thumb_path, e);
                }
            }
        }

        // Delete from database
        if let Err(e) = database::delete_recording(&conn, &recording_id) {
            log::error!("[Storage] Failed to delete recording from DB: {}", e);
        }

        deleted_paths.push(video_path);
        bytes_freed += file_size;
    }

    if !deleted_paths.is_empty() {
        log::info!(
            "[Storage] Cleanup complete: deleted {} recordings, freed {} bytes",
            deleted_paths.len(),
            bytes_freed
        );
    }

    Ok(StorageCleanupResult {
        deleted_paths,
        bytes_freed,
    })
}
