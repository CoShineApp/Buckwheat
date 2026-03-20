//! Core recording commands

use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::database::{self, RecordingRow, RecordingWithStats, GameStatsRow, PlayerStatsRow};
use crate::slippi::{PlayerInfo, RecordingSession, SlippiMetadata};
use serde::{Deserialize, Serialize};
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
// HELPER FUNCTIONS
// ============================================================================

/// Convert a RecordingWithStats (from paginated query) to RecordingSession
pub(crate) fn recording_with_stats_to_session(rws: RecordingWithStats) -> RecordingSession {
    let row = rws.recording;
    let game_stats = rws.stats;
    let player_stats = rws.player_stats;

    recording_row_to_session(row, game_stats, player_stats)
}

/// Convert a database row + optional stats to a RecordingSession
/// Player info is now built from player_stats (source of truth for kill_count, character, etc.)
/// Game stats only provides game-level metadata (stage, duration, etc.)
pub(crate) fn recording_row_to_session(
    row: RecordingRow,
    game_stats: Option<GameStatsRow>,
    player_stats: Vec<PlayerStatsRow>,
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
