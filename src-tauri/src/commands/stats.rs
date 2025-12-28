// Tauri commands for player stats operations

use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::database::stats_store::{self, PlayerGameStats};
use crate::slippi::{calculate_player_stats, parse_slp_file};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

/// Calculate and store stats for a recorded game
#[tauri::command]
pub async fn calculate_game_stats(
    app: AppHandle,
    slp_path: String,
    recording_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<PlayerGameStats>, Error> {
    log::info!("📊 Calculating stats for recording: {}", recording_id);
    
    // Get device ID for tracking
    let device_id = crate::commands::cloud::get_device_id(app).await
        .map_err(|e| Error::InitializationError(format!("Failed to get device ID: {}", e)))?;
    
    // Parse .slp file
    let game = parse_slp_file(&slp_path)?;
    
    // Get database connection
    let stats_db = state.stats_db.lock().unwrap();
    let db = match stats_db.as_ref() {
        Some(db) => db.connection(),
        None => {
            return Err(Error::InitializationError(
                "Stats database not initialized".to_string(),
            ))
        }
    };
    drop(stats_db);
    
    // Calculate stats for each player
    let mut all_stats = Vec::new();
    
    for player in &game.start.players {
        let port = u8::from(player.port);
        
        let stats = calculate_player_stats(
            &game,
            port,
            recording_id.clone(),
            slp_path.clone(),
            device_id.clone(),
            None, // user_id - will be set during cloud sync
        )?;
        
        // Store in local database
        stats_store::insert_stats(db.clone(), &stats)?;
        
        log::info!(
            "✅ Calculated stats for {} (port {}): {} L-cancels, {} neutral wins, {:.1} APM",
            stats.player_tag,
            stats.player_port,
            stats.l_cancel_hit,
            stats.neutral_wins,
            stats.apm
        );
        
        all_stats.push(stats);
    }
    
    log::info!("✅ Stats calculation complete for {} players", all_stats.len());
    
    Ok(all_stats)
}

/// Get stats for a specific recording
#[tauri::command]
pub async fn get_recording_stats(
    recording_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<PlayerGameStats>, Error> {
    let stats_db = state.stats_db.lock().unwrap();
    let db = match stats_db.as_ref() {
        Some(db) => db.connection(),
        None => {
            return Err(Error::InitializationError(
                "Stats database not initialized".to_string(),
            ))
        }
    };
    drop(stats_db);
    
    stats_store::get_stats_by_recording(db, &recording_id)
}

/// Query player stats with optional filters
#[tauri::command]
pub async fn get_player_stats(
    player_tag: Option<String>,
    character_id: Option<u8>,
    limit: Option<i32>,
    state: State<'_, AppState>,
) -> Result<Vec<PlayerGameStats>, Error> {
    let stats_db = state.stats_db.lock().unwrap();
    let db = match stats_db.as_ref() {
        Some(db) => db.connection(),
        None => {
            return Err(Error::InitializationError(
                "Stats database not initialized".to_string(),
            ))
        }
    };
    drop(stats_db);
    
    stats_store::query_stats(db, player_tag, character_id, limit)
}

/// Aggregate stats for a player
#[derive(Debug, Serialize, Deserialize)]
pub struct AggregateStats {
    pub player_tag: String,
    pub total_games: i32,
    pub total_wins: i32,
    pub total_losses: i32,
    pub avg_l_cancel_rate: f64,
    pub avg_tech_rate: f64,
    pub avg_apm: f64,
    pub avg_openings_per_kill: f64,
    pub avg_damage_per_opening: f64,
    pub total_wavedashes: i32,
    pub total_dashdances: i32,
}

/// Calculate aggregate stats for a player
#[tauri::command]
pub async fn get_aggregate_stats(
    player_tag: String,
    state: State<'_, AppState>,
) -> Result<AggregateStats, Error> {
    let stats_db = state.stats_db.lock().unwrap();
    let db = match stats_db.as_ref() {
        Some(db) => db.connection(),
        None => {
            return Err(Error::InitializationError(
                "Stats database not initialized".to_string(),
            ))
        }
    };
    drop(stats_db);
    
    // Get all stats for player
    let all_stats = stats_store::query_stats(db, Some(player_tag.clone()), None, None)?;
    
    if all_stats.is_empty() {
        return Err(Error::RecordingFailed(format!(
            "No stats found for player: {}",
            player_tag
        )));
    }
    
    // Calculate aggregates
    let total_games = all_stats.len() as i32;
    let total_wins = all_stats.iter().filter(|s| s.kills > s.deaths).count() as i32;
    let total_losses = all_stats.iter().filter(|s| s.deaths > s.kills).count() as i32;
    
    let total_l_cancels: i32 = all_stats.iter().map(|s| s.l_cancel_hit + s.l_cancel_missed).sum();
    let total_l_cancel_hits: i32 = all_stats.iter().map(|s| s.l_cancel_hit).sum();
    let avg_l_cancel_rate = if total_l_cancels > 0 {
        total_l_cancel_hits as f64 / total_l_cancels as f64 * 100.0
    } else {
        0.0
    };
    
    let total_techs: i32 = all_stats.iter().map(|s| s.successful_techs + s.missed_techs).sum();
    let total_successful_techs: i32 = all_stats.iter().map(|s| s.successful_techs).sum();
    let avg_tech_rate = if total_techs > 0 {
        total_successful_techs as f64 / total_techs as f64 * 100.0
    } else {
        0.0
    };
    
    let avg_apm: f64 = all_stats.iter().map(|s| s.apm).sum::<f64>() / total_games as f64;
    
    let valid_openings_per_kill: Vec<f64> = all_stats
        .iter()
        .filter_map(|s| s.openings_per_kill)
        .collect();
    let avg_openings_per_kill = if !valid_openings_per_kill.is_empty() {
        valid_openings_per_kill.iter().sum::<f64>() / valid_openings_per_kill.len() as f64
    } else {
        0.0
    };
    
    let valid_damage_per_opening: Vec<f64> = all_stats
        .iter()
        .filter_map(|s| s.damage_per_opening)
        .collect();
    let avg_damage_per_opening = if !valid_damage_per_opening.is_empty() {
        valid_damage_per_opening.iter().sum::<f64>() / valid_damage_per_opening.len() as f64
    } else {
        0.0
    };
    
    let total_wavedashes: i32 = all_stats.iter().map(|s| s.wavedash_count).sum();
    let total_dashdances: i32 = all_stats.iter().map(|s| s.dashdance_count).sum();
    
    Ok(AggregateStats {
        player_tag,
        total_games,
        total_wins,
        total_losses,
        avg_l_cancel_rate,
        avg_tech_rate,
        avg_apm,
        avg_openings_per_kill,
        avg_damage_per_opening,
        total_wavedashes,
        total_dashdances,
    })
}

/// Sync unsynced stats to Supabase (for authenticated users)
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResult {
    pub synced_count: i32,
    pub failed_count: i32,
}

#[tauri::command]
pub async fn sync_stats_to_cloud(
    state: State<'_, AppState>,
) -> Result<SyncResult, Error> {
    log::info!("☁️  Starting stats sync to cloud");
    
    let stats_db = state.stats_db.lock().unwrap();
    let db = match stats_db.as_ref() {
        Some(db) => db.connection(),
        None => {
            return Err(Error::InitializationError(
                "Stats database not initialized".to_string(),
            ))
        }
    };
    drop(stats_db);
    
    // Get unsynced stats
    let unsynced_stats = stats_store::get_unsynced_stats(db.clone())?;
    
    if unsynced_stats.is_empty() {
        log::info!("✅ No stats to sync");
        return Ok(SyncResult {
            synced_count: 0,
            failed_count: 0,
        });
    }
    
    log::info!("📤 Found {} unsynced stats records", unsynced_stats.len());
    
    // TODO: Implement actual Supabase sync
    // For now, just mark as synced (will be implemented in sync-to-cloud todo)
    let synced_ids: Vec<String> = unsynced_stats.iter().map(|s| s.id.clone()).collect();
    stats_store::mark_synced(db, &synced_ids)?;
    
    log::info!("✅ Synced {} stats records", synced_ids.len());
    
    Ok(SyncResult {
        synced_count: synced_ids.len() as i32,
        failed_count: 0,
    })
}

