//! Slippi replay file parsing and event extraction module
//!
//! This module handles all .slp file parsing, metadata extraction,
//! and game event detection.

pub mod events;
pub mod metadata;
pub mod parser;
pub mod types;

// Re-export commonly used items
pub use events::extract_death_events;
pub use metadata::{extract_metadata, frames_to_seconds};
pub use parser::parse_slp_file;
pub use types::{GameEvent, RecordingSession, SlippiMetadata};
