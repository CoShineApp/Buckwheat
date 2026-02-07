//! Window detection and preview commands
//!
//! Thin command handlers that delegate to the window_detector module.

use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::window_detector::{self, GameWindow};
use base64::Engine as _;
use tauri::State;

/// List all potential game windows (Slippi/Dolphin)
#[tauri::command]
pub fn list_game_windows() -> Result<Vec<GameWindow>, Error> {
    Ok(window_detector::find_game_windows())
}

/// Check if the game window is currently open
#[tauri::command]
pub async fn check_game_window(state: State<'_, AppState>) -> Result<bool, Error> {
    let stored_id = {
        let settings = state
            .settings
            .lock()
            .map_err(|e| Error::InitializationError(format!("Failed to lock settings: {}", e)))?;
        settings
            .get("game_process_name")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    
    Ok(window_detector::check_game_window_open(stored_id.as_deref()))
}

/// Capture a preview screenshot of the selected game window
#[tauri::command]
pub async fn capture_window_preview(state: State<'_, AppState>) -> Result<Option<String>, Error> {
    let identifier = {
        let settings = state
            .settings
            .lock()
            .map_err(|e| Error::InitializationError(format!("Failed to lock settings: {}", e)))?;
        settings
            .get("game_process_name")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    
    let Some(target_id) = identifier else {
        return Ok(None);
    };
    
    match window_detector::capture_window_preview(&target_id) {
        Ok(bytes) => {
            let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
            Ok(Some(encoded))
        }
        Err(err) => {
            log::warn!("Failed to capture window preview: {}", err);
            Ok(None)
        }
    }
}

/// Get the stored game process name
#[tauri::command]
pub async fn get_game_process_name(state: State<'_, AppState>) -> Result<Option<String>, Error> {
    let settings = state
        .settings
        .lock()
        .map_err(|e| Error::InitializationError(format!("Failed to lock settings: {}", e)))?;
    
    Ok(settings
        .get("game_process_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string()))
}

/// Set the game process name for detection and recording
#[tauri::command]
pub async fn set_game_process_name(
    process_name: String,
    state: State<'_, AppState>,
) -> Result<(), Error> {
    log::info!("Setting game process name to: {}", process_name);
    
    let mut settings = state
        .settings
        .lock()
        .map_err(|e| Error::InitializationError(format!("Failed to lock settings: {}", e)))?;
    
    settings.insert(
        "game_process_name".to_string(),
        serde_json::Value::String(process_name),
    );
    
    Ok(())
}

/// Highlight a window by bringing it to foreground and drawing a yellow border
#[tauri::command]
pub async fn highlight_game_window(process_id: u32) -> Result<(), Error> {
    log::info!("Highlighting window with PID: {}", process_id);
    
    // Run in a blocking task since it uses thread::sleep
    tokio::task::spawn_blocking(move || {
        window_detector::highlight_window(process_id)
    })
    .await
    .map_err(|e| Error::RecordingFailed(format!("Task failed: {}", e)))?
    .map_err(|e| Error::RecordingFailed(e))
}

