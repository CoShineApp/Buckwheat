//! Library management for recordings and clips
//!
//! This module handles scanning, matching, and managing video recordings
//! and their associated Slippi replay files.

mod recordings;
mod sync;
mod thumbnails;

pub use recordings::get_recording_directory;
pub use sync::sync_recordings_cache;

