use crate::game_detector::GameDetector;
use crate::recorder::Recorder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Instant, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipMarker {
    pub recording_file: String,
    pub timestamp_seconds: f64,
}

#[derive(Clone)]
pub struct SlpCacheEntry {
    pub metadata: serde_json::Value,
    pub duration: Option<u64>,
    pub end_time: Option<String>,
    pub modified_time: SystemTime,
}

/// Global application state managed by Tauri
pub struct AppState {
    pub game_detector: Mutex<Option<GameDetector>>,
    pub recorder: Mutex<Option<Box<dyn Recorder + Send>>>,
    pub settings: Mutex<HashMap<String, serde_json::Value>>,
    pub last_replay_path: Mutex<Option<String>>,
    pub current_recording_file: Mutex<Option<String>>,
    pub last_file_modification: Mutex<Option<Instant>>,
    pub clip_markers: Mutex<Vec<ClipMarker>>,
    pub slp_cache: Mutex<HashMap<String, SlpCacheEntry>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            game_detector: Mutex::new(None),
            recorder: Mutex::new(None),
            settings: Mutex::new(HashMap::new()),
            last_replay_path: Mutex::new(None),
            current_recording_file: Mutex::new(None),
            last_file_modification: Mutex::new(None),
            clip_markers: Mutex::new(Vec::new()),
            slp_cache: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
