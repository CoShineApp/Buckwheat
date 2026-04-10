//! Stats commands

use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::database::{self, AggregatedPlayerStats, AvailableFilterOptions, StatsFilter, TimeSeriesDataPoint};
use serde::{Deserialize, Serialize};
use tauri::State;

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

    // L-Cancel stats (totals)
    pub l_cancel_success_count: i32,
    pub l_cancel_fail_count: i32,

    // L-Cancel detailed breakdown: per aerial x target x outcome
    // Nair
    pub l_cancel_nair_shield_success: i32,
    pub l_cancel_nair_shield_fail: i32,
    pub l_cancel_nair_whiff_success: i32,
    pub l_cancel_nair_whiff_fail: i32,
    pub l_cancel_nair_hit_success: i32,
    pub l_cancel_nair_hit_fail: i32,
    // Fair
    pub l_cancel_fair_shield_success: i32,
    pub l_cancel_fair_shield_fail: i32,
    pub l_cancel_fair_whiff_success: i32,
    pub l_cancel_fair_whiff_fail: i32,
    pub l_cancel_fair_hit_success: i32,
    pub l_cancel_fair_hit_fail: i32,
    // Bair
    pub l_cancel_bair_shield_success: i32,
    pub l_cancel_bair_shield_fail: i32,
    pub l_cancel_bair_whiff_success: i32,
    pub l_cancel_bair_whiff_fail: i32,
    pub l_cancel_bair_hit_success: i32,
    pub l_cancel_bair_hit_fail: i32,
    // Uair
    pub l_cancel_uair_shield_success: i32,
    pub l_cancel_uair_shield_fail: i32,
    pub l_cancel_uair_whiff_success: i32,
    pub l_cancel_uair_whiff_fail: i32,
    pub l_cancel_uair_hit_success: i32,
    pub l_cancel_uair_hit_fail: i32,
    // Dair
    pub l_cancel_dair_shield_success: i32,
    pub l_cancel_dair_shield_fail: i32,
    pub l_cancel_dair_whiff_success: i32,
    pub l_cancel_dair_whiff_fail: i32,
    pub l_cancel_dair_hit_success: i32,
    pub l_cancel_dair_hit_fail: i32,

    // Shield grab (placeholder for now)
    pub shield_grab_count: i32,

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
        match_id: stats.match_id.clone(),
        game_number: stats.game_number,
        game_end_method: stats.game_end_method.clone(),
        slp_path: Some(stats.slp_path.clone()),
        notes: None, // Preserve existing notes on update; user edits via set_game_notes
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
            // L-Cancel detailed breakdown
            l_cancel_nair_shield_success: player.l_cancel_nair_shield_success,
            l_cancel_nair_shield_fail: player.l_cancel_nair_shield_fail,
            l_cancel_nair_whiff_success: player.l_cancel_nair_whiff_success,
            l_cancel_nair_whiff_fail: player.l_cancel_nair_whiff_fail,
            l_cancel_nair_hit_success: player.l_cancel_nair_hit_success,
            l_cancel_nair_hit_fail: player.l_cancel_nair_hit_fail,
            l_cancel_fair_shield_success: player.l_cancel_fair_shield_success,
            l_cancel_fair_shield_fail: player.l_cancel_fair_shield_fail,
            l_cancel_fair_whiff_success: player.l_cancel_fair_whiff_success,
            l_cancel_fair_whiff_fail: player.l_cancel_fair_whiff_fail,
            l_cancel_fair_hit_success: player.l_cancel_fair_hit_success,
            l_cancel_fair_hit_fail: player.l_cancel_fair_hit_fail,
            l_cancel_bair_shield_success: player.l_cancel_bair_shield_success,
            l_cancel_bair_shield_fail: player.l_cancel_bair_shield_fail,
            l_cancel_bair_whiff_success: player.l_cancel_bair_whiff_success,
            l_cancel_bair_whiff_fail: player.l_cancel_bair_whiff_fail,
            l_cancel_bair_hit_success: player.l_cancel_bair_hit_success,
            l_cancel_bair_hit_fail: player.l_cancel_bair_hit_fail,
            l_cancel_uair_shield_success: player.l_cancel_uair_shield_success,
            l_cancel_uair_shield_fail: player.l_cancel_uair_shield_fail,
            l_cancel_uair_whiff_success: player.l_cancel_uair_whiff_success,
            l_cancel_uair_whiff_fail: player.l_cancel_uair_whiff_fail,
            l_cancel_uair_hit_success: player.l_cancel_uair_hit_success,
            l_cancel_uair_hit_fail: player.l_cancel_uair_hit_fail,
            l_cancel_dair_shield_success: player.l_cancel_dair_shield_success,
            l_cancel_dair_shield_fail: player.l_cancel_dair_shield_fail,
            l_cancel_dair_whiff_success: player.l_cancel_dair_whiff_success,
            l_cancel_dair_whiff_fail: player.l_cancel_dair_whiff_fail,
            l_cancel_dair_hit_success: player.l_cancel_dair_hit_success,
            l_cancel_dair_hit_fail: player.l_cancel_dair_hit_fail,
            shield_grab_count: player.shield_grab_count,
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

/// Get game notes for a recording/game by id
#[tauri::command]
pub async fn get_game_notes(
    recording_id: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, Error> {
    let db = state.database.clone();
    let conn = db.connection();
    database::get_game_notes(&conn, &recording_id)
        .map_err(|e| Error::RecordingFailed(format!("Failed to get game notes: {}", e)))
}

/// Set game notes for a recording/game (creates game_stats row if needed)
#[tauri::command]
pub async fn set_game_notes(
    recording_id: String,
    notes: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), Error> {
    let db = state.database.clone();
    let conn = db.connection();
    database::set_game_notes(&conn, &recording_id, notes.as_deref())
        .map_err(|e| Error::RecordingFailed(format!("Failed to set game notes: {}", e)))
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

/// Get per-game stats time series for chart visualization
#[tauri::command]
pub async fn get_player_stats_timeseries(
    connect_code: String,
    filter: Option<StatsFilter>,
    state: State<'_, AppState>,
) -> Result<Vec<TimeSeriesDataPoint>, Error> {
    log::debug!(
        "Getting time series stats for {} with filter: {:?}",
        connect_code,
        filter
    );

    let db = state.database.clone();
    let conn = db.connection();

    database::get_player_stats_timeseries(&conn, &connect_code, filter)
        .map_err(|e| Error::RecordingFailed(format!("Failed to get time series stats: {}", e)))
}
