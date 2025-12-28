//! Library management for recordings and clips
//!
//! This module handles scanning, matching, and managing video recordings
//! and their associated Slippi replay files.

mod recordings;
mod thumbnails;

pub use recordings::{create_recording_session, get_recording_directory, scan_recordings};

