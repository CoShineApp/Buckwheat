use crate::game_detector::GameDetector;
use crate::recorder::Recorder;
use std::sync::Mutex;

/// Global application state managed by Tauri
pub struct AppState {
    pub game_detector: Mutex<Option<GameDetector>>,
    pub recorder: Mutex<Option<Box<dyn Recorder + Send>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            game_detector: Mutex::new(None),
            recorder: Mutex::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

