//! Window detection and preview commands
//!
//! DEPRECATED: These commands predate the OBS sidecar integration. OBS uses
//! game_capture with any_fullscreen mode, so manual window selection is no
//! longer needed for recording. These are kept for the sidebar status indicator
//! but should be removed in a future cleanup.

use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::window_detector::{self, GameWindow};
use base64::Engine as _;
use tauri::State;

/// List all potential game windows (Slippi/Dolphin).
///
/// Runs on a blocking thread — enumerating processes and windows is slow,
/// and this used to freeze the UI when called synchronously.
#[tauri::command]
pub async fn list_game_windows() -> Result<Vec<GameWindow>, Error> {
    tokio::task::spawn_blocking(window_detector::find_game_windows)
        .await
        .map_err(|e| Error::RecordingFailed(format!("list_game_windows task failed: {e}")))
}

/// List ALL visible windows (not just Slippi/Dolphin). Used by the capture
/// window picker so users can pick any app to record.
#[tauri::command]
pub async fn list_all_windows() -> Result<Vec<GameWindow>, Error> {
    tokio::task::spawn_blocking(window_detector::find_all_windows)
        .await
        .map_err(|e| Error::RecordingFailed(format!("list_all_windows task failed: {e}")))
}

/// Check if the game window is currently open.
///
/// Reads `gameWindow` from settings.json (the user's picked window) and
/// checks whether a matching process is running with a visible window.
/// Falls back to generic Slippi/Dolphin detection if no window is picked.
#[tauri::command]
pub async fn check_game_window(app: tauri::AppHandle) -> Result<bool, Error> {
    let process_filter = read_game_window_process(&app);

    tokio::task::spawn_blocking(move || match process_filter {
        Some(proc) => window_detector::check_window_by_process(&proc),
        None => window_detector::check_game_window_open(None),
    })
    .await
    .map_err(|e| Error::RecordingFailed(format!("check_game_window task failed: {e}")))
}

/// Read the user's selected game window process name from settings.json.
fn read_game_window_process(app: &tauri::AppHandle) -> Option<String> {
    use tauri::Manager;

    let path = app.path().app_data_dir().ok()?.join("settings.json");
    let contents = std::fs::read_to_string(&path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&contents).ok()?;
    let gw = json.get("gameWindow")?;
    gw.get("processName")?.as_str().map(|s| s.to_string())
}

/// Capture a preview screenshot of a specific window by PID.
/// Returns the PNG as a base64-encoded string so it can be used directly as an
/// `<img src="data:image/png;base64,..." />`.
#[tauri::command]
pub async fn capture_window_by_pid(process_id: u32) -> Result<String, Error> {
    let identifier = format!("(PID: {})", process_id);

    tokio::task::spawn_blocking(move || window_detector::capture_window_preview(&identifier))
        .await
        .map_err(|e| Error::RecordingFailed(format!("capture task failed: {e}")))?
        .map(|bytes| base64::engine::general_purpose::STANDARD.encode(bytes))
        .map_err(|e| Error::RecordingFailed(format!("Failed to capture window: {e}")))
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

