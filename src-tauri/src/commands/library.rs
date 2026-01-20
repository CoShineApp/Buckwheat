//! Library management commands
//!
//! Commands for browsing recordings, clips, and managing files.

use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::database::{self, RecordingWithStats, AggregatedPlayerStats, StatsFilter, AvailableFilterOptions};
use crate::slippi::{PlayerInfo, RecordingSession, SlippiMetadata};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::State;

/// Response for paginated recordings
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedRecordings {
    pub recordings: Vec<RecordingSession>,
    pub total: i32,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

/// Get list of recorded sessions with pagination
/// Returns cached data from SQLite for instant response
#[tauri::command]
pub async fn get_recordings(
    page: Option<i32>,
    per_page: Option<i32>,
    state: State<'_, AppState>,
) -> Result<PaginatedRecordings, Error> {
    let page = page.unwrap_or(1).max(1);
    let per_page = per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;
    
    log::debug!("📂 Loading recordings from cache (page {}, {} per page)", page, per_page);
    
    let db = state.database.clone();
    let conn = db.connection();
    
    let (rows, total) = database::get_recordings_paginated(&conn, per_page, offset)
        .map_err(|e| Error::InitializationError(format!("Database error: {}", e)))?;
    
    // Convert database rows to RecordingSession
    let recordings: Vec<RecordingSession> = rows
        .into_iter()
        .map(recording_with_stats_to_session)
        .collect();
    
    let total_pages = (total as f64 / per_page as f64).ceil() as i32;
    
    log::info!("✅ Loaded {} recording(s) from cache (page {}/{})", recordings.len(), page, total_pages);
    
    Ok(PaginatedRecordings {
        recordings,
        total,
        page,
        per_page,
        total_pages,
    })
}

/// Get list of all clips (clips don't use pagination yet, they're usually fewer)
#[tauri::command]
pub async fn get_clips(
    state: State<'_, AppState>,
) -> Result<Vec<RecordingSession>, Error> {
    log::debug!("📂 Loading clips from cache...");
    
    let db = state.database.clone();
    let conn = db.connection();
    
    // Get all recordings and filter to clips (those in Clips folder)
    let all = database::get_all_recordings(&conn)
        .map_err(|e| Error::InitializationError(format!("Database error: {}", e)))?;
    
    let clips: Vec<RecordingSession> = all
        .into_iter()
        .filter(|row| row.video_path.contains("Clips"))
        .map(|row| recording_row_to_session(row, None))
        .collect();
    
    log::info!("✅ Found {} clip(s)", clips.len());
    Ok(clips)
}

/// Delete a recording (video file and cache entry)
#[tauri::command]
pub async fn delete_recording(
    video_path: Option<String>,
    _slp_path: String,
    state: State<'_, AppState>,
) -> Result<(), Error> {
    if let Some(ref video) = video_path {
        if !video.is_empty() {
            let db = state.database.clone();
            let conn = db.connection();
            
            // Look up by video path and delete from cache
            if let Ok(Some(recording)) = database::get_recording_by_video_path(&conn, video) {
                let _ = database::delete_recording(&conn, &recording.id);
                log::debug!("🗑️ Removed {} from cache", recording.id);
            }
            
            // Delete the actual file
            if std::path::Path::new(video).exists() {
                std::fs::remove_file(video)
                    .map_err(|e| Error::RecordingFailed(format!("Failed to delete video: {}", e)))?;
                log::info!("✅ Deleted video: {}", video);
            }
        }
    }
    Ok(())
}

/// Manually trigger a cache refresh
#[tauri::command]
pub async fn refresh_recordings_cache(app: tauri::AppHandle) -> Result<(), Error> {
    log::info!("🔄 Manual cache refresh triggered");
    crate::library::sync_recordings_cache(&app).await
}

// ============================================================================
// COMPUTED STATS (from slippi-js)
// ============================================================================

/// Computed game stats from the frontend (slippi-js)
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputedGameStats {
    pub recording_id: String,
    pub slp_path: String,
    
    // Game metadata
    pub stage: i32,
    pub game_duration: i32,
    pub total_frames: i32,
    pub is_pal: bool,
    pub played_on: Option<String>,
    pub match_id: Option<String>,
    pub game_number: Option<i32>,
    
    // Outcome
    pub winner_index: Option<i32>,
    pub loser_index: Option<i32>,
    pub game_end_method: Option<String>,
    
    // Player stats
    pub players: Vec<ComputedPlayerStats>,
}

/// Computed player stats from the frontend (slippi-js)
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputedPlayerStats {
    pub player_index: i32,
    pub connect_code: Option<String>,
    pub display_name: Option<String>,
    pub character_id: i32,
    pub character_color: i32,
    pub port: i32,
    
    // Overall performance
    pub total_damage: f64,
    pub kill_count: i32,
    pub conversion_count: i32,
    pub successful_conversions: i32,
    pub openings_per_kill: Option<f64>,
    pub damage_per_opening: Option<f64>,
    pub neutral_win_ratio: Option<f64>,
    pub counter_hit_ratio: Option<f64>,
    pub beneficial_trade_ratio: Option<f64>,
    
    // Input stats
    pub inputs_total: i32,
    pub inputs_per_minute: Option<f64>,
    pub avg_kill_percent: Option<f64>,
    
    // Action counts
    pub wavedash_count: i32,
    pub waveland_count: i32,
    pub air_dodge_count: i32,
    pub dash_dance_count: i32,
    pub spot_dodge_count: i32,
    pub ledgegrab_count: i32,
    pub roll_count: i32,
    pub grab_count: i32,
    pub throw_count: i32,
    pub ground_tech_count: i32,
    pub wall_tech_count: i32,
    pub wall_jump_tech_count: i32,
    
    // L-Cancel stats
    pub l_cancel_success_count: i32,
    pub l_cancel_fail_count: i32,
    
    // Final state
    pub stocks_remaining: i32,
    pub final_percent: Option<f64>,
}

/// Save computed stats from slippi-js to the database
#[tauri::command]
pub async fn save_computed_stats(
    stats: ComputedGameStats,
    state: State<'_, AppState>,
) -> Result<(), Error> {
    log::info!("[SlippiStats] Saving computed stats for recording: {}", stats.recording_id);
    
    let db = state.database.clone();
    let conn = db.connection();
    
    // Update game_stats with match info
    if let Ok(Some(_existing)) = database::get_game_stats_by_id(&conn, &stats.recording_id) {
        // Update existing game stats with new match info columns
        // Note: We intentionally do NOT update winner_port/loser_port here.
        // The initial sync from Rust/peppi sets these correctly. Overwriting them
        // from frontend slippi-js parsing caused race conditions and inconsistencies.
        conn.execute(
            "UPDATE game_stats SET 
                match_id = ?, game_number = ?, game_end_method = ?,
                stage = ?, game_duration = ?, total_frames = ?,
                is_pal = ?, played_on = ?
            WHERE id = ?",
            rusqlite::params![
                stats.match_id,
                stats.game_number,
                stats.game_end_method,
                stats.stage,
                stats.game_duration,
                stats.total_frames,
                stats.is_pal as i32,
                stats.played_on,
                stats.recording_id,
            ],
        ).map_err(|e| Error::RecordingFailed(format!("Failed to update game stats: {}", e)))?;
        
        log::debug!("Updated game_stats for {}", stats.recording_id);
    } else {
        log::debug!("No existing game_stats found for {}, will be created by sync", stats.recording_id);
    }
    
    // Save player stats
    for player in &stats.players {
        let player_stats = database::PlayerStatsRow {
            id: None,
            recording_id: stats.recording_id.clone(),
            player_index: player.player_index,
            connect_code: player.connect_code.clone(),
            display_name: player.display_name.clone(),
            character_id: player.character_id,
            character_color: player.character_color,
            port: player.port,
            total_damage: player.total_damage,
            kill_count: player.kill_count,
            conversion_count: player.conversion_count,
            successful_conversions: player.successful_conversions,
            openings_per_kill: player.openings_per_kill,
            damage_per_opening: player.damage_per_opening,
            neutral_win_ratio: player.neutral_win_ratio,
            counter_hit_ratio: player.counter_hit_ratio,
            beneficial_trade_ratio: player.beneficial_trade_ratio,
            inputs_total: player.inputs_total,
            inputs_per_minute: player.inputs_per_minute,
            avg_kill_percent: player.avg_kill_percent,
            wavedash_count: player.wavedash_count,
            waveland_count: player.waveland_count,
            air_dodge_count: player.air_dodge_count,
            dash_dance_count: player.dash_dance_count,
            spot_dodge_count: player.spot_dodge_count,
            ledgegrab_count: player.ledgegrab_count,
            roll_count: player.roll_count,
            grab_count: player.grab_count,
            throw_count: player.throw_count,
            ground_tech_count: player.ground_tech_count,
            wall_tech_count: player.wall_tech_count,
            wall_jump_tech_count: player.wall_jump_tech_count,
            l_cancel_success_count: player.l_cancel_success_count,
            l_cancel_fail_count: player.l_cancel_fail_count,
            stocks_remaining: player.stocks_remaining,
            final_percent: player.final_percent,
        };
        
        database::upsert_player_stats(&conn, &player_stats)
            .map_err(|e| Error::RecordingFailed(format!("Failed to save player stats: {}", e)))?;
        
        log::debug!(
            "Saved stats for player {} ({:?}) - {} kills, L-cancel: {}/{}",
            player.player_index,
            player.connect_code,
            player.kill_count,
            player.l_cancel_success_count,
            player.l_cancel_success_count + player.l_cancel_fail_count
        );
    }
    
    log::info!("[SlippiStats] Saved computed stats for {} players", stats.players.len());
    Ok(())
}

/// Get player stats for a recording
#[tauri::command]
pub async fn get_player_stats(
    recording_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<database::PlayerStatsRow>, Error> {
    let db = state.database.clone();
    let conn = db.connection();
    
    database::get_player_stats_by_recording(&conn, &recording_id)
        .map_err(|e| Error::RecordingFailed(format!("Failed to get player stats: {}", e)))
}

/// Get aggregated stats for a player across all recordings
#[tauri::command]
pub async fn get_total_player_stats(
    connect_code: String,
    filter: Option<StatsFilter>,
    state: State<'_, AppState>,
) -> Result<AggregatedPlayerStats, Error> {
    log::debug!(
        "Getting total stats for {} with filter: {:?}", 
        connect_code, 
        filter
    );
    
    let db = state.database.clone();
    let conn = db.connection();
    
    database::get_aggregated_player_stats(&conn, &connect_code, filter)
        .map_err(|e| Error::RecordingFailed(format!("Failed to get aggregated stats: {}", e)))
}

/// Get available filter options (connect codes, characters, stages) from the database
#[tauri::command]
pub async fn get_available_filter_options(
    state: State<'_, AppState>,
) -> Result<AvailableFilterOptions, Error> {
    let db = state.database.clone();
    let conn = db.connection();
    
    database::get_available_filter_options(&conn)
        .map_err(|e| Error::RecordingFailed(format!("Failed to get filter options: {}", e)))
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

/// Convert a RecordingWithStats (from paginated query) to RecordingSession
fn recording_with_stats_to_session(rws: RecordingWithStats) -> RecordingSession {
    let row = rws.recording;
    let stats = rws.stats;
    
    recording_row_to_session(row, stats)
}

/// Convert a database row + optional stats to a RecordingSession
fn recording_row_to_session(
    row: database::RecordingRow,
    stats: Option<database::GameStatsRow>,
) -> RecordingSession {
    // Build SlippiMetadata from stats if available
    let slippi_metadata = stats.as_ref().map(|s| {
        // Build players array from stats
        let mut players = Vec::new();
        
        if let Some(p1_char) = s.player1_character {
            players.push(PlayerInfo {
                character_id: p1_char as u8,
                character_color: s.player1_color.unwrap_or(0) as u8,
                player_tag: s.player1_id.clone().unwrap_or_default(),
                port: s.player1_port.unwrap_or(1) as u8,
            });
        }
        
        if let Some(p2_char) = s.player2_character {
            players.push(PlayerInfo {
                character_id: p2_char as u8,
                character_color: s.player2_color.unwrap_or(0) as u8,
                player_tag: s.player2_id.clone().unwrap_or_default(),
                port: s.player2_port.unwrap_or(2) as u8,
            });
        }
        
        // Build characters array
        let characters: Vec<u8> = players.iter().map(|p| p.character_id).collect();
        
        SlippiMetadata {
            characters,
            stage: s.stage.unwrap_or(0) as u16,
            players,
            game_duration: s.game_duration.unwrap_or(0),
            start_time: row.start_time.clone().unwrap_or_default(),
            is_pal: s.is_pal.unwrap_or(false),
            winner_port: s.winner_port.map(|p| p as u8),
            played_on: s.played_on.clone(),
            total_frames: s.total_frames.unwrap_or(0),
        }
    });
    
    // Calculate duration from stats if available
    let duration = stats
        .as_ref()
        .and_then(|s| s.game_duration)
        .map(|d| (d as f64 / 60.0) as u64);
    
    RecordingSession {
        id: row.id,
        start_time: row.start_time.unwrap_or_default(),
        end_time: None,
        slp_path: row.slp_path.unwrap_or_default(),
        video_path: Some(row.video_path),
        thumbnail_path: row.thumbnail_path,
        duration,
        file_size: row.file_size.map(|s| s as u64),
        slippi_metadata,
    }
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
