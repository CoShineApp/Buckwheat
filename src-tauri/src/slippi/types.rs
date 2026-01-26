//! Type definitions for Slippi game data
//!
//! These types are used by the API to return data to the frontend.
//! Actual .slp parsing is done in the frontend using slippi-js.

use serde::{Deserialize, Serialize};

// ============================================================================
// GAME METADATA
// ============================================================================

/// Metadata extracted from a Slippi replay file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlippiMetadata {
    /// Character IDs for each player
    pub characters: Vec<u8>,
    /// Stage ID
    pub stage: u16,
    /// Player information
    pub players: Vec<PlayerInfo>,
    /// Game duration in frames
    pub game_duration: i32,
    /// Start time (ISO 8601)
    pub start_time: String,
    /// Whether the game is PAL version
    pub is_pal: bool,
    /// Winner port (0-indexed), if determinable
    pub winner_port: Option<u8>,
    /// Platform the game was played on (e.g., "dolphin", "console")
    pub played_on: Option<String>,
    /// Total number of frames
    pub total_frames: i32,
}

/// Information about a player in the game
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerInfo {
    /// Character ID (internal Melee ID)
    pub character_id: u8,
    /// Character color/costume index
    pub character_color: u8,
    /// Player tag (connect code or netplay name)
    pub player_tag: String,
    /// Controller port (0-indexed)
    pub port: u8,
    /// Number of kills (stocks taken from opponent). Winner has 4 in a standard game.
    pub kill_count: Option<i32>,
}

// ============================================================================
// RECORDING SESSION
// ============================================================================

/// A recording session that links a video file to its Slippi replay
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecordingSession {
    /// Unique identifier (usually filename)
    pub id: String,
    /// Start time (ISO 8601)
    pub start_time: String,
    /// End time (ISO 8601), if available
    pub end_time: Option<String>,
    /// Path to the .slp file
    pub slp_path: String,
    /// Path to the video file
    pub video_path: Option<String>,
    /// Path to the thumbnail image
    pub thumbnail_path: Option<String>,
    /// Duration in seconds
    pub duration: Option<u64>,
    /// File size in bytes
    pub file_size: Option<u64>,
    /// Parsed Slippi metadata
    pub slippi_metadata: Option<SlippiMetadata>,
}
