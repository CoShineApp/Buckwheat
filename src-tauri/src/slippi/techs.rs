// Tech skill detection: L-cancels, techs, wavedashes, dashdances

use crate::commands::errors::Error;
use peppi::game::immutable::Game;

#[derive(Debug, Clone, Default)]
pub struct TechStats {
    pub l_cancel_hit: i32,
    pub l_cancel_missed: i32,
    pub successful_techs: i32,
    pub missed_techs: i32,
    pub wavedash_count: i32,
    pub dashdance_count: i32,
}

/// Calculate tech skill stats for a player
pub fn calculate_tech_stats(
    game: &Game,
    player_port: u8,
    rollbacks: &[bool],
) -> Result<TechStats, Error> {
    let port_idx = (player_port - 1) as usize;
    let mut stats = TechStats::default();
    
    let mut prev_state: Option<u16> = None;
    let mut dash_direction_changes = 0;
    let mut prev_dash_frame = 0usize;
    let mut prev_direction = 0.0f32;
    
    for frame_idx in 0..game.frames.len() {
        if rollbacks[frame_idx] {
            continue;
        }
        
        // Access frame data using .get() method
        let post = &game.frames.ports[port_idx].leader.post;
        let pre = &game.frames.ports[port_idx].leader.pre;
        
        let current_state = post.state.get(frame_idx).unwrap_or(0);
        
        // L-cancel detection - pass frame index to function
        detect_l_cancel(current_state, prev_state, pre, frame_idx, &mut stats);
        
        // Tech detection
        detect_tech(current_state, prev_state, &mut stats);
        
        // Wavedash detection - pass frame index to function
        detect_wavedash(current_state, prev_state, pre, frame_idx, &mut stats);
        
        // Dashdance detection
        let stick_x = pre.joystick.x.get(frame_idx).unwrap_or(0.0);
        detect_dashdance(
            current_state,
            stick_x,
            prev_direction,
            frame_idx,
            prev_dash_frame,
            &mut dash_direction_changes,
            &mut stats,
        );
        
        // Update previous state tracking
        prev_state = Some(current_state);
        
        // Update dashdance tracking
        if current_state == 20 || current_state == 21 {
            // Dash or Run
            let stick_x = pre.joystick.x.get(frame_idx).unwrap_or(0.0);
            if stick_x.abs() > 0.5 {
                let current_direction = stick_x.signum();
                if current_direction != prev_direction && prev_direction != 0.0 {
                    dash_direction_changes += 1;
                    prev_dash_frame = frame_idx;
                }
                prev_direction = current_direction;
            }
        }
    }
    
    Ok(stats)
}

/// Detect L-cancel success or failure
fn detect_l_cancel(
    current_state: u16,
    prev_state: Option<u16>,
    pre: &peppi::frame::immutable::Pre,
    frame_idx: usize,
    stats: &mut TechStats,
) {
    // L-cancel happens when landing from an aerial attack
    // Landing lag is reduced by half if L/R was pressed within 7 frames before landing
    
    // Aerial attack states (approximate)
    let is_aerial_attack = matches!(
        current_state,
        44..=63 // Aerial attacks range
    );
    
    // Landing states
    let is_landing = matches!(
        current_state,
        24 | 25 | 26 | 27 | 28 // Various landing states
    );
    
    // Check if transitioning from aerial to landing
    if let Some(prev) = prev_state {
        if matches!(prev, 44..=63) && is_landing {
            // TODO: Properly detect L-cancel by checking trigger state
            // For now, we'll use a simplified heuristic based on landing lag
            // This is a placeholder - proper implementation requires accessing trigger data correctly
            stats.l_cancel_missed += 1;
        }
    }
}

/// Detect successful tech vs missed tech
fn detect_tech(
    current_state: u16,
    prev_state: Option<u16>,
    stats: &mut TechStats,
) {
    // Tech states: tech in place (197), tech roll left (198), tech roll right (199), wall tech (various)
    let successful_tech_states = [197, 198, 199, 200, 201];
    
    // Missed tech state (183 = down/lying down)
    let missed_tech_state = 183;
    
    if let Some(prev) = prev_state {
        // Check if just entered a successful tech state
        if successful_tech_states.contains(&current_state) && !successful_tech_states.contains(&prev) {
            stats.successful_techs += 1;
        }
        
        // Check if just entered missed tech state (lying down)
        if current_state == missed_tech_state && prev != missed_tech_state {
            // Make sure we're entering from a tumble/hitstun state
            if matches!(prev, 26 | 27 | 28 | 29 | 30) {
                stats.missed_techs += 1;
            }
        }
    }
}

/// Detect wavedash (airdodge + landing within a few frames)
fn detect_wavedash(
    current_state: u16,
    prev_state: Option<u16>,
    pre: &peppi::frame::immutable::Pre,
    frame_idx: usize,
    stats: &mut TechStats,
) {
    // Wavedash = airdodge (state 236) + landing quickly + diagonal angle
    
    if let Some(prev) = prev_state {
        // Check if we just landed from an airdodge
        if prev == 236 && matches!(current_state, 24 | 25 | 26 | 27 | 28) {
            // Check if the airdodge was at a wavedash angle (diagonal, not straight down)
            let stick_x = pre.joystick.x.get(frame_idx).unwrap_or(0.0).abs();
            let stick_y = pre.joystick.y.get(frame_idx).unwrap_or(0.0);
            
            // Wavedash requires significant horizontal input and downward angle
            if stick_x > 0.5 && stick_y < -0.3 {
                stats.wavedash_count += 1;
            }
        }
    }
}

/// Detect dashdance (rapid direction changes while dashing)
fn detect_dashdance(
    current_state: u16,
    _stick_x: f32,
    _prev_direction: f32,
    frame_idx: usize,
    prev_dash_frame: usize,
    dash_direction_changes: &mut i32,
    stats: &mut TechStats,
) {
    // Dashdance = at least 2 direction changes within a short window
    
    // If we're in dash/run state and have changed direction
    if (current_state == 20 || current_state == 21) && *dash_direction_changes >= 2 {
        // Check if the direction changes were quick (within 30 frames)
        if frame_idx - prev_dash_frame < 30 {
            stats.dashdance_count += 1;
            *dash_direction_changes = 0; // Reset for next dashdance
        }
    }
    
    // Reset if we exit dash state
    if current_state != 20 && current_state != 21 {
        *dash_direction_changes = 0;
    }
}

