//! Recording lifecycle commands
//!
//! Commands for starting, stopping, and managing video recordings.

use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::events::recording as recording_events;
use crate::library;
use crate::recorder::{self, RecordingQuality};
use std::path::Path;
use tauri::{Emitter, State};

/// Start recording with a specific output path
#[tauri::command]
pub async fn start_recording(output_path: String, state: State<'_, AppState>) -> Result<(), Error> {
    let quality = resolve_recording_quality(&state)?;
    log_quality_info(&quality);
    
    configure_target_window(&state);
    start_recording_with_quality(&state, &output_path, quality)?;
    Ok(())
}

/// Start a generic/manual recording with an auto-generated filename
#[tauri::command]
pub async fn start_generic_recording(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, Error> {
    let recording_dir = library::get_recording_directory(&app).await?;
    let output_path = generate_generic_recording_path(&recording_dir);
    
    let quality = resolve_recording_quality(&state)?;
    log_quality_info(&quality);
    
    configure_target_window(&state);
    start_recording_with_quality(&state, &output_path, quality)?;
    
    if let Ok(mut current_file) = state.current_recording_file.lock() {
        *current_file = Some(output_path.clone());
    }
    
    Ok(output_path)
}

/// Stop the current recording
#[tauri::command]
pub async fn stop_recording(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, Error> {
    let mut recorder_lock = state
        .recorder
        .lock()
        .map_err(|e| Error::RecordingFailed(format!("Failed to lock recorder: {}", e)))?;
    
    if let Some(recorder) = recorder_lock.as_mut() {
        let output_path = recorder.stop_recording()?;
        
        // Clean up recorder
        *recorder_lock = None;
        
        // Log any clip markers
        let marker_snapshot = {
            let markers = state.clip_markers.lock().map_err(|e| {
                Error::InitializationError(format!("Failed to lock clip markers: {}", e))
            })?;
            markers
                .iter()
                .filter(|m| m.recording_file == output_path)
                .map(|m| m.timestamp_seconds)
                .collect::<Vec<_>>()
        };
        
        if marker_snapshot.is_empty() {
            log::info!("No clip markers queued for {}", output_path);
        } else {
            log::info!("Clip markers for {}: {:?}", output_path, marker_snapshot);
        }
        
        if let Err(e) = app.emit(recording_events::STOPPED, output_path.clone()) {
            log::error!("Failed to emit {} event: {:?}", recording_events::STOPPED, e);
        }
        
        if let Ok(mut current_file) = state.current_recording_file.lock() {
            if current_file.as_ref().map(|s| s == &output_path).unwrap_or(false) {
                *current_file = None;
            }
        }
        
        Ok(output_path)
    } else {
        Err(Error::RecordingFailed("No active recording to stop".to_string()))
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

pub(crate) fn resolve_recording_quality(state: &State<'_, AppState>) -> Result<RecordingQuality, Error> {
    let settings = state
        .settings
        .lock()
        .map_err(|e| Error::InitializationError(format!("Failed to lock settings: {}", e)))?;
    
    let quality_str = settings
        .get("recordingQuality")
        .and_then(|v| v.as_str())
        .unwrap_or("high");
    
    let quality = match quality_str {
        "low" => RecordingQuality::Low,
        "medium" => RecordingQuality::Medium,
        "high" => RecordingQuality::High,
        "ultra" => RecordingQuality::Ultra,
        _ => RecordingQuality::High,
    };
    
    Ok(quality)
}

fn log_quality_info(quality: &RecordingQuality) {
    let resolution_info = quality
        .target_resolution()
        .map(|(w, h)| format!("{}x{}", w, h))
        .unwrap_or_else(|| "native".to_string());
    log::info!(
        "📊 Recording quality: {:?} ({}p, {} Mbps)",
        quality,
        resolution_info,
        quality.bitrate() / 1_000_000
    );
}

pub(crate) fn start_recording_with_quality(
    state: &State<'_, AppState>,
    output_path: &str,
    quality: RecordingQuality,
) -> Result<(), Error> {
    let mut recorder_lock = state
        .recorder
        .lock()
        .map_err(|e| Error::InitializationError(format!("Failed to lock recorder: {}", e)))?;
    
    if recorder_lock.is_none() {
        *recorder_lock = Some(recorder::get_recorder());
    }
    
    if let Some(recorder) = recorder_lock.as_mut() {
        recorder.start_recording(output_path, quality)?;
        Ok(())
    } else {
        Err(Error::InitializationError("Failed to initialize recorder".to_string()))
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn configure_target_window(state: &State<'_, AppState>) {
    let identifier = match state.settings.lock() {
        Ok(settings) => settings
            .get("game_process_name")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string()),
        Err(err) => {
            log::error!("Failed to lock settings while configuring target window: {}", err);
            None
        }
    };
    
    if let Some(id_string) = identifier {
        if id_string.is_empty() {
            return;
        }
        
        std::env::set_var("PEPPI_TARGET_WINDOW", &id_string);
        
        if let Some(pos) = id_string.find("(PID:") {
            let after = &id_string[pos + 5..];
            let digits: String = after.chars().filter(|c| c.is_ascii_digit()).collect();
            if !digits.is_empty() {
                std::env::set_var("PEPPI_TARGET_PID", digits);
            }
        }
        
        log::info!("Providing target window to recorder: {}", id_string);
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn configure_target_window(_state: &State<'_, AppState>) {}

fn generate_generic_recording_path(recording_dir: &str) -> String {
    let now = chrono::Utc::now();
    let timestamp = now.format("%Y%m%dT%H%M%S").to_string();
    
    let mut counter = 0;
    loop {
        let filename = if counter == 0 {
            format!("Manual_{}.mp4", timestamp)
        } else {
            format!("Manual_{}_{}.mp4", timestamp, counter)
        };
        
        let candidate = Path::new(recording_dir).join(&filename);
        if !candidate.exists() {
            return candidate.to_string_lossy().to_string();
        }
        
        counter += 1;
    }
}

