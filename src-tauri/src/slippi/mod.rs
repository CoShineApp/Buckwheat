//! Slippi type definitions for the API
//!
//! This module contains type definitions used by the API.
//! Actual .slp parsing is done in the frontend using slippi-js.

pub mod types;

// Re-export types used by the API
pub use types::{PlayerInfo, RecordingSession, SlippiMetadata};
