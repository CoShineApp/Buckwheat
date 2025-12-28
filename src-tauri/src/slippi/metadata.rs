//! Slippi metadata extraction from parsed game data

use super::types::{PlayerInfo, SlippiMetadata};
use peppi::game::immutable::Game;

/// Extract metadata from a parsed Slippi game
pub fn extract_metadata(game: &Game) -> SlippiMetadata {
    let mut characters = Vec::new();
    let mut players = Vec::new();
    
    // Get player codes from metadata JSON
    let player_metadata = game
        .metadata
        .as_ref()
        .and_then(|m| m.get("players"))
        .and_then(|p| p.as_object());
    
    // Get winner from end game data
    let winner_port = game
        .end
        .as_ref()
        .and_then(|end| end.players.as_ref())
        .and_then(|end_players| {
            end_players
                .iter()
                .find(|p| p.placement == 0)
                .map(|p| u8::from(p.port))
        });
    
    // Iterate through players
    for player in &game.start.players {
        let port = u8::from(player.port);
        let char_id = player.character as u8;
        
        characters.push(char_id);
        
        // Get player tag from metadata
        let player_tag = player_metadata
            .and_then(|m| m.get(&port.to_string()))
            .and_then(|p| p.get("names"))
            .and_then(|n| n.get("code").or_else(|| n.get("netplay")))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("P{}", port));
        
        players.push(PlayerInfo {
            character_id: char_id,
            character_color: player.costume,
            player_tag,
            port,
        });
    }
    
    let stage = game.start.stage as u16;
    
    // Get duration from metadata
    let game_duration = game
        .metadata
        .as_ref()
        .and_then(|m| m.get("lastFrame"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    
    // Get start time from metadata
    let start_time = game
        .metadata
        .as_ref()
        .and_then(|m| m.get("startAt"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    
    let is_pal = game.start.is_pal.unwrap_or(false);
    
    // Get additional metadata
    let played_on = game
        .metadata
        .as_ref()
        .and_then(|m| m.get("playedOn"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let total_frames = game.frames.len() as i32;
    
    SlippiMetadata {
        characters,
        stage,
        players,
        game_duration,
        start_time,
        is_pal,
        winner_port,
        played_on,
        total_frames,
    }
}

/// Calculate duration in seconds from frame count
pub fn frames_to_seconds(frames: i32) -> u64 {
    (frames as f64 / 60.0) as u64
}

