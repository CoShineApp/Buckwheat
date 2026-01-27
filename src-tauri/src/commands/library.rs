//! Library management commands
//!
//! Commands for browsing recordings, clips, and managing files.

use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::database::{self, AggregatedPlayerStats, StatsFilter, AvailableFilterOptions};
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
        .map(|row| recording_row_to_session(row, None, Vec::new()))
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
    
    // Timestamp when game was played (ISO 8601)
    pub created_at: Option<String>,
    
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

/// Save computed stats from slippi-js to the database.
/// This is the SINGLE ENTRY POINT for saving game statistics.
/// Creates/updates both game_stats and player_stats tables.
#[tauri::command]
pub async fn save_computed_stats(
    stats: ComputedGameStats,
    state: State<'_, AppState>,
) -> Result<(), Error> {
    log::info!("[SlippiStats] Saving computed stats for recording: {}", stats.recording_id);
    
    let db = state.database.clone();
    let conn = db.connection();
    
    // Get player info for game_stats
    let p1 = stats.players.get(0);
    let p2 = stats.players.get(1);
    
    // Determine winner by stocks remaining:
    // 1. If one player has 0 stocks, the other wins
    // 2. If both have stocks, the one with MORE stocks wins
    // 3. If tied stocks, no winner (LRAS quit or timeout)
    let (winner_port, loser_port) = if stats.players.len() == 2 {
        let player_a = &stats.players[0];
        let player_b = &stats.players[1];
        
        let a_stocks = player_a.stocks_remaining;
        let b_stocks = player_b.stocks_remaining;
        
        if a_stocks > b_stocks {
            // Player A has more stocks = winner
            (Some(player_a.port), Some(player_b.port))
        } else if b_stocks > a_stocks {
            // Player B has more stocks = winner
            (Some(player_b.port), Some(player_a.port))
        } else {
            // Tied stocks - no winner (probably LRAS quit with same stocks)
            log::warn!("[SlippiStats] No winner: tied stocks ({}) for {}", a_stocks, stats.recording_id);
            (None, None)
        }
    } else {
        log::error!("[SlippiStats] Expected 2 players for {}, got {}", stats.recording_id, stats.players.len());
        (None, None)
    };
    
    // Build and upsert game_stats (creates if missing, updates if exists)
    let game_stats = database::GameStatsRow {
        id: stats.recording_id.clone(),
        player1_id: p1.and_then(|p| p.connect_code.clone()),
        player2_id: p2.and_then(|p| p.connect_code.clone()),
        player1_port: p1.map(|p| p.port),
        player2_port: p2.map(|p| p.port),
        player1_character: p1.map(|p| p.character_id),
        player2_character: p2.map(|p| p.character_id),
        player1_color: p1.map(|p| p.character_color),
        player2_color: p2.map(|p| p.character_color),
        winner_port,
        loser_port,
        stage: Some(stats.stage),
        game_duration: Some(stats.game_duration),
        total_frames: Some(stats.total_frames),
        is_pal: Some(stats.is_pal),
        played_on: stats.played_on.clone(),
        created_at: stats.created_at.clone(),
        slp_path: Some(stats.slp_path.clone()),
    };
    
    database::upsert_game_stats(&conn, &game_stats)
        .map_err(|e| Error::RecordingFailed(format!("Failed to save game stats: {}", e)))?;
    
    log::info!("[SlippiStats] Saved game_stats: stage={}, winner_port={:?}", 
        stats.stage, winner_port);
    
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
            slp_path: Some(stats.slp_path.clone()),
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
    connect_code: Option<String>,
    state: State<'_, AppState>,
) -> Result<AvailableFilterOptions, Error> {
    let db = state.database.clone();
    let conn = db.connection();
    
    database::get_available_filter_options(&conn, connect_code.as_deref())
        .map_err(|e| Error::RecordingFailed(format!("Failed to get filter options: {}", e)))
}

/// List all .slp files in a directory (recursive, up to 5 levels deep)
#[tauri::command]
pub async fn list_slp_files(directory: String) -> Result<Vec<String>, Error> {
    use walkdir::WalkDir;
    
    let dir_path = std::path::Path::new(&directory);
    if !dir_path.exists() {
        return Err(Error::InvalidPath(format!("Directory does not exist: {}", directory)));
    }
    
    let mut slp_files = Vec::new();
    
    for entry in WalkDir::new(&directory)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("slp") {
            slp_files.push(path.to_string_lossy().to_string());
        }
    }
    
    log::info!("Found {} .slp files in {}", slp_files.len(), directory);
    Ok(slp_files)
}

/// Check if a game with the given slp_path already exists in the database
#[tauri::command]
pub async fn check_slp_synced(
    slp_path: String,
    state: State<'_, AppState>,
) -> Result<bool, Error> {
    let db = state.database.clone();
    let conn = db.connection();
    
    database::game_stats_exists_by_slp_path(&conn, &slp_path)
        .map_err(|e| Error::RecordingFailed(format!("Failed to check slp sync status: {}", e)))
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
fn recording_with_stats_to_session(rws: database::RecordingWithStats) -> RecordingSession {
    let row = rws.recording;
    let game_stats = rws.stats;
    let player_stats = rws.player_stats;
    
    recording_row_to_session(row, game_stats, player_stats)
}

/// Convert a database row + optional stats to a RecordingSession
/// Player info is now built from player_stats (source of truth for kill_count, character, etc.)
/// Game stats only provides game-level metadata (stage, duration, etc.)
fn recording_row_to_session(
    row: database::RecordingRow,
    game_stats: Option<database::GameStatsRow>,
    player_stats: Vec<database::PlayerStatsRow>,
) -> RecordingSession {
    // Build SlippiMetadata - players come from player_stats now
    let slippi_metadata = if !player_stats.is_empty() || game_stats.is_some() {
        // Build players array from player_stats (includes kill_count for winner detection)
        let players: Vec<PlayerInfo> = player_stats
            .iter()
            .map(|ps| PlayerInfo {
                character_id: ps.character_id as u8,
                character_color: ps.character_color as u8,
                player_tag: ps.connect_code.clone().unwrap_or_else(|| 
                    ps.display_name.clone().unwrap_or_else(|| format!("P{}", ps.port + 1))
                ),
                port: ps.port as u8,
                kill_count: Some(ps.kill_count),
            })
            .collect();
        
        let characters: Vec<u8> = players.iter().map(|p| p.character_id).collect();
        
        // Get game-level metadata from game_stats
        let (stage, game_duration, total_frames, is_pal, played_on, winner_port) = 
            if let Some(ref gs) = game_stats {
                (
                    gs.stage.unwrap_or(0) as u16,
                    gs.game_duration.unwrap_or(0),
                    gs.total_frames.unwrap_or(0),
                    gs.is_pal.unwrap_or(false),
                    gs.played_on.clone(),
                    gs.winner_port.map(|p| p as u8),
                )
            } else {
                (0, 0, 0, false, None, None)
            };
        
        Some(SlippiMetadata {
            characters,
            stage,
            players,
            game_duration,
            start_time: row.start_time.clone().unwrap_or_default(),
            is_pal,
            winner_port,
            played_on,
            total_frames,
        })
    } else {
        None
    };
    
    // Calculate duration from stats if available
    let duration = game_stats
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
