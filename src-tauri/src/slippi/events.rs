// Event extraction from .slp game data

use super::types::{DeathEvent, GameEvent};
use crate::commands::errors::Error;
use peppi::frame::Rollbacks;
use ssbm_data::action_state::Common;

/// Extract death events from a parsed game
///
/// This function iterates through all frames and detects when a player enters
/// a death animation state, recording the frame number and player information.
pub fn extract_death_events(game: &peppi::game::immutable::Game) -> Result<Vec<GameEvent>, Error> {
    log::info!("🔍 Extracting death events from game data");

    let mut events = Vec::new();
    let mut is_dead = vec![false; game.frames.ports.len()];
    let rollbacks = game.frames.rollbacks(Rollbacks::ExceptLast);

    // Get player tags from metadata for event data
    let player_tags: Vec<String> = game
        .start
        .players
        .iter()
        .map(|p| {
            let port = u8::from(p.port);
            game.metadata
                .as_ref()
                .and_then(|m| m.get("players"))
                .and_then(|players| players.as_object())
                .and_then(|players_obj| players_obj.get(&port.to_string()))
                .and_then(|player_data| player_data.get("names"))
                .and_then(|names| names.get("code").or_else(|| names.get("netplay")))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("P{}", port))
        })
        .collect();

    // Iterate through frames looking for death states
    for frame_idx in 0..game.frames.len() {
        // Skip rollback frames
        if rollbacks[frame_idx] {
            continue;
        }

        for (port_idx, port_data) in game.frames.ports.iter().enumerate() {
            // Check if player entered a death state
            let death_detected = match port_data
                .leader
                .post
                .state
                .get(frame_idx)
                .and_then(|s| Common::try_from(s).ok())
            {
                Some(Common::DeadDown)
                | Some(Common::DeadLeft)
                | Some(Common::DeadRight)
                | Some(Common::DeadUp)
                | Some(Common::DeadUpStar)
                | Some(Common::DeadUpStarIce)
                | Some(Common::DeadUpFall)
                | Some(Common::DeadUpFallHitCamera)
                | Some(Common::DeadUpFallHitCameraFlat)
                | Some(Common::DeadUpFallIce)
                | Some(Common::DeadUpFallHitCameraIce) => true,
                _ => false,
            };

            if death_detected && !is_dead[port_idx] {
                is_dead[port_idx] = true;

                let frame = game.frames.id.get(frame_idx).unwrap_or(0);
                let timestamp = frame as f64 / 60.0; // Melee runs at 60fps
                let port = game.start.players[port_idx].port;
                let player_tag = player_tags
                    .get(port_idx)
                    .cloned()
                    .unwrap_or_else(|| format!("P{}", u8::from(port)));

                log::info!(
                    "💀 Death detected: {} on frame {} ({:.2}s)",
                    player_tag,
                    frame,
                    timestamp
                );

                events.push(GameEvent::Death(DeathEvent {
                    frame,
                    timestamp,
                    port: u8::from(port),
                    player_tag,
                }));
            } else if !death_detected {
                is_dead[port_idx] = false;
            }
        }
    }

    log::info!("✅ Extracted {} death events", events.len());
    Ok(events)
}
