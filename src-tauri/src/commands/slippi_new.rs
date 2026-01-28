//! Slippi-specific commands
//!
//! Commands for watching .slp files, parsing replays, and Slippi-related functionality.

use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::commands::recording::{configure_target_window, resolve_recording_quality, start_recording_with_quality};
use crate::events::{game as game_events, recording as recording_events};
use crate::game_detector::{slippi_paths, GameDetector};
use crate::library;
use std::path::PathBuf;
use tauri::{Emitter, Listener, Manager, State};

/// Get the default Slippi replay folder path for the current OS
#[tauri::command]
pub fn get_default_slippi_path() -> Result<String, Error> {
    let path = slippi_paths::get_default_slippi_path();
    
    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| Error::InvalidPath("Failed to convert path to string".to_string()))
}

/// Get the last detected replay file path
#[tauri::command]
pub fn get_last_replay_path(state: State<'_, AppState>) -> Option<String> {
    state
        .last_replay_path
        .lock()
        .ok()
        .and_then(|path| path.clone())
}

/// Start watching for new Slippi games
#[tauri::command]
pub async fn start_watching(
    path: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), Error> {
    let slippi_path = PathBuf::from(&path);
    
    // Check if path exists
    if !slippi_path.exists() {
        log::error!("Path does not exist: {}", path);
        return Err(Error::InvalidPath(format!("Slippi folder does not exist: {}", path)));
    }
    
    // Create new GameDetector with app handle
    let mut detector = GameDetector::new(slippi_path);
    detector.set_app_handle(app.clone());
    detector.start_watching()?;
    
    // Store in app state
    let mut game_detector = state
        .game_detector
        .lock()
        .map_err(|e| Error::InitializationError(format!("Failed to lock game detector: {}", e)))?;
    *game_detector = Some(detector);
    
    // Set up event listener for game start (auto-recording)
    let app_clone = app.clone();
    log::info!("Setting up event listener for '{}' events", game_events::FILE_CREATED);
    
    let app_clone2 = app.clone();
    app.listen(game_events::FILE_CREATED, move |event| {
        let slp_path: &str = event.payload();
        log::info!("========================================");
        log::info!("Received {} event!", game_events::FILE_CREATED);
        log::info!("Payload: {}", slp_path);
        log::info!("========================================");
        
        let app_handle = app_clone.clone();
        let state_ref = app_handle.state::<AppState>();
        
        // Store the last replay path
        if let Ok(mut last_replay) = state_ref.last_replay_path.lock() {
            *last_replay = Some(slp_path.to_string());
            log::info!("Last replay path stored: {}", slp_path);
            
            // Emit event to frontend
            if let Err(e) = app_handle.emit(game_events::LAST_REPLAY_UPDATED, slp_path) {
                log::error!("Failed to emit {} event: {:?}", game_events::LAST_REPLAY_UPDATED, e);
            }
        }
        
        // Check if auto-start recording is enabled
        if let Ok(settings) = state_ref.settings.lock() {
            let auto_start = settings
                .get("autoStartRecording")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            
            if !auto_start {
                log::info!("Auto-start recording is disabled");
                return;
            }
        }
        
        // Check if already recording
        if let Ok(recorder_lock) = state_ref.recorder.lock() {
            if recorder_lock.is_some() {
                log::info!("Already recording, skipping");
                return;
            }
        }
        
        // Track the file for game end detection
        let slp_path_clean = slp_path.trim_matches('"');
        if let Ok(mut current_file) = state_ref.current_recording_file.lock() {
            *current_file = Some(slp_path_clean.to_string());
            log::info!("Tracking recording file for game end detection: {}", slp_path_clean);
        }
        
        let slp_path_for_recording = slp_path_clean.to_string();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = trigger_auto_recording(app_handle, slp_path_for_recording).await {
                log::error!("Failed to trigger auto-recording: {:?}", e);
            }
        });
    });
    
    // Set up event listener for game end (stop recording)
    log::info!("Setting up event listener for '{}' events", game_events::FILE_MODIFIED);
    let app_clone2_inner = app_clone2.clone();
    app_clone2.listen(game_events::FILE_MODIFIED, move |event| {
        let modified_path = event.payload();
        log::info!("File modified - game likely ended: {}", modified_path);
        
        let state_ref = app_clone2_inner.state::<AppState>();
        
        // Check if this is the file we're currently recording
        if let Ok(current_file) = state_ref.current_recording_file.lock() {
            if let Some(recording_file) = current_file.as_ref() {
                let modified_path_clean = modified_path.trim_matches('"');
                
                // Compare by base filename
                let stored_base = std::path::Path::new(recording_file)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                let modified_base = std::path::Path::new(modified_path_clean)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                
                log::info!("Comparing base filenames: stored='{}' modified='{}'", stored_base, modified_base);
                
                if stored_base == modified_base && !stored_base.is_empty() {
                    log::info!("Detected modification of recording file - game ended!");
                    drop(current_file);
                    
                    // Wait for file write to complete, then stop recording
                    let app_handle = app_clone2_inner.clone();
                    tauri::async_runtime::spawn(async move {
                        log::info!("Waiting 3 seconds for file write to complete...");
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                        
                        log::info!("Stopping recording after game end...");
                        if let Err(e) = stop_recording_internal(&app_handle).await {
                            log::error!("Failed to stop recording: {:?}", e);
                        }
                    });
                }
            }
        }
    });
    
    log::info!("Now watching for .slp files");
    Ok(())
}

/// Stop watching for new games
#[tauri::command]
pub async fn stop_watching(state: State<'_, AppState>) -> Result<(), Error> {
    let mut game_detector = state
        .game_detector
        .lock()
        .map_err(|e| Error::InitializationError(format!("Failed to lock game detector: {}", e)))?;
    
    if let Some(detector) = game_detector.as_mut() {
        detector.stop_watching();
    }
    
    *game_detector = None;
    Ok(())
}

// ============================================================================
// INTERNAL HELPERS
// ============================================================================

async fn stop_recording_internal(app: &tauri::AppHandle) -> Result<(), Error> {
    let state = app.state::<AppState>();
    
    let mut recorder_lock = state
        .recorder
        .lock()
        .map_err(|e| Error::RecordingFailed(format!("Failed to lock recorder: {}", e)))?;
    
    if let Some(recorder) = recorder_lock.as_mut() {
        let output_path = recorder.stop_recording()?;
        log::info!("Auto-stopped recording: {}", output_path);
        
        *recorder_lock = None;
        drop(recorder_lock);
        
        // Log clip markers
        let associated_recording = {
            let mut current_file = state.current_recording_file.lock().map_err(|e| {
                Error::InitializationError(format!("Failed to lock current file: {}", e))
            })?;
            current_file.take()
        };
        
        if let Some(ref identifier) = associated_recording {
            let marker_snapshot = {
                let markers = state.clip_markers.lock().map_err(|e| {
                    Error::InitializationError(format!("Failed to lock clip markers: {}", e))
                })?;
                markers
                    .iter()
                    .filter(|m| &m.recording_file == identifier)
                    .map(|m| m.timestamp_seconds)
                    .collect::<Vec<_>>()
            };
            
            if marker_snapshot.is_empty() {
                log::info!("No clip markers queued for {}", identifier);
            } else {
                log::info!("Clip markers for {}: {:?}", identifier, marker_snapshot);
            }
        }
        
        if let Ok(mut last_mod) = state.last_file_modification.lock() {
            *last_mod = None;
        }
        
        // Emit event to frontend
        if let Err(e) = app.emit(recording_events::STOPPED, output_path) {
            log::error!("Failed to emit {} event: {:?}", recording_events::STOPPED, e);
        }
        
        Ok(())
    } else {
        Err(Error::RecordingFailed("No active recording".to_string()))
    }
}

async fn trigger_auto_recording(app: tauri::AppHandle, slp_path: String) -> Result<(), Error> {
    log::info!("Triggering auto-recording for: {}", slp_path);
    
    let state = app.state::<AppState>();
    
    // Get recording directory
    let recording_dir = library::get_recording_directory(&app).await?;
    
    // Generate output path matching the .slp filename
    let slp_filename = std::path::Path::new(&slp_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("recording");
    
    let output_path = format!("{}/{}.mp4", recording_dir, slp_filename);
    log::info!("Output path: {}", output_path);
    
    // Get recording quality
    let quality = resolve_recording_quality(&state)?;
    let resolution_info = quality
        .target_resolution()
        .map(|(w, h)| format!("{}x{}", w, h))
        .unwrap_or_else(|| "native".to_string());
    log::info!(
        "📊 Auto-recording quality: {:?} ({}p, {} Mbps)",
        quality,
        resolution_info,
        quality.bitrate() / 1_000_000
    );
    
    configure_target_window(&state);
    start_recording_with_quality(&state, &output_path, quality)?;
    
    // Track the video output path
    if let Ok(mut current_file) = state.current_recording_file.lock() {
        *current_file = Some(output_path.clone());
    }
    
    // Emit event to frontend
    if let Err(e) = app.emit(recording_events::STARTED, output_path.clone()) {
        log::error!("Failed to emit {} event: {:?}", recording_events::STARTED, e);
    }
    
    Ok(())
}

