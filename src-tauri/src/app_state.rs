use crate::game_detector::GameDetector;
use crate::recorder::Recorder;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

/// Global application state managed by Tauri
pub struct AppState {
    pub game_detector: Mutex<Option<GameDetector>>,
    pub recorder: Mutex<Option<Box<dyn Recorder + Send>>>,
    pub settings: Mutex<HashMap<String, serde_json::Value>>,
    pub last_replay_path: Mutex<Option<String>>,
    pub current_recording_file: Mutex<Option<String>>,
    pub last_file_modification: Mutex<Option<Instant>>,
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
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
