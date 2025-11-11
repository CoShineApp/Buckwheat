use crate::app_state::AppState;
use crate::commands::errors::Error;
use crate::game_detector::{slippi_paths, GameDetector};
use crate::recorder;
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use sysinfo::System;
use tauri::{Emitter, Listener, Manager, State};
use walkdir::WalkDir;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetDC,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS,
    HGDIOBJ, SRCCOPY,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetClientRect, GetWindow, GetWindowRect, GetWindowTextW,
    GetWindowThreadProcessId, GW_OWNER,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameWindow {
    pub process_name: String,
    pub window_title: String,
    pub width: i32,
    pub height: i32,
    pub process_id: u32,
    pub class_name: String,
    pub is_cloaked: bool,
    pub is_child: bool,
    pub has_owner: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlippiMetadata {
    pub characters: Vec<u8>,
    pub stage: u16,
    pub players: Vec<PlayerInfo>,
    pub game_duration: i32,
    pub start_time: String,
    pub is_pal: bool,
    pub winner_port: Option<u8>,
    pub played_on: Option<String>,
    pub total_frames: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerInfo {
    pub character_id: u8,
    pub character_color: u8,
    pub player_tag: String,
    pub port: u8,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecordingSession {
    pub id: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub slp_path: String,
    pub video_path: Option<String>,
    pub duration: Option<u64>,
    pub file_size: Option<u64>,
    pub slippi_metadata: Option<SlippiMetadata>,
}

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
    state.last_replay_path.lock().ok().and_then(|path| path.clone())
}

/// Start watching for new Slippi games
#[tauri::command]
pub async fn start_watching(path: String, app: tauri::AppHandle, state: State<'_, AppState>) -> Result<(), Error> {
    log::info!("📁 Starting to watch Slippi folder: {}", path);

    let slippi_path = PathBuf::from(&path);

    // Check if path exists
    if !slippi_path.exists() {
        log::error!("❌ Path does not exist: {}", path);
        return Err(Error::InvalidPath(format!(
            "Slippi folder does not exist: {}",
            path
        )));
    }

    log::info!("✅ Path exists: {}", path);
    log::info!("📊 Path is directory: {}", slippi_path.is_dir());

    // Create new GameDetector with app handle
    let mut detector = GameDetector::new(slippi_path);
    detector.set_app_handle(app.clone());
    
    log::info!("🚀 Calling detector.start_watching()");
    detector.start_watching()?;
    log::info!("✅ detector.start_watching() completed successfully");

    // Store in app state
    let mut game_detector = state
        .game_detector
        .lock()
        .map_err(|e| Error::InitializationError(format!("Failed to lock game detector: {}", e)))?;
    *game_detector = Some(detector);

    // Set up event listener for auto-recording start
    let app_clone = app.clone();
    log::info!("🎧 Setting up event listener for 'slp-file-created' events");
    
    let app_clone2 = app.clone();
    app.listen("slp-file-created", move |event| {
        let slp_path = event.payload();
        log::info!("📥 ========================================");
        log::info!("📥 Received slp-file-created event!");
        log::info!("📥 Payload: {}", slp_path);
        log::info!("📥 ========================================");
        
        let app_handle = app_clone.clone();
        
        // Get state from app handle
        let state_ref = app_handle.state::<AppState>();
        
        // Store the last replay path
        log::info!("💾 Attempting to store last replay path");
        if let Ok(mut last_replay) = state_ref.last_replay_path.lock() {
            *last_replay = Some(slp_path.to_string());
            log::info!("✅ Last replay path stored: {}", slp_path);
            
            // Emit event to frontend to update UI
            log::info!("📤 Emitting last-replay-updated event to frontend");
            match app_handle.emit("last-replay-updated", slp_path) {
                Ok(_) => log::info!("✅ Frontend event emitted successfully"),
                Err(e) => log::error!("❌ Failed to emit last-replay-updated event: {:?}", e),
            }
        } else {
            log::error!("❌ Failed to lock last_replay_path mutex");
        }
        
        // Check if auto-start recording is enabled
        if let Ok(settings) = state_ref.settings.lock() {
            let auto_start = settings
                .get("autoStartRecording")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            
            if !auto_start {
                log::info!("⏭️  Auto-start recording is disabled");
                return;
            }
        }
        
        // Check if already recording
        if let Ok(recorder_lock) = state_ref.recorder.lock() {
            if recorder_lock.is_some() {
                log::info!("⏭️  Already recording, skipping");
                return;
            }
        }
        
        // Start recording and track the file
        if let Ok(mut current_file) = state_ref.current_recording_file.lock() {
            *current_file = Some(slp_path.to_string());
            log::info!("📝 Tracking recording file for game end detection: {}", slp_path);
        }
        
        let slp_path_for_recording = slp_path.to_string();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = trigger_auto_recording(app_handle, slp_path_for_recording).await {
                log::error!("Failed to trigger auto-recording: {:?}", e);
            }
        });
    });
    
    // Set up event listener for file modifications (game ending!)
    log::info!("🎧 Setting up event listener for 'slp-file-modified' events");
    let app_clone2_inner = app_clone2.clone();
    app_clone2.listen("slp-file-modified", move |event| {
        let modified_path = event.payload();
        log::info!("📝 File modified - game likely ended: {}", modified_path);
        
        let state_ref = app_clone2_inner.state::<AppState>();
        
        // Check if this is the file we're currently recording
        if let Ok(current_file) = state_ref.current_recording_file.lock() {
            if let Some(recording_file) = current_file.as_ref() {
                if recording_file == modified_path {
                    log::info!("✅ Detected modification of recording file - game ended!");
                    drop(current_file); // Drop the lock before spawning task
                    
                    // Slippi writes all data at once when game ends
                    // Wait a few seconds to ensure write is complete, then stop recording
                    let app_handle = app_clone2_inner.clone();
                    tauri::async_runtime::spawn(async move {
                        log::info!("⏰ Waiting 3 seconds for file write to complete...");
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                        
                        log::info!("🛑 Stopping recording after game end...");
                        if let Err(e) = stop_recording_internal(&app_handle).await {
                            log::error!("Failed to stop recording: {:?}", e);
                        }
                    });
                }
            }
        };
    });

    log::info!("✅ Now watching for .slp files");
    Ok(())
}

async fn stop_recording_internal(app: &tauri::AppHandle) -> Result<(), Error> {
    let state = app.state::<AppState>();
    
    // Stop recording
    let mut recorder_lock = state
        .recorder
        .lock()
        .map_err(|e| Error::RecordingFailed(format!("Failed to lock recorder: {}", e)))?;
    
    if let Some(recorder) = recorder_lock.as_mut() {
        let output_path = recorder.stop_recording()?;
        log::info!("✅ Auto-stopped recording: {}", output_path);
        
        // Clear recording state
        *recorder_lock = None;
        drop(recorder_lock);
        
        if let Ok(mut current_file) = state.current_recording_file.lock() {
            *current_file = None;
        }
        if let Ok(mut last_mod) = state.last_file_modification.lock() {
            *last_mod = None;
        }
        
        // Emit event to frontend
        if let Err(e) = app.emit("recording-stopped", output_path) {
            log::error!("Failed to emit recording-stopped event: {:?}", e);
        }
        
        Ok(())
    } else {
        Err(Error::RecordingFailed("No active recording".to_string()))
    }
}

async fn trigger_auto_recording(app: tauri::AppHandle, slp_path: String) -> Result<(), Error> {
    log::info!("🎬 Triggering auto-recording for: {}", slp_path);
    
    let state = app.state::<AppState>();
    
    // Generate output path matching the .slp filename
    let recording_dir = match get_recording_directory_internal(&app).await {
        Ok(dir) => dir,
        Err(e) => {
            log::error!("Failed to get recording directory: {:?}", e);
            return Err(e);
        }
    };
    
    // Extract filename from .slp path and change extension to .mp4
    let slp_filename = std::path::Path::new(&slp_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("recording");
    
    let output_path = format!("{}/{}.mp4", recording_dir, slp_filename);
    log::info!("📹 Output path: {}", output_path);
    
    // Get or create recorder
    let mut recorder_lock = state
        .recorder
        .lock()
        .map_err(|e| Error::InitializationError(format!("Failed to lock recorder: {}", e)))?;
    
    // Create new recorder if none exists
    if recorder_lock.is_none() {
        *recorder_lock = Some(recorder::get_recorder());
    }
    
    // Provide the selected target window (if any) to the Windows recorder via env
    #[cfg(target_os = "windows")]
    {
        if let Ok(settings) = state.settings.lock() {
            if let Some(val) = settings.get("game_process_name").and_then(|v| v.as_str()) {
                let id_string = val.trim().to_string();
                if !id_string.is_empty() {
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
        }
    }
    
    // Start recording
    if let Some(recorder) = recorder_lock.as_mut() {
        recorder.start_recording(&output_path)?;
        log::info!("✅ Auto-recording started: {}", output_path);
        
        // Store the .slp path associated with this recording
        if let Ok(mut current_file) = state.current_recording_file.lock() {
            *current_file = Some(slp_path.clone());
            log::info!("💾 Stored current recording .slp: {}", slp_path);
        }
        
        // Emit event to frontend
        if let Err(e) = app.emit("recording-started", output_path) {
            log::error!("Failed to emit recording-started event: {:?}", e);
        }
        
        Ok(())
    } else {
        Err(Error::InitializationError(
            "Failed to initialize recorder".to_string(),
        ))
    }
}

async fn get_recording_directory_internal(app: &tauri::AppHandle) -> Result<String, Error> {
    use tauri_plugin_store::StoreExt;
    
    let store = app.store("settings.json")
        .map_err(|e| Error::InitializationError(format!("Failed to open settings store: {}", e)))?;
    
    if let Some(value) = store.get("recordingPath") {
        if let Some(path) = value.as_str() {
            if !path.is_empty() {
                let path_string = path.to_string();
                std::fs::create_dir_all(&path_string)
                    .map_err(|e| Error::RecordingFailed(format!("Failed to create directory: {}", e)))?;
                return Ok(path_string);
            }
        }
    }
    
    // Use default: Videos/Buckwheat
    let default_dir = app
        .path()
        .video_dir()
        .map_err(|e| Error::InitializationError(format!("Failed to get videos directory: {}", e)))?
        .join("Buckwheat");
    
    std::fs::create_dir_all(&default_dir)
        .map_err(|e| Error::RecordingFailed(format!("Failed to create default directory: {}", e)))?;
    
    default_dir
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| Error::InvalidPath("Failed to convert path to string".to_string()))
}

/// Stop watching for new games
#[tauri::command]
pub async fn stop_watching(state: State<'_, AppState>) -> Result<(), Error> {
    log::info!("⏹️  Stopping file watcher");

    let mut game_detector = state
        .game_detector
        .lock()
        .map_err(|e| Error::InitializationError(format!("Failed to lock game detector: {}", e)))?;

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
    let mut recorder_lock = state
        .recorder
        .lock()
        .map_err(|e| Error::InitializationError(format!("Failed to lock recorder: {}", e)))?;

    // Create new recorder if none exists
    if recorder_lock.is_none() {
        *recorder_lock = Some(recorder::get_recorder());
    }

    // Provide the selected target window (if any) to the Windows recorder via env
    #[cfg(target_os = "windows")]
    {
        if let Ok(settings) = state.settings.lock() {
            if let Some(val) = settings.get("game_process_name").and_then(|v| v.as_str()) {
                let id_string = val.trim().to_string();
                if !id_string.is_empty() {
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
        }
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

    let mut recorder_lock = state
        .recorder
        .lock()
        .map_err(|e| Error::RecordingFailed(format!("Failed to lock recorder: {}", e)))?;

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

/// Delete a recording (video and optionally .slp file)
#[tauri::command]
pub async fn delete_recording(
    video_path: Option<String>,
    _slp_path: String,
) -> Result<(), Error> {
    log::info!("🗑️  Deleting recording...");
    
    // Delete video file if it exists
    if let Some(video) = video_path {
        if !video.is_empty() && std::path::Path::new(&video).exists() {
            std::fs::remove_file(&video)
                .map_err(|e| Error::RecordingFailed(format!("Failed to delete video: {}", e)))?;
            log::info!("✅ Deleted video: {}", video);
        }
    }
    
    // Delete .slp file if it exists and user wants to
    // For now, we'll keep the .slp files since they're the source of truth
    // Uncomment this if you want to delete .slp files too:
    /*
    if !slp_path.is_empty() && std::path::Path::new(&slp_path).exists() {
        std::fs::remove_file(&slp_path)
            .map_err(|e| Error::RecordingFailed(format!("Failed to delete .slp: {}", e)))?;
        log::info!("✅ Deleted .slp: {}", slp_path);
    }
    */
    
    log::info!("✅ Recording deleted successfully");
    Ok(())
}

/// Open a video file in the default player
#[tauri::command]
pub async fn open_video(video_path: String) -> Result<(), Error> {
    log::info!("🎬 Opening video: {}", video_path);
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", "", &video_path])
            .spawn()
            .map_err(|e| Error::RecordingFailed(format!("Failed to open video: {}", e)))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&video_path)
            .spawn()
            .map_err(|e| Error::RecordingFailed(format!("Failed to open video: {}", e)))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&video_path)
            .spawn()
            .map_err(|e| Error::RecordingFailed(format!("Failed to open video: {}", e)))?;
    }
    
    Ok(())
}

/// Open the folder containing a video file
#[tauri::command]
pub async fn open_recording_folder(video_path: String) -> Result<(), Error> {
    log::info!("📂 Opening folder for: {}", video_path);
    
    let path = std::path::Path::new(&video_path);
    let folder = path.parent()
        .ok_or_else(|| Error::InvalidPath("Failed to get parent directory".to_string()))?;
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(folder)
            .spawn()
            .map_err(|e| Error::RecordingFailed(format!("Failed to open folder: {}", e)))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(folder)
            .spawn()
            .map_err(|e| Error::RecordingFailed(format!("Failed to open folder: {}", e)))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(folder)
            .spawn()
            .map_err(|e| Error::RecordingFailed(format!("Failed to open folder: {}", e)))?;
    }
    
    Ok(())
}

/// Get list of recorded sessions
#[tauri::command]
pub async fn get_recordings(app: tauri::AppHandle) -> Result<Vec<RecordingSession>, Error> {
    log::info!("📂 Scanning for recordings...");
    
    // Get recording directory
    let recording_dir = match get_recording_directory_internal(&app).await {
        Ok(dir) => dir,
        Err(e) => {
            log::error!("Failed to get recording directory: {:?}", e);
            return Ok(Vec::new()); // Return empty list instead of error
        }
    };
    
    log::info!("📁 Recording directory: {}", recording_dir);
    
    // Get Slippi directory
    let slippi_dir = {
        use tauri_plugin_store::StoreExt;
        let store = app.store("settings.json")
            .map_err(|e| Error::InitializationError(format!("Failed to open settings store: {}", e)))?;
        
        if let Some(value) = store.get("slippiPath") {
            if let Some(path) = value.as_str() {
                if !path.is_empty() {
                    Some(path.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };
    
    let slippi_dir = match slippi_dir {
        Some(path) => path,
        None => slippi_paths::get_default_slippi_path()
            .to_str()
            .unwrap_or("")
            .to_string(),
    };
    
    log::info!("📁 Slippi directory: {}", slippi_dir);
    
    // Scan for MP4 files in recording directory
    let mut recordings = Vec::new();
    
    for entry in WalkDir::new(&recording_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("mp4") {
            log::info!("🎥 Found video: {:?}", path);
            
            if let Ok(session) = create_recording_session(path, &slippi_dir).await {
                recordings.push(session);
            }
        }
    }
    
    // Sort by start time (newest first)
    recordings.sort_by(|a, b| b.start_time.cmp(&a.start_time));
    
    log::info!("✅ Found {} recordings", recordings.len());
    Ok(recordings)
}

async fn create_recording_session(
    video_path: &Path,
    slippi_dir: &str,
) -> Result<RecordingSession, Error> {
    let video_path_str = video_path.to_string_lossy().to_string();
    
    // Get file metadata
    let metadata = std::fs::metadata(video_path)
        .map_err(|e| Error::InvalidPath(format!("Failed to read file metadata: {}", e)))?;
    
    let file_size = metadata.len();
    let start_time = metadata.created()
        .or_else(|_| metadata.modified())
        .ok()
        .and_then(|t| {
            use std::time::SystemTime;
            t.duration_since(SystemTime::UNIX_EPOCH).ok()
        })
        .map(|d| {
            chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                .unwrap_or_default()
                .to_rfc3339()
        })
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    
    // Try to find matching .slp file
    // Look for files with similar timestamp
    let video_filename = video_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let slp_path = find_matching_slp(video_filename, slippi_dir).await;
    
    // Parse .slp file if found
    let (slippi_metadata, duration, end_time) = if let Some(ref slp) = slp_path {
        parse_slp_file(slp).await
    } else {
        (None, None, None)
    };
    
    // Generate ID from filename
    let id = video_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    Ok(RecordingSession {
        id,
        start_time,
        end_time,
        slp_path: slp_path.unwrap_or_default(),
        video_path: Some(video_path_str),
        duration,
        file_size: Some(file_size),
        slippi_metadata,
    })
}

async fn find_matching_slp(video_filename: &str, slippi_dir: &str) -> Option<String> {
    // Video files now have format: Game_20251110T200349.mp4
    // .slp files have format: Game_20251110T200349.slp
    // They should match exactly!
    
    log::debug!("🔍 Looking for .slp file matching: {}", video_filename);
    
    // Build expected .slp path
    let slp_filename = format!("{}.slp", video_filename);
    
    // Search for exact match first
    for entry in WalkDir::new(slippi_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
            if filename == slp_filename {
                log::debug!("✅ Found exact match: {:?}", path);
                return Some(path.to_string_lossy().to_string());
            }
        }
    }
    
    log::warn!("⚠️ No matching .slp file found for: {}", video_filename);
    None
}

async fn parse_slp_file(slp_path: &str) -> (Option<SlippiMetadata>, Option<u64>, Option<String>) {
    use std::fs::File;
    use std::io::BufReader;
    use peppi::io::slippi::read;
    
    log::info!("📊 Parsing .slp file: {}", slp_path);
    
    let file = match File::open(slp_path) {
        Ok(f) => f,
        Err(e) => {
            log::error!("Failed to open .slp file: {:?}", e);
            return (None, None, None);
        }
    };
    
    let mut reader = BufReader::new(file);
    
    match read(&mut reader, None) {
        Ok(game) => {
            log::info!("✅ Successfully parsed .slp file");
            
            // SIMPLE: Just use peppi's data directly
            let mut characters = Vec::new();
            let mut players = Vec::new();
            
            // Get player codes from metadata JSON
            let player_metadata = game.metadata.as_ref()
                .and_then(|m| m.get("players"))
                .and_then(|p| p.as_object());
            
            // Get winner from end game data
            let winner_port = game.end.as_ref()
                .and_then(|end| end.players.as_ref())
                .and_then(|end_players| {
                    end_players.iter()
                        .find(|p| p.placement == 0)
                        .map(|p| u8::from(p.port))
                });
            
            log::info!("🏆 Winner port: {:?}", winner_port);
            
            // Iterate through players - peppi gives us the correct data
            for player in &game.start.players {
                let port = u8::from(player.port);
                let char_id = player.character as u8;
                
                characters.push(char_id);
                
                // Get player tag from metadata
                let player_tag = player_metadata
                    .and_then(|m| m.get(&port.to_string()))
                    .and_then(|p| p.get("names"))
                    .and_then(|n| n.get("code").or_else(|| n.get("netplay")))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("P{}", port));
                
                log::info!("👤 Port {}: {} playing character ID {}", port, player_tag, char_id);
                
                players.push(PlayerInfo {
                    character_id: char_id,
                    character_color: player.costume,
                    player_tag,
                    port,
                });
            }
            
            let stage = game.start.stage as u16;
            log::info!("🎭 Stage ID: {}", stage);
            
            // Get duration from metadata
            let game_duration = game.metadata.as_ref()
                .and_then(|m| m.get("lastFrame"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            
            let duration_secs = (game_duration as f64 / 60.0) as u64;
            log::info!("⏱️  Duration: {} frames = {} seconds", game_duration, duration_secs);
            
            // Get start time from metadata
            let start_time = game.metadata.as_ref()
                .and_then(|m| m.get("startAt"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
            
            let is_pal = game.start.is_pal.unwrap_or(false);
            
            // Get additional metadata
            let played_on = game.metadata.as_ref()
                .and_then(|m| m.get("playedOn"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            
            let total_frames = game.frames.len() as i32;
            
            let metadata = SlippiMetadata {
                characters,
                stage,
                players,
                game_duration,
                start_time: start_time.clone(),
                is_pal,
                winner_port,
                played_on,
                total_frames,
            };
            
            (Some(metadata), Some(duration_secs), Some(start_time))
        }
        Err(e) => {
            log::error!("Failed to parse .slp file: {:?}", e);
            (None, None, None)
        }
    }
}

#[cfg(target_os = "windows")]
struct ChildEnumContext {
    windows: Vec<GameWindow>,
    parent_pid: u32,
}

/// Enumerate all windows and find potential game windows (Windows-specific)
#[cfg(target_os = "windows")]
#[tauri::command]
pub fn list_game_windows() -> Result<Vec<GameWindow>, Error> {
    use std::collections::HashMap;

    log::info!("🔍 Enumerating windows to find game candidates...");

    // First, get all processes with sysinfo
    let mut sys = System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

    // Map PIDs to process names
    let mut pid_to_name: HashMap<u32, String> = HashMap::new();
    for (pid, process) in sys.processes() {
        pid_to_name.insert(pid.as_u32(), process.name().to_string_lossy().to_string());
    }

    let mut windows: Vec<GameWindow> = Vec::new();

    unsafe {
        // Enumerate all top-level windows
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut windows as *mut Vec<GameWindow> as isize),
        );

        // Note: We intentionally avoid the previous PID-based child scan here
        // because it produced large duplicate sets. If needed, we can add a
        // proper per-parent `EnumChildWindows` pass in a follow-up.
    }

    // Attach process names using the PID->name map
    for w in &mut windows {
        if let Some(name) = pid_to_name.get(&w.process_id) {
            w.process_name = name.clone();
        }
    }

    // Heuristic scoring to prioritize the actual render/game window
    fn score_window(w: &GameWindow) -> i32 {
        let mut s = 0;
        let title = w.window_title.to_lowercase();
        if title.contains("slippi") || title.contains("melee") || title.contains("dolphin") {
            s += 3;
        }
        if title.contains("launcher")
            || title.contains("settings")
            || title.contains("configuration")
        {
            s -= 3;
        }
        if w.width >= 640 && w.height >= 480 && !w.is_cloaked {
            s += 3;
        }
        // Owner often indicates dialogs; don't boost here
        if w.height > 0 {
            let ar = (w.width as f32) / (w.height as f32);
            let d43 = (ar - (4.0 / 3.0)).abs();
            let d169 = (ar - (16.0 / 9.0)).abs();
            if d43 < 0.08 || d169 < 0.08 {
                s += 2;
            }
        }
        let class = w.class_name.to_lowercase();
        if class.contains("dolphin") || class.contains("wxwindownr") {
            s += 3;
        }
        if class.starts_with("#32770") || class.contains("tooltips") {
            s -= 4;
        }
        s
    }

    // Pre-filter to likely Dolphin/Slippi candidates and sensible sizes
    let prefiltered: Vec<GameWindow> = windows
        .clone()
        .into_iter()
        .filter(|w| {
            let pn = w.process_name.to_lowercase();
            let tl = w.window_title.to_lowercase();
            let cn = w.class_name.to_lowercase();
            let keyword = pn.contains("dolphin")
                || pn.contains("slippi")
                || pn.contains("melee")
                || tl.contains("slippi")
                || tl.contains("melee")
                || tl.contains("dolphin")
                || cn.contains("wxwindownr");
            keyword && w.width >= 640 && w.height >= 480 && !w.is_cloaked
        })
        .collect();

    // Build a scored candidate list; if empty fall back to the legacy filter
    let mut scored: Vec<GameWindow> = prefiltered
        .into_iter()
        .filter(|w| score_window(w) >= 2)
        .collect();
    scored.sort_by_key(|w| -score_window(w));

    // Filter for likely game windows (prefer scored results)
    let mut game_windows: Vec<GameWindow> = if !scored.is_empty() {
        scored
    } else {
        windows
            .into_iter()
            .filter(|w| {
                let title_lower = w.window_title.to_lowercase();
                (title_lower.contains("slippi")
                    || title_lower.contains("melee")
                    || title_lower.contains("dolphin"))
                    && !title_lower.contains("launcher")
                    && !title_lower.contains("settings")
                    && !title_lower.contains("configuration")
                    && w.width >= 640
                    && w.height >= 480
                    && !w.is_cloaked
            })
            .collect()
    };

    // De-duplicate by (pid,title,size,class)
    {
        use std::collections::HashSet;
        let mut seen: HashSet<String> = HashSet::new();
        game_windows.retain(|w| {
            let key = format!(
                "{}:{}x{}:{}:{}",
                w.process_id, w.width, w.height, w.class_name, w.window_title
            );
            seen.insert(key)
        });
    }

    log::info!("✅ Found {} potential game windows", game_windows.len());
    for window in &game_windows {
        log::info!(
            "  - PID: {} | Title: {} | Size: {}x{} | Class: {} | Cloaked: {} | Child: {} | HasOwner: {}", 
            window.process_id,
            window.window_title, 
            window.width, 
            window.height,
            window.class_name,
            window.is_cloaked,
            window.is_child,
            window.has_owner
        );
    }

    Ok(game_windows)
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<GameWindow>);

    // Process all windows (not just visible ones, to catch more possibilities)
    // Get window title (might be empty for some windows)
    let mut title: [u16; 512] = [0; 512];
    let len = GetWindowTextW(hwnd, &mut title);
    let window_title = if len > 0 {
        String::from_utf16_lossy(&title[..len as usize])
    } else {
        "(No Title)".to_string()
    };

    // Get window dimensions
    let mut rect = RECT::default();
    if GetWindowRect(hwnd, &mut rect).is_ok() {
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        // Get process ID
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        // Get window class name
        let mut class_name: [u16; 256] = [0; 256];
        let class_len = GetClassNameW(hwnd, &mut class_name);
        let class_name_str = if class_len > 0 {
            String::from_utf16_lossy(&class_name[..class_len as usize])
        } else {
            "Unknown".to_string()
        };

        // Check if window is cloaked (hidden but still visible)
        let mut is_cloaked = 0u32;
        let cloaked = DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut is_cloaked as *mut _ as *mut _,
            std::mem::size_of::<u32>() as u32,
        )
        .is_ok()
            && is_cloaked != 0;

        // Check if window has an owner (owned windows are often dialogs/popups)
        let has_owner = GetWindow(hwnd, GW_OWNER)
            .map(|h| !h.is_invalid())
            .unwrap_or(false);

        windows.push(GameWindow {
            process_name: format!("PID: {}", process_id),
            window_title: window_title.clone(),
            width,
            height,
            process_id,
            class_name: class_name_str,
            is_cloaked: cloaked,
            is_child: false, // Top-level windows are not children
            has_owner,
        });
    }

    BOOL::from(true) // Continue enumeration
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_child_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let context = &mut *(lparam.0 as *mut ChildEnumContext);

    // Get process ID for this window
    let mut process_id: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut process_id));

    // Only process windows from the same process as parent
    if process_id == context.parent_pid {
        let mut title: [u16; 512] = [0; 512];
        let len = GetWindowTextW(hwnd, &mut title);
        let window_title = if len > 0 {
            String::from_utf16_lossy(&title[..len as usize])
        } else {
            "(No Title - Child)".to_string()
        };

        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            let mut class_name: [u16; 256] = [0; 256];
            let class_len = GetClassNameW(hwnd, &mut class_name);
            let class_name_str = if class_len > 0 {
                String::from_utf16_lossy(&class_name[..class_len as usize])
            } else {
                "Unknown".to_string()
            };

            let mut is_cloaked = 0u32;
            let cloaked = DwmGetWindowAttribute(
                hwnd,
                DWMWA_CLOAKED,
                &mut is_cloaked as *mut _ as *mut _,
                std::mem::size_of::<u32>() as u32,
            )
            .is_ok()
                && is_cloaked != 0;

            let has_owner = GetWindow(hwnd, GW_OWNER)
                .map(|h| !h.is_invalid())
                .unwrap_or(false);

            // Only add if it has reasonable dimensions
            if width > 100 && height > 100 {
                context.windows.push(GameWindow {
                    process_name: format!("PID: {} (Child)", process_id),
                    window_title,
                    width,
                    height,
                    process_id,
                    class_name: class_name_str,
                    is_cloaked: cloaked,
                    is_child: true,
                    has_owner,
                });
            }
        }
    }

    BOOL::from(true) // Continue enumeration
}

/// List game windows (stub for non-Windows platforms)
#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn list_game_windows() -> Result<Vec<GameWindow>, Error> {
    Ok(vec![])
}

/// Check if the Slippi/Dolphin game window is currently open
/// Uses the stored process name from settings, or falls back to auto-detection
#[tauri::command]
pub async fn check_game_window(state: State<'_, AppState>) -> Result<bool, Error> {
    // Read stored identifier (may include PID)
    let settings = state
        .settings
        .lock()
        .map_err(|e| Error::InitializationError(format!("Failed to lock settings: {}", e)))?;
    let stored_id = settings
        .get("game_process_name")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    drop(settings); // Release lock

    // Windows: use window enumeration + scoring to detect the actual game window
    #[cfg(target_os = "windows")]
    {
        use sysinfo::System;

        let mut windows: Vec<GameWindow> = Vec::new();
        unsafe {
            let _ = EnumWindows(
                Some(enum_windows_callback),
                LPARAM(&mut windows as *mut Vec<GameWindow> as isize),
            );
            let copy = windows.clone();
            for parent in copy {
                let mut ctx = ChildEnumContext {
                    windows: Vec::new(),
                    parent_pid: parent.process_id,
                };
                let _ = EnumWindows(
                    Some(enum_child_windows_callback),
                    LPARAM(&mut ctx as *mut ChildEnumContext as isize),
                );
                windows.extend(ctx.windows);
            }
        }

        // Attach process names
        let mut sys = System::new_all();
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
        for w in &mut windows {
            if let Some(p) = sys.process(sysinfo::Pid::from_u32(w.process_id)) {
                w.process_name = p.name().to_string_lossy().to_string();
            }
        }

        // Optional narrowing from stored selection
        let mut pid_filter: Option<u32> = None;
        let mut title_filter: Option<String> = None;
        if let Some(id) = stored_id.clone() {
            if let Some(pos) = id.find("PID:") {
                let after = id[pos + 4..].trim_start();
                let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                pid_filter = digits.parse::<u32>().ok();
            } else {
                title_filter = Some(id.to_lowercase());
            }
        }

        fn score_window(w: &GameWindow) -> i32 {
            let mut s = 0;
            let t = w.window_title.to_lowercase();
            if t.contains("slippi") || t.contains("melee") || t.contains("dolphin") {
                s += 3;
            }
            if t.contains("launcher") || t.contains("settings") || t.contains("configuration") {
                s -= 3;
            }
            if w.width >= 640 && w.height >= 480 && !w.is_cloaked {
                s += 3;
            }
            // Owner can indicate dialogs; don't boost
            if w.height > 0 {
                let ar = (w.width as f32) / (w.height as f32);
                let d43 = (ar - (4.0 / 3.0)).abs();
                let d169 = (ar - (16.0 / 9.0)).abs();
                if d43 < 0.08 || d169 < 0.08 {
                    s += 2;
                }
            }
            let c = w.class_name.to_lowercase();
            if c.contains("dolphin") || c.contains("wxwindownr") {
                s += 3;
            }
            if c.starts_with("#32770") || c.contains("tooltips") {
                s -= 4;
            }
            s
        }

        let mut candidates: Vec<&GameWindow> = windows.iter().collect();
        if let Some(pid) = pid_filter {
            candidates.retain(|w| w.process_id == pid);
        }
        if let Some(ref tf) = title_filter {
            candidates.retain(|w| w.window_title.to_lowercase().contains(tf));
        }
        if pid_filter.is_none() && title_filter.is_none() {
            candidates.retain(|w| {
                let pn = w.process_name.to_lowercase();
                pn.contains("dolphin") || pn.contains("slippi") || pn.contains("melee")
            });
        }

        let best = candidates.into_iter().max_by_key(|w| score_window(w));
        if let Some(w) = best {
            return Ok(score_window(w) >= 4);
        }
        return Ok(false);
    }

    // Non-Windows platforms
    #[cfg(not(target_os = "windows"))]
    return Ok(false);
}

/// Capture a one-off preview of the selected game window (base64 PNG)
#[tauri::command]
pub async fn capture_window_preview(state: State<'_, AppState>) -> Result<Option<String>, Error> {
    #[cfg(target_os = "windows")]
    {
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

        match capture_window_preview_impl(&target_id) {
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

    #[cfg(not(target_os = "windows"))]
    {
        let _ = state;
        Ok(None)
    }
}

// (Removed dynamic dimension override to let backend use exact window size)

/// Get the stored or detected game process name
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

/// Set the game process name to use for detection and recording
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

/// Open a file location in the system file explorer
#[tauri::command]
pub fn open_file_location(path: String) -> Result<(), Error> {
    use std::path::Path;

    let file_path = Path::new(&path);
    let dir_path = if file_path.is_file() {
        file_path
            .parent()
            .ok_or_else(|| Error::InvalidPath("Could not get parent directory".to_string()))?
    } else {
        file_path
    };

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(dir_path)
            .spawn()
            .map_err(|e| Error::RecordingFailed(format!("Failed to open folder: {}", e)))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(dir_path)
            .spawn()
            .map_err(|e| Error::RecordingFailed(format!("Failed to open folder: {}", e)))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(dir_path)
            .spawn()
            .map_err(|e| Error::RecordingFailed(format!("Failed to open folder: {}", e)))?;
    }

    Ok(())
}

// --- Windows-only helpers for preview capture ---

#[cfg(target_os = "windows")]
fn capture_window_preview_impl(identifier: &str) -> Result<Vec<u8>, String> {
    let hwnd = find_window_handle(identifier).ok_or_else(|| {
        format!(
            "No window found matching identifier '{}'",
            identifier.trim()
        )
    })?;
    capture_hwnd_png(hwnd)
}

#[cfg(target_os = "windows")]
fn find_window_handle(identifier: &str) -> Option<HWND> {
    let (title, pid) = parse_identifier(identifier);
    if title.is_empty() && pid.is_none() {
        return None;
    }

    let mut ctx = WindowSearchContext {
        pid,
        needle: title.to_lowercase(),
        hwnd: None,
    };
    unsafe {
        let _ = EnumWindows(
            Some(find_window_enum_callback),
            LPARAM(&mut ctx as *mut WindowSearchContext as isize),
        );
    }
    if ctx.hwnd.is_none() && pid.is_some() {
        let mut fallback = WindowSearchContext {
            pid: None,
            needle: title.to_lowercase(),
            hwnd: None,
        };
        unsafe {
            let _ = EnumWindows(
                Some(find_window_enum_callback),
                LPARAM(&mut fallback as *mut WindowSearchContext as isize),
            );
        }
        return fallback.hwnd;
    }
    ctx.hwnd
}

#[cfg(target_os = "windows")]
fn parse_identifier(identifier: &str) -> (String, Option<u32>) {
    let trimmed = identifier.trim();
    if trimmed.is_empty() {
        return (String::new(), None);
    }
    if let Some(idx) = trimmed.rfind("(PID:") {
        let title = trimmed[..idx].trim_end().to_string();
        let digits: String = trimmed[idx + 5..]
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        let pid = digits.parse::<u32>().ok();
        (title, pid)
    } else {
        (trimmed.to_string(), None)
    }
}

#[cfg(target_os = "windows")]
struct WindowSearchContext {
    pid: Option<u32>,
    needle: String,
    hwnd: Option<HWND>,
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn find_window_enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let ctx = &mut *(lparam.0 as *mut WindowSearchContext);
    if let Some(pid) = ctx.pid {
        let mut window_pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut window_pid));
        if window_pid != pid {
            return BOOL(1);
        }
    }

    let mut buf: [u16; 512] = [0; 512];
    let len = GetWindowTextW(hwnd, &mut buf);
    if len == 0 {
        return BOOL(1);
    }
    let title = String::from_utf16_lossy(&buf[..len as usize]).to_lowercase();
    if ctx.needle.is_empty() || title.contains(&ctx.needle) {
        ctx.hwnd = Some(hwnd);
        BOOL(0)
    } else {
        BOOL(1)
    }
}

#[cfg(target_os = "windows")]
fn capture_hwnd_png(hwnd: HWND) -> Result<Vec<u8>, String> {
    unsafe {
        let mut rect = RECT::default();
        if GetClientRect(hwnd, &mut rect).is_err() {
            return Err("Failed to get window bounds".into());
        }
        let width = (rect.right - rect.left) as i32;
        let height = (rect.bottom - rect.top) as i32;
        if width <= 0 || height <= 0 {
            return Err("Window has invalid dimensions".into());
        }

        let hdc_window = GetDC(hwnd);
        if hdc_window.is_invalid() {
            return Err("Failed to acquire window device context".into());
        }
        let hdc_mem = CreateCompatibleDC(hdc_window);
        if hdc_mem.is_invalid() {
            ReleaseDC(hwnd, hdc_window);
            return Err("Failed to create memory device context".into());
        }
        let hbitmap = CreateCompatibleBitmap(hdc_window, width, height);
        if hbitmap.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(hwnd, hdc_window);
            return Err("Failed to create compatible bitmap".into());
        }

        let old_obj = SelectObject(hdc_mem, HGDIOBJ(hbitmap.0));
        if old_obj.is_invalid() {
            let _ = DeleteObject(HGDIOBJ(hbitmap.0));
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(hwnd, hdc_window);
            return Err("Failed to select bitmap into memory DC".into());
        }

        let blt_result = BitBlt(hdc_mem, 0, 0, width, height, hdc_window, 0, 0, SRCCOPY | CAPTUREBLT);
        if let Err(err) = blt_result {
            let _ = SelectObject(hdc_mem, old_obj);
            let _ = DeleteObject(HGDIOBJ(hbitmap.0));
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(hwnd, hdc_window);
            return Err(format!(
                "BitBlt failed while copying window content: {}",
                err
            ));
        }

        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [Default::default(); 1],
        };
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        let dib_res = GetDIBits(
            hdc_mem,
            hbitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        );

        let _ = SelectObject(hdc_mem, old_obj);
        let _ = DeleteObject(HGDIOBJ(hbitmap.0));
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(hwnd, hdc_window);

        if dib_res == 0 {
            return Err("Failed to read bitmap pixels".into());
        }

        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2); // Convert BGRA -> RGBA
        }

        let mut png_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_data, width as u32, height as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder
                .write_header()
                .map_err(|e| format!("Failed to write PNG header: {}", e))?;
            writer
                .write_image_data(&pixels)
                .map_err(|e| format!("Failed to encode PNG: {}", e))?;
        }

        Ok(png_data)
    }
}
