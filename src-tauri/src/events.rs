//! Type-safe Tauri event names and payloads
//!
//! This module centralizes all event names used for communication between
//! the Rust backend and the frontend, preventing typos and making it easier
//! to track game state changes.

use serde::{Deserialize, Serialize};

/// Events emitted by the game detector when Slippi file changes are detected
pub mod game {
    /// Emitted when a new .slp file is created (game started)
    pub const FILE_CREATED: &str = "slp-file-created";

    /// Emitted when an existing .slp file is modified (game ended - Slippi writes all data at once)
    pub const FILE_MODIFIED: &str = "slp-file-modified";

    /// Emitted when the last replay path is updated
    pub const LAST_REPLAY_UPDATED: &str = "last-replay-updated";
}

/// Events emitted during the recording lifecycle
pub mod recording {
    /// Emitted when recording starts (includes output path)
    pub const STARTED: &str = "recording-started";

    /// Emitted when recording stops (includes output path)
    pub const STOPPED: &str = "recording-stopped";
}

/// Events emitted during clip processing
pub mod clips {
    /// Emitted when clips have been created (includes list of clip paths)
    pub const CREATED: &str = "clips-created";
}

/// Represents the current state of a Slippi game session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    /// No game is currently active
    Idle,
    /// A game is in progress (file was created but not yet finalized)
    InProgress,
    /// Game has ended (file was modified/finalized)
    Ended,
}

impl Default for GameState {
    fn default() -> Self {
        Self::Idle
    }
}

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameState::Idle => write!(f, "Idle"),
            GameState::InProgress => write!(f, "In Progress"),
            GameState::Ended => write!(f, "Ended"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_names_are_consistent() {
        // Ensure event names follow the expected format
        assert!(game::FILE_CREATED.contains("slp"));
        assert!(game::FILE_MODIFIED.contains("slp"));
    }

    #[test]
    fn test_game_state_default() {
        assert_eq!(GameState::default(), GameState::Idle);
    }
}

