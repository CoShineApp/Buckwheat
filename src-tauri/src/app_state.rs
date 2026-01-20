use crate::database::Database;
use crate::game_detector::GameDetector;
use crate::recorder::Recorder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipMarker {
    pub recording_file: String,
    pub timestamp_seconds: f64,
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
    /// SQLite database for persistent metadata cache
    pub database: Arc<Database>,
}

impl AppState {
    /// Create new app state with a database at the specified path
    pub fn with_database(db: Database) -> Self {
        Self {
            game_detector: Mutex::new(None),
            recorder: Mutex::new(None),
            settings: Mutex::new(HashMap::new()),
            last_replay_path: Mutex::new(None),
            current_recording_file: Mutex::new(None),
            last_file_modification: Mutex::new(None),
            clip_markers: Mutex::new(Vec::new()),
            database: Arc::new(db),
        }
    }
}

// Note: AppState requires a database, so it cannot implement Default.
// Use AppState::with_database() to construct it.
