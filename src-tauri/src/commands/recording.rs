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
pub async fn start_recording(
    app: tauri::AppHandle,
    output_path: String,
    state: State<'_, AppState>,
) -> Result<(), Error> {
    let quality = resolve_recording_quality(&state)?;
    log_quality_info(&quality);

    start_recording_with_quality(&app, &state, &output_path, quality)?;
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

    start_recording_with_quality(&app, &state, &output_path, quality)?;

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

        // Keep the recorder + OBS connection alive so we don't re-launch OBS every recording.
        // The recorder is dropped when the app exits (see lib.rs RunEvent::Exit handler).

        // Reattach any pending clip markers to the recording that just stopped.
        // Markers are created with the app's intended output path (e.g.
        // "Manual_....mp4"), but OBS writes to its own filename. Since markers
        // only accumulate during an active recording, every pending marker
        // belongs to the one we just stopped — rewrite their recording_file to
        // match so process_clip_markers can find them.
        let marker_snapshot = {
            let mut markers = state.clip_markers.lock().map_err(|e| {
                Error::InitializationError(format!("Failed to lock clip markers: {}", e))
            })?;
            for m in markers.iter_mut() {
                m.recording_file = output_path.clone();
            }
            markers.iter().map(|m| m.timestamp_seconds).collect::<Vec<_>>()
        };

        if marker_snapshot.is_empty() {
            log::info!("No clip markers queued for {}", output_path);
        } else {
            log::info!(
                "Reattached {} clip marker(s) to {}: {:?}",
                marker_snapshot.len(),
                output_path,
                marker_snapshot
            );
        }

        if let Err(e) = app.emit(recording_events::STOPPED, output_path.clone()) {
            log::error!("Failed to emit {} event: {:?}", recording_events::STOPPED, e);
        }

        // Always clear the current recording file tracker on stop — we're no
        // longer recording anything, and the game-end detector compares base
        // filenames so a stale "Manual_..." path would block auto-start on the
        // next .slp.
        if let Ok(mut current_file) = state.current_recording_file.lock() {
            *current_file = None;
        }

        // Enforce storage limit in background
        let app_clone = app.clone();
        tokio::spawn(async move {
            match library::enforce_storage_limit(&app_clone).await {
                Ok(result) => {
                    if !result.deleted_paths.is_empty() {
                        log::info!(
                            "[Storage] Cleanup: deleted {} recordings, freed {} bytes",
                            result.deleted_paths.len(),
                            result.bytes_freed
                        );
                        // Emit storage cleanup event to frontend
                        if let Err(e) = app_clone.emit(crate::events::storage::CLEANUP, &result) {
                            log::error!("Failed to emit storage-cleanup event: {:?}", e);
                        }
                    }
                }
                Err(e) => log::error!("[Storage] Failed to enforce storage limit: {:?}", e),
            }
        });

        Ok(output_path)
    } else {
        Err(Error::RecordingFailed("No active recording to stop".to_string()))
    }
}

/// Test the OBS websocket connection with the configured port/password.
#[tauri::command]
pub async fn ensure_obs_ready(app: tauri::AppHandle) -> Result<(), Error> {
    let (port, password) = read_obs_settings(&app);

    crate::recorder::obs::ObsRecorder::test_connection(port, &password)
        .map_err(|e| Error::InitializationError(format!("{e}")))?;

    Ok(())
}

/// No-op kept for frontend compat — OBS is managed by the user now.
#[tauri::command]
pub async fn install_obs() -> Result<(), Error> {
    Ok(())
}

/// Initialize the OBS recorder at app startup.
///
/// Connects to the user's running OBS, or spawns a managed instance if none
/// is found. The recorder stays alive for the lifetime of the app so subsequent
/// recordings don't need to relaunch OBS.
pub fn init_recorder_on_startup(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Give the app a moment to settle and for settings.json to be available
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let (port, password) = read_obs_settings(&app);
        let mut obs_recorder = crate::recorder::obs::ObsRecorder::new(port, password);

        if let Err(e) = obs_recorder.ensure_connected_async().await {
            log::warn!("OBS not ready at startup: {e}. Will retry on first recording.");
            return;
        }

        let state = {
            use tauri::Manager;
            app.state::<AppState>()
        };

        let mut recorder_lock = match state.recorder.lock() {
            Ok(l) => l,
            Err(e) => {
                log::error!("Failed to lock recorder at startup: {e}");
                return;
            }
        };

        if recorder_lock.is_none() {
            *recorder_lock = Some(Box::new(obs_recorder));
            log::info!("OBS recorder initialized at startup");
        }
    });
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
        "Recording quality: {:?} ({}p, {} Mbps)",
        quality,
        resolution_info,
        quality.bitrate() / 1_000_000
    );
}

/// Read OBS websocket port and password from the settings.json file.
pub(crate) fn read_obs_settings(app: &tauri::AppHandle) -> (u16, String) {
    let (port, password, _) = read_obs_settings_full(app);
    (port, password)
}

/// Read all OBS-related settings including the selected game window.
pub(crate) fn read_obs_settings_full(
    app: &tauri::AppHandle,
) -> (u16, String, Option<crate::obs::connection::GameWindowInfo>) {
    use tauri::Manager;

    let path = match app.path().app_data_dir() {
        Ok(p) => p.join("settings.json"),
        Err(_) => return (4455, String::new(), None),
    };

    let contents = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return (4455, String::new(), None),
    };

    let json: serde_json::Value = match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(_) => return (4455, String::new(), None),
    };

    let port = json
        .get("obsPort")
        .and_then(|v| v.as_u64())
        .unwrap_or(4455) as u16;

    let password = json
        .get("obsPassword")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let game_window = json
        .get("gameWindow")
        .and_then(|v| serde_json::from_value::<crate::obs::connection::GameWindowInfo>(v.clone()).ok());

    (port, password, game_window)
}

pub(crate) fn start_recording_with_quality(
    app: &tauri::AppHandle,
    state: &State<'_, AppState>,
    output_path: &str,
    quality: RecordingQuality,
) -> Result<(), Error> {
    let (port, password, game_window) = read_obs_settings_full(app);

    let mut recorder_lock = state
        .recorder
        .lock()
        .map_err(|e| Error::InitializationError(format!("Failed to lock recorder: {}", e)))?;

    if recorder_lock.is_none() {
        *recorder_lock = Some(recorder::get_recorder(port, password));
    }

    if let Some(recorder) = recorder_lock.as_mut() {
        recorder.set_target_window(game_window);
        recorder.start_recording(output_path, quality)?;
        Ok(())
    } else {
        Err(Error::InitializationError("Failed to initialize recorder".to_string()))
    }
}

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
