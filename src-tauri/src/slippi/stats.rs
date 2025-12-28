// Main stats calculation and aggregation module

use crate::commands::errors::Error;
use crate::database::stats_store::PlayerGameStats;
use crate::slippi::openings::calculate_openings_stats;
use crate::slippi::techs::calculate_tech_stats;
use chrono::Utc;
use peppi::frame::Rollbacks;
use peppi::game::immutable::Game;
use uuid::Uuid;

/// Per-player stats extracted from a game
#[derive(Debug, Clone, Default)]
pub struct PlayerStatsRaw {
    // L-Cancel stats
    pub l_cancel_hit: i32,
    pub l_cancel_missed: i32,
    
    // Neutral & opening stats
    pub neutral_wins: i32,
    pub neutral_losses: i32,
    pub openings: i32,
    pub total_opening_damage: f64,
    
    // Kill stats
    pub kills: i32,
    pub deaths: i32,
    pub kill_percents: Vec<f64>,
    pub total_damage_dealt: f64,
    pub total_damage_taken: f64,
    
    // Tech skill stats
    pub successful_techs: i32,
    pub missed_techs: i32,
    pub wavedash_count: i32,
    pub dashdance_count: i32,
    
    // Input stats
    pub total_inputs: i32,
    pub grab_attempts: i32,
    pub grab_success: i32,
}

/// Calculate all stats for a specific player in a game
pub fn calculate_player_stats(
    game: &Game,
    player_port: u8,
    recording_id: String,
    slp_file_path: String,
    device_id: String,
    user_id: Option<String>,
) -> Result<PlayerGameStats, Error> {
    log::info!("📊 Calculating stats for port {}", player_port);
    
    let port_idx = (player_port - 1) as usize;
    
    // Ensure port exists
    if port_idx >= game.start.players.len() {
        return Err(Error::RecordingFailed(format!(
            "Invalid port index: {}",
            port_idx
        )));
    }
    
    let player = &game.start.players[port_idx];
    let rollbacks = game.frames.rollbacks(Rollbacks::ExceptLast);
    
    // Get player tag from metadata
    let player_tag = game
        .metadata
        .as_ref()
        .and_then(|m| m.get("players"))
        .and_then(|players| players.as_object())
        .and_then(|players_obj| players_obj.get(&player_port.to_string()))
        .and_then(|player_data| player_data.get("names"))
        .and_then(|names| names.get("code").or_else(|| names.get("netplay")))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("P{}", player_port));
    
    // Determine opponent character (for 1v1 games)
    let opponent_character_id = if game.start.players.len() == 2 {
        let opponent_idx = if port_idx == 0 { 1 } else { 0 };
        Some(game.start.players[opponent_idx].character as u8)
    } else {
        None
    };
    
    // Initialize raw stats
    let mut stats = PlayerStatsRaw::default();
    
    // Calculate tech stats (L-cancels, techs, wavedashes, dashdances)
    let tech_stats = calculate_tech_stats(game, player_port, &rollbacks)?;
    stats.l_cancel_hit = tech_stats.l_cancel_hit;
    stats.l_cancel_missed = tech_stats.l_cancel_missed;
    stats.successful_techs = tech_stats.successful_techs;
    stats.missed_techs = tech_stats.missed_techs;
    stats.wavedash_count = tech_stats.wavedash_count;
    stats.dashdance_count = tech_stats.dashdance_count;
    
    // Calculate openings and neutral stats
    let openings_stats = calculate_openings_stats(game, player_port, &rollbacks)?;
    stats.neutral_wins = openings_stats.neutral_wins;
    stats.neutral_losses = openings_stats.neutral_losses;
    stats.openings = openings_stats.openings;
    stats.total_opening_damage = openings_stats.total_opening_damage;
    stats.total_damage_dealt = openings_stats.total_damage_dealt;
    stats.total_damage_taken = openings_stats.total_damage_taken;
    stats.kills = openings_stats.kills;
    stats.deaths = openings_stats.deaths;
    stats.kill_percents = openings_stats.kill_percents;
    
    // Calculate APM and input stats
    calculate_input_stats(game, player_port, &rollbacks, &mut stats)?;
    
    // Calculate derived stats
    let damage_per_opening = if stats.openings > 0 {
        Some(stats.total_opening_damage / stats.openings as f64)
    } else {
        None
    };
    
    let openings_per_kill = if stats.kills > 0 {
        Some(stats.openings as f64 / stats.kills as f64)
    } else {
        None
    };
    
    let avg_kill_percent = if !stats.kill_percents.is_empty() {
        let sum: f64 = stats.kill_percents.iter().sum();
        Some(sum / stats.kill_percents.len() as f64)
    } else {
        None
    };
    
    // Calculate APM (actions per minute)
    let game_duration_seconds = game.frames.len() as f64 / 60.0;
    let apm = if game_duration_seconds > 0.0 {
        (stats.total_inputs as f64 / game_duration_seconds) * 60.0
    } else {
        0.0
    };
    
    // Get game date from metadata or use current time
    let game_date = game
        .metadata
        .as_ref()
        .and_then(|m| m.get("startAt"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Utc::now().to_rfc3339());
    
    // Build final stats struct
    Ok(PlayerGameStats {
        id: Uuid::new_v4().to_string(),
        user_id,
        device_id,
        slp_file_path,
        recording_id,
        game_date,
        stage_id: game.start.stage as u16,
        game_duration_frames: game.frames.len() as i32,
        player_port,
        player_tag,
        character_id: player.character as u8,
        opponent_character_id,
        l_cancel_hit: stats.l_cancel_hit,
        l_cancel_missed: stats.l_cancel_missed,
        neutral_wins: stats.neutral_wins,
        neutral_losses: stats.neutral_losses,
        openings: stats.openings,
        damage_per_opening,
        openings_per_kill,
        kills: stats.kills,
        deaths: stats.deaths,
        avg_kill_percent,
        total_damage_dealt: stats.total_damage_dealt,
        total_damage_taken: stats.total_damage_taken,
        successful_techs: stats.successful_techs,
        missed_techs: stats.missed_techs,
        wavedash_count: stats.wavedash_count,
        dashdance_count: stats.dashdance_count,
        apm,
        grab_attempts: stats.grab_attempts,
        grab_success: stats.grab_success,
        synced_to_cloud: false,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    })
}

/// Calculate input stats (APM, grabs)
fn calculate_input_stats(
    game: &Game,
    player_port: u8,
    rollbacks: &[bool],
    stats: &mut PlayerStatsRaw,
) -> Result<(), Error> {
    let port_idx = (player_port - 1) as usize;
    
    let mut total_inputs = 0;
    let mut grab_attempts = 0;
    let mut grab_success = 0;
    let mut prev_buttons = 0u32;
    let mut in_grab_attempt = false;
    
    for frame_idx in 0..game.frames.len() {
        if rollbacks[frame_idx] {
            continue;
        }
        
        // Access frame data using .get() method
        let pre = &game.frames.ports[port_idx].leader.pre;
        let post = &game.frames.ports[port_idx].leader.post;
        
        // Count inputs - button presses and significant joystick movements
        // buttons.get() returns the raw button bitmask (u32)
        let current_buttons = pre.buttons.get(frame_idx).unwrap_or(0);
        
        // Count button presses (new buttons pressed this frame)
        let new_buttons = current_buttons & !prev_buttons;
        let button_count = new_buttons.count_ones();
        total_inputs += button_count as i32;
        
        // Count joystick movements (>0.3 threshold)
        let stick_x = pre.joystick.x.get(frame_idx).unwrap_or(0.0);
        let stick_y = pre.joystick.y.get(frame_idx).unwrap_or(0.0);
        if stick_x.abs() > 0.3 || stick_y.abs() > 0.3 {
            total_inputs += 1;
        }
        
        prev_buttons = current_buttons;
        
        // Detect grab attempts and success
        let action_state = post.state.get(frame_idx).unwrap_or(0);
        
        // Grab attempt detection (action states 212-216 are grabs)
        if action_state >= 212 && action_state <= 216 {
            if !in_grab_attempt {
                grab_attempts += 1;
                in_grab_attempt = true;
            }
        } else {
            in_grab_attempt = false;
        }
        
        // Grab success detection (opponent in grabbed state)
        // We'd need to check opponent's state, simplified for now
        // TODO: Improve grab success detection
    }
    
    stats.total_inputs = total_inputs;
    stats.grab_attempts = grab_attempts;
    stats.grab_success = grab_success;
    
    Ok(())
}

