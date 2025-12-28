// Neutral win and opening detection

use crate::commands::errors::Error;
use peppi::game::immutable::Game;

#[derive(Debug, Clone, Default)]
pub struct OpeningsStats {
    pub neutral_wins: i32,
    pub neutral_losses: i32,
    pub openings: i32,
    pub total_opening_damage: f64,
    pub total_damage_dealt: f64,
    pub total_damage_taken: f64,
    pub kills: i32,
    pub deaths: i32,
    pub kill_percents: Vec<f64>,
}

/// Calculate opening and neutral stats for a player
pub fn calculate_openings_stats(
    game: &Game,
    player_port: u8,
    rollbacks: &[bool],
) -> Result<OpeningsStats, Error> {
    let port_idx = (player_port - 1) as usize;
    let mut stats = OpeningsStats::default();
    
    // Track opponent(s) - for simplicity, assume 1v1
    let opponent_idx = if port_idx == 0 { 1 } else { 0 };
    
    // Ensure opponent exists
    if opponent_idx >= game.frames.ports.len() {
        return Ok(stats); // Not a 1v1, return empty stats
    }
    
    // Track states for neutral and opening detection
    let mut player_in_hitstun = false;
    let mut opponent_in_hitstun = false;
    let mut current_opening_damage = 0.0;
    let mut in_opening = false;
    
    let mut prev_player_percent = 0.0;
    let mut prev_opponent_percent = 0.0;
    
    for frame_idx in 0..game.frames.len() {
        if rollbacks[frame_idx] {
            continue;
        }
        
        // Access frame data using .get() method
        let player_post = &game.frames.ports[port_idx].leader.post;
        let opponent_post = &game.frames.ports[opponent_idx].leader.post;
        
        let player_state = player_post.state.get(frame_idx).unwrap_or(0);
        let opponent_state = opponent_post.state.get(frame_idx).unwrap_or(0);
        let player_percent = player_post.percent.get(frame_idx).unwrap_or(0.0);
        let opponent_percent = opponent_post.percent.get(frame_idx).unwrap_or(0.0);
        
        // Detect hitstun states
        let player_now_in_hitstun = is_hitstun_state(player_state);
        let opponent_now_in_hitstun = is_hitstun_state(opponent_state);
        
        // Detect neutral wins (first hit in neutral)
        if !player_in_hitstun && opponent_now_in_hitstun && !opponent_in_hitstun {
            // Player won neutral
            stats.neutral_wins += 1;
            
            // Start tracking opening
            in_opening = true;
            current_opening_damage = 0.0;
        } else if !opponent_in_hitstun && player_now_in_hitstun && !player_in_hitstun {
            // Opponent won neutral
            stats.neutral_losses += 1;
            
            // End player's opening if active
            if in_opening {
                stats.openings += 1;
                stats.total_opening_damage += current_opening_damage;
                in_opening = false;
            }
        }
        
        // Track opening damage
        if in_opening && opponent_now_in_hitstun {
            let damage_this_frame = opponent_percent - prev_opponent_percent;
            if damage_this_frame > 0.0 && damage_this_frame < 50.0 {
                // Reasonable damage increase (not a stock loss reset)
                current_opening_damage += damage_this_frame as f64;
            }
        }
        
        // End opening if opponent escapes to neutral
        if in_opening && !opponent_now_in_hitstun && is_neutral_state(opponent_state) {
            stats.openings += 1;
            stats.total_opening_damage += current_opening_damage;
            in_opening = false;
        }
        
        // Track total damage dealt and taken
        let player_damage_taken = player_percent - prev_player_percent;
        let opponent_damage_taken = opponent_percent - prev_opponent_percent;
        
        if player_damage_taken > 0.0 && player_damage_taken < 50.0 {
            stats.total_damage_taken += player_damage_taken as f64;
        }
        
        if opponent_damage_taken > 0.0 && opponent_damage_taken < 50.0 {
            stats.total_damage_dealt += opponent_damage_taken as f64;
        }
        
        // Detect deaths (stocks changing)
        // Deaths happen when damage resets to 0
        if player_percent == 0.0 && prev_player_percent > 10.0 {
            stats.deaths += 1;
        }
        
        if opponent_percent == 0.0 && prev_opponent_percent > 10.0 {
            stats.kills += 1;
            stats.kill_percents.push(prev_opponent_percent as f64);
            
            // End opening on kill
            if in_opening {
                stats.openings += 1;
                stats.total_opening_damage += current_opening_damage;
                in_opening = false;
            }
        }
        
        // Update previous frame tracking
        player_in_hitstun = player_now_in_hitstun;
        opponent_in_hitstun = opponent_now_in_hitstun;
        prev_player_percent = player_percent;
        prev_opponent_percent = opponent_percent;
    }
    
    // Close any remaining opening at end of game
    if in_opening {
        stats.openings += 1;
        stats.total_opening_damage += current_opening_damage;
    }
    
    Ok(stats)
}

/// Check if a state is a hitstun state
fn is_hitstun_state(state: u16) -> bool {
    matches!(
        state,
        26 | 27 | 28 | 29 | 30 | // Damage states
        35 | 36 | 37 | 38 | 39 | // Tumble states
        75 | 76 | 77 | 78 | 79 | // Shield damage states
        91 | 92 | 93 // Grabbed states
    )
}

/// Check if a state is a neutral state (standing, walking, dashing, etc.)
fn is_neutral_state(state: u16) -> bool {
    matches!(
        state,
        0 | 1 | 2 | 3 | 4 | 5 | 6 | // Stand, walk
        10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | // Dash, run
        24 | 25 | 26 | 27 | 28 // Landing
    )
}

