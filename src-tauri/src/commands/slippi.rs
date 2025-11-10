use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::game_detector::{slippi_paths, GameDetector};
use crate::recorder;
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordingSession {
    pub id: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub slp_path: String,
    pub video_path: Option<String>,
    pub duration: Option<u64>,
}

/// Get the default Slippi replay folder path for the current OS
#[tauri::command]
pub fn get_default_slippi_path() -> Result<String, Error> {
    let path = slippi_paths::get_default_slippi_path();

    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| Error::InvalidPath("Failed to convert path to string".to_string()))
}

/// Start watching for new Slippi games
#[tauri::command]
pub async fn start_watching(path: String, state: State<'_, AppState>) -> Result<(), Error> {
    log::info!("📁 Starting to watch Slippi folder: {}", path);
    
    let slippi_path = PathBuf::from(&path);
    
    // Check if path exists
    if !slippi_path.exists() {
        return Err(Error::InvalidPath(format!(
            "Slippi folder does not exist: {}",
            path
        )));
    }
    
    // Create new GameDetector
    let mut detector = GameDetector::new(slippi_path);
    detector.start_watching()?;
    
    // Store in app state
    let mut game_detector = state.game_detector.lock().map_err(|e| {
        Error::InitializationError(format!("Failed to lock game detector: {}", e))
    })?;
    *game_detector = Some(detector);
    
    log::info!("✅ Now watching for .slp files");
    Ok(())
}

/// Stop watching for new games
#[tauri::command]
pub async fn stop_watching(state: State<'_, AppState>) -> Result<(), Error> {
    log::info!("⏹️  Stopping file watcher");
    
    let mut game_detector = state.game_detector.lock().map_err(|e| {
        Error::InitializationError(format!("Failed to lock game detector: {}", e))
    })?;
    
    if let Some(detector) = game_detector.as_mut() {
        detector.stop_watching();
    }
    
    *game_detector = None;
    log::info!("✅ File watcher stopped");
    Ok(())
}

/// Start recording gameplay
#[tauri::command]
pub async fn start_recording(output_path: String, state: State<'_, AppState>) -> Result<(), Error> {
    log::info!("🎥 Starting recording to: {}", output_path);
    
    // Get or create recorder
    let mut recorder_lock = state.recorder.lock().map_err(|e| {
        Error::InitializationError(format!("Failed to lock recorder: {}", e))
    })?;
    
    // Create new recorder if none exists
    if recorder_lock.is_none() {
        *recorder_lock = Some(recorder::get_recorder());
    }
    
    // Start recording
    if let Some(recorder) = recorder_lock.as_mut() {
        recorder.start_recording(&output_path)?;
        log::info!("✅ Recording started successfully");
        Ok(())
    } else {
        Err(Error::InitializationError(
            "Failed to initialize recorder".to_string(),
        ))
    }
}

/// Stop recording gameplay
#[tauri::command]
pub async fn stop_recording(state: State<'_, AppState>) -> Result<String, Error> {
    log::info!("⏹️  Stopping recording");
    
    let mut recorder_lock = state.recorder.lock().map_err(|e| {
        Error::RecordingFailed(format!("Failed to lock recorder: {}", e))
    })?;
    
    if let Some(recorder) = recorder_lock.as_mut() {
        let output_path = recorder.stop_recording()?;
        log::info!("✅ Recording stopped: {}", output_path);
        
        // Clean up recorder
        *recorder_lock = None;
        
        Ok(output_path)
    } else {
        Err(Error::RecordingFailed(
            "No active recording to stop".to_string(),
        ))
    }
}

/// Get list of recorded sessions
#[tauri::command]
pub fn get_recordings() -> Result<Vec<RecordingSession>, Error> {
    let mock_recordings = vec![
        RecordingSession {
            id: "1".to_string(),
            start_time: "2025-11-06T12:00:00Z".to_string(),
            end_time: Some("2025-11-06T12:05:00Z".to_string()),
            slp_path: "/path/to/game1.slp".to_string(),
            video_path: Some("/path/to/game1.mp4".to_string()),
            duration: Some(300),
        },
        RecordingSession {
            id: "2".to_string(),
            start_time: "2025-11-06T13:00:00Z".to_string(),
            end_time: Some("2025-11-06T13:03:30Z".to_string()),
            slp_path: "/path/to/game2.slp".to_string(),
            video_path: Some("/path/to/game2.mp4".to_string()),
            duration: Some(210),
        },
    ];

    Ok(mock_recordings)
}

