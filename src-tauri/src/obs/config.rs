use crate::recorder::RecordingQuality;
use std::path::Path;

/// Settings for the managed OBS instance.
pub struct ObsSettings {
    pub recording_path: String,
    pub quality: RecordingQuality,
    pub websocket_port: u16,
    pub websocket_password: String,
}

/// Get the standard OBS config directory (%APPDATA%/obs-studio on Windows).
fn obs_config_dir() -> Result<std::path::PathBuf, String> {
    dirs::config_dir()
        .map(|d| d.join("obs-studio"))
        .ok_or_else(|| "Cannot find OBS config directory".into())
}

/// Generate a Peppi profile and scene collection inside the standard OBS config.
///
/// Writes into `%APPDATA%/obs-studio/` so OBS can find its data files (locale,
/// themes) normally. The Peppi profile/scenes are selected at launch via
/// `--profile Peppi --collection Peppi`.
pub fn generate_portable_config(config_dir: &Path, settings: &ObsSettings) -> Result<(), String> {
    let obs_config = obs_config_dir()?;
    let ws_config = obs_config.join("plugin_config").join("obs-websocket");
    let profile_dir = obs_config.join("basic").join("profiles").join("Peppi");
    let scenes_dir = obs_config.join("basic").join("scenes");

    for dir in [&ws_config, &profile_dir, &scenes_dir] {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create config dir: {e}"))?;
    }

    // WebSocket config (shared across profiles)
    let ws_json = serde_json::json!({
        "server_enabled": true,
        "server_port": settings.websocket_port,
        "server_password": settings.websocket_password,
        "alerts_enabled": false,
        "first_load": false
    });
    std::fs::write(
        ws_config.join("config.json"),
        serde_json::to_string_pretty(&ws_json).unwrap(),
    )
    .map_err(|e| format!("Failed to write ws config: {e}"))?;

    // Profile basic.ini
    write_profile_ini(&profile_dir, settings)?;

    // Scene collection
    write_scene_config(&scenes_dir)?;

    log::info!("Generated Peppi OBS profile at {:?}", profile_dir);
    Ok(())
}

fn write_profile_ini(profile_dir: &Path, settings: &ObsSettings) -> Result<(), String> {
    let (rec_quality, vbitrate, output_cx, output_cy) = match settings.quality {
        RecordingQuality::Low => ("Stream", 2000, 640, 360),
        RecordingQuality::Medium => ("Stream", 8000, 1280, 720),
        RecordingQuality::High => ("HQ", 18000, 1920, 1080),
        RecordingQuality::Ultra => ("HQ", 35000, 0, 0), // 0 = native
    };

    let recording_path_escaped = settings.recording_path.replace('\\', "/");

    let mut ini = format!(
        "[SimpleOutput]\n\
        RecQuality={rec_quality}\n\
        VBitrate={vbitrate}\n\
        RecFormat2=mp4\n\
        RecEncoder=x264\n\
        FilePath={recording_path_escaped}\n\
        \n\
        [Output]\n\
        Mode=Simple\n\
        \n\
        [Video]\n\
        BaseCX=1920\n\
        BaseCY=1080\n"
    );

    if output_cx > 0 {
        ini.push_str(&format!("OutputCX={output_cx}\nOutputCY={output_cy}\n"));
    }
    ini.push_str("FPSCommon=60\n");

    std::fs::write(profile_dir.join("basic.ini"), ini)
        .map_err(|e| format!("Failed to write basic.ini: {e}"))
}

fn write_scene_config(scenes_dir: &Path) -> Result<(), String> {
    let capture_source = platform_capture_source();

    let scene = serde_json::json!({
        "current_scene": "Peppi Recording",
        "current_program_scene": "Peppi Recording",
        "scene_order": [{"name": "Peppi Recording"}],
        "sources": [
            {
                "id": "scene",
                "name": "Peppi Recording",
                "settings": {},
                "mixers": 0,
                "filters": []
            },
            capture_source
        ],
        "groups": [],
        "name": "Peppi"
    });

    std::fs::write(
        scenes_dir.join("Peppi.json"),
        serde_json::to_string_pretty(&scene).unwrap(),
    )
    .map_err(|e| format!("Failed to write scene config: {e}"))
}

/// Returns the platform-appropriate capture source for OBS scenes.
/// Windows: game_capture (any_fullscreen) — hooks into the running game directly.
/// macOS: display_capture (ScreenCaptureKit) — captures the display showing the game.
fn platform_capture_source() -> serde_json::Value {
    #[cfg(target_os = "windows")]
    {
        serde_json::json!({
            "id": "game_capture",
            "name": "Game Capture",
            "settings": {
                "capture_mode": "any_fullscreen",
                "capture_cursor": false,
                "anti_cheat_hook": true,
                "hook_rate": 1
            },
            "mixers": 0,
            "filters": []
        })
    }

    #[cfg(target_os = "macos")]
    {
        serde_json::json!({
            "id": "display_capture",
            "name": "Display Capture",
            "settings": {
                "type": 0,
                "show_cursor": false,
                "show_empty_names": false
            },
            "mixers": 0,
            "filters": []
        })
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        serde_json::json!({
            "id": "xcomposite_input",
            "name": "Game Capture",
            "settings": {},
            "mixers": 0,
            "filters": []
        })
    }
}

/// Update just the recording output path in the profile.
pub fn update_recording_path(_config_dir: &Path, output_dir: &str) -> Result<(), String> {
    let ini_path = obs_config_dir()?
        .join("basic")
        .join("profiles")
        .join("Peppi")
        .join("basic.ini");

    if !ini_path.exists() {
        return Err("Profile basic.ini not found".to_string());
    }

    let content = std::fs::read_to_string(&ini_path)
        .map_err(|e| format!("Failed to read basic.ini: {e}"))?;

    let path_escaped = output_dir.replace('\\', "/");
    let updated = content
        .lines()
        .map(|line| {
            if line.starts_with("FilePath=") {
                format!("FilePath={path_escaped}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    std::fs::write(&ini_path, updated)
        .map_err(|e| format!("Failed to write basic.ini: {e}"))
}

/// Update quality settings in the profile.
pub fn update_quality(_config_dir: &Path, quality: &RecordingQuality) -> Result<(), String> {
    let ini_path = obs_config_dir()?
        .join("basic")
        .join("profiles")
        .join("Peppi")
        .join("basic.ini");

    if !ini_path.exists() {
        return Err("Profile basic.ini not found".to_string());
    }

    let (rec_quality, vbitrate, output_cx, output_cy) = match quality {
        RecordingQuality::Low => ("Stream", 2000, 640, 360),
        RecordingQuality::Medium => ("Stream", 8000, 1280, 720),
        RecordingQuality::High => ("HQ", 18000, 1920, 1080),
        RecordingQuality::Ultra => ("HQ", 35000, 0, 0),
    };

    let content = std::fs::read_to_string(&ini_path)
        .map_err(|e| format!("Failed to read basic.ini: {e}"))?;

    let updated = content
        .lines()
        .map(|line| {
            if line.starts_with("RecQuality=") {
                format!("RecQuality={rec_quality}")
            } else if line.starts_with("VBitrate=") {
                format!("VBitrate={vbitrate}")
            } else if line.starts_with("OutputCX=") {
                if output_cx > 0 {
                    format!("OutputCX={output_cx}")
                } else {
                    line.to_string()
                }
            } else if line.starts_with("OutputCY=") {
                if output_cy > 0 {
                    format!("OutputCY={output_cy}")
                } else {
                    line.to_string()
                }
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    std::fs::write(&ini_path, updated)
        .map_err(|e| format!("Failed to write basic.ini: {e}"))
}
