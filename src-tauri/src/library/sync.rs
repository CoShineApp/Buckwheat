//! Background sync of recordings cache
//!
//! Scans for new, modified, and deleted recordings and updates the SQLite cache.

use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::database::{self, GameStatsRow, RecordingRow};
use crate::game_detector::slippi_paths;
use crate::slippi;
use std::collections::HashSet;
use std::path::Path;
use std::time::SystemTime;
use tauri::Manager;
use tauri_plugin_store::StoreExt;
use uuid::Uuid;
use walkdir::WalkDir;

/// Sync the recordings cache with the file system
/// This runs in the background after app startup
pub async fn sync_recordings_cache(app: &tauri::AppHandle) -> Result<(), Error> {
    log::info!("🔄 Starting background sync of recordings cache...");
    
    let state = app.state::<AppState>();
    let db = state.database.clone();
    
    // Get directories
    let recording_dir = super::get_recording_directory(app).await?;
    let slippi_dir = get_slippi_directory(app)?;
    
    // Get existing cached paths
    let cached_paths: HashSet<String> = {
        let conn = db.connection();
        database::get_cached_video_paths(&conn)
            .unwrap_or_default()
            .into_iter()
            .collect()
    };
    
    // Scan file system for current recordings
    let mut found_paths: HashSet<String> = HashSet::new();
    let mut new_count = 0;
    let mut updated_count = 0;
    
    for entry in WalkDir::new(&recording_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("mp4") {
            continue;
        }
        
        let video_path = path.to_string_lossy().to_string();
        found_paths.insert(video_path.clone());
        
        // Check if we need to parse this file
        let needs_parse = if cached_paths.contains(&video_path) {
            // Check if file was modified
            check_file_modified(&db, &video_path)
        } else {
            // New file
            true
        };
        
        if needs_parse {
            // Parse and cache the recording
            match parse_and_cache_recording(path, &slippi_dir, &db).await {
                Ok(is_new) => {
                    if is_new {
                        new_count += 1;
                    } else {
                        updated_count += 1;
                    }
                }
                Err(e) => {
                    log::warn!("Failed to parse recording {:?}: {:?}", path, e);
                }
            }
        }
    }
    
    // Remove deleted recordings from cache (by video path)
    let deleted: Vec<_> = cached_paths.difference(&found_paths).cloned().collect();
    if !deleted.is_empty() {
        let conn = db.connection();
        for path in &deleted {
            // Look up by video path and delete
            if let Ok(Some(recording)) = database::get_recording_by_video_path(&conn, path) {
                let _ = database::delete_recording(&conn, &recording.id);
            }
        }
        log::info!("🗑️ Removed {} deleted recordings from cache", deleted.len());
    }
    
    log::info!(
        "✅ Sync complete: {} new, {} updated, {} deleted",
        new_count,
        updated_count,
        deleted.len()
    );
    
    Ok(())
}

/// Check if a cached file has been modified since caching
fn check_file_modified(db: &database::Database, video_path: &str) -> bool {
    let conn = db.connection();
    
    // Look up by video path
    let cached = match database::get_recording_by_video_path(&conn, video_path) {
        Ok(Some(row)) => row,
        _ => return true,
    };
    
    // Get current file modified time
    let current_modified = match std::fs::metadata(video_path) {
        Ok(meta) => meta.modified().ok(),
        Err(_) => return true,
    };
    
    // Compare
    if let (Some(cached_time), Some(current_time)) = (cached.file_modified_at, current_modified) {
        let cached_ts = chrono::DateTime::parse_from_rfc3339(&cached_time)
            .map(|dt| dt.timestamp())
            .unwrap_or(0);
        let current_ts = current_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        
        current_ts > cached_ts
    } else {
        true
    }
}

/// Parse a recording and cache it in the database
async fn parse_and_cache_recording(
    video_path: &Path,
    slippi_dir: &str,
    db: &database::Database,
) -> Result<bool, Error> {
    let video_path_str = video_path.to_string_lossy().to_string();
    
    // Check if this recording already exists (by video path)
    let (id, is_new) = {
        let conn = db.connection();
        match database::get_recording_by_video_path(&conn, &video_path_str) {
            Ok(Some(existing)) => (existing.id, false),
            _ => (Uuid::new_v4().to_string(), true),
        }
    };
    
    // Get file metadata
    let file_meta = std::fs::metadata(video_path)
        .map_err(|e| Error::InvalidPath(format!("Failed to read file metadata: {}", e)))?;
    
    let file_size = file_meta.len() as i64;
    let file_modified_at = file_meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| {
            chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                .unwrap_or_default()
                .to_rfc3339()
        });
    
    // Find matching .slp file
    let video_filename = video_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let slp_path = find_matching_slp_sync(video_filename, slippi_dir);
    
    // Parse Slippi metadata if .slp exists
    let (start_time, game_stats) = if let Some(ref slp) = slp_path {
        match slippi::parse_slp_file(slp) {
            Ok(game) => {
                let meta = slippi::extract_metadata(&game);
                
                // Extract player info
                let (player1, player2) = if meta.players.len() >= 2 {
                    (Some(&meta.players[0]), Some(&meta.players[1]))
                } else if meta.players.len() == 1 {
                    (Some(&meta.players[0]), None)
                } else {
                    (None, None)
                };
                
                // Determine loser port (opposite of winner)
                let loser_port = meta.winner_port.and_then(|winner| {
                    if meta.players.len() >= 2 {
                        meta.players
                            .iter()
                            .find(|p| p.port != winner)
                            .map(|p| p.port)
                    } else {
                        None
                    }
                });
                
                let stats = GameStatsRow {
                    id: id.clone(),
                    player1_id: player1.map(|p| p.player_tag.clone()),
                    player2_id: player2.map(|p| p.player_tag.clone()),
                    player1_port: player1.map(|p| p.port as i32),
                    player2_port: player2.map(|p| p.port as i32),
                    player1_character: player1.map(|p| p.character_id as i32),
                    player2_character: player2.map(|p| p.character_id as i32),
                    player1_color: player1.map(|p| p.character_color as i32),
                    player2_color: player2.map(|p| p.character_color as i32),
                    winner_port: meta.winner_port.map(|p| p as i32),
                    loser_port: loser_port.map(|p| p as i32),
                    stage: Some(meta.stage as i32),
                    game_duration: Some(meta.game_duration),
                    total_frames: Some(meta.total_frames),
                    is_pal: Some(meta.is_pal),
                    played_on: meta.played_on,
                };
                
                (Some(meta.start_time), Some(stats))
            }
            Err(e) => {
                log::warn!("Failed to parse .slp file: {:?}", e);
                (None, None)
            }
        }
    } else {
        // Use file creation time as start_time if no .slp
        let fallback_time = file_meta
            .created()
            .or_else(|_| file_meta.modified())
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| {
                chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                    .unwrap_or_default()
                    .to_rfc3339()
            });
        (fallback_time, None)
    };
    
    // Generate thumbnail (use video filename for thumbnail naming)
    let thumbnail_id = video_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(&id);
    let thumbnail_path = super::thumbnails::generate_thumbnail_if_missing(video_path, thumbnail_id);
    
    // Create recording row
    let row = RecordingRow {
        id: id.clone(),
        video_path: video_path_str,
        slp_path,
        file_size: Some(file_size),
        file_modified_at,
        thumbnail_path,
        start_time: start_time.or_else(|| Some(chrono::Utc::now().to_rfc3339())),
        cached_at: chrono::Utc::now().to_rfc3339(),
        needs_reparse: false,
    };
    
    // Insert/update in database
    {
        let conn = db.connection();
        
        // Upsert recording
        database::upsert_recording(&conn, &row)
            .map_err(|e| Error::InitializationError(format!("Database error: {}", e)))?;
        
        // Upsert game stats if we have them
        if let Some(stats) = game_stats {
            database::upsert_game_stats(&conn, &stats)
                .map_err(|e| Error::InitializationError(format!("Database error (stats): {}", e)))?;
        }
    }
    
    if is_new {
        log::debug!("📦 Cached new recording: {}", id);
    } else {
        log::debug!("🔄 Updated cached recording: {}", id);
    }
    
    Ok(is_new)
}

/// Find matching .slp file (sync version for background task)
fn find_matching_slp_sync(video_filename: &str, slippi_dir: &str) -> Option<String> {
    if !video_filename.starts_with("Game_") {
        return None;
    }
    
    let slp_filename = format!("{}.slp", video_filename);
    
    for entry in WalkDir::new(slippi_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if let Some(filename) = entry.path().file_name().and_then(|s| s.to_str()) {
            if filename == slp_filename {
                return Some(entry.path().to_string_lossy().to_string());
            }
        }
    }
    
    None
}

/// Get Slippi directory from settings
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
