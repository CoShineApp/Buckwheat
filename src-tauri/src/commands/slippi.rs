use serde::{Deserialize, Serialize};
use crate::commands::errors::Error;
use crate::game_detector::slippi_paths;

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
pub async fn start_watching(path: String) -> Result<(), Error> {
    println!("Starting to watch: {}", path);
    // TODO: Implement actual file watching with GameDetector
    Ok(())
}

/// Stop watching for new games
#[tauri::command]
pub async fn stop_watching() -> Result<(), Error> {
    println!("Stopped watching");
    Ok(())
}

/// Start recording gameplay
#[tauri::command]
pub async fn start_recording(output_path: String) -> Result<(), Error> {
    println!("Starting recording to: {}", output_path);

    #[cfg(target_os = "windows")]
    {
        return Err(Error::InitializationError(
            "Windows recording not yet implemented".to_string(),
        ));
    }

    #[cfg(target_os = "macos")]
    {
        println!("Mock recording started (macOS development mode)");
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        return Err(Error::InitializationError(
            "Linux recording not yet implemented".to_string(),
        ));
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(Error::UnsupportedPlatform)
    }
}

/// Stop recording gameplay
#[tauri::command]
pub async fn stop_recording() -> Result<String, Error> {
    println!("Stopping recording");
    Ok("/mock/path/recording.mp4".to_string())
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

