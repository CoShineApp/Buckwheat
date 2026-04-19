use serde::{Deserialize, Serialize};
use std::time::Duration;

const PEPPI_COLLECTION: &str = "Peppi";
const PEPPI_SCENE: &str = "Peppi Recording";
const PEPPI_SOURCE: &str = "Slippi Dolphin";
const PEPPI_AUDIO_SOURCE: &str = "Desktop Audio";
const PEPPI_TARGET_FPS: u32 = 60;

/// Selected game window to tell OBS to capture.
/// Maps to OBS game_capture's "title:class:executable" format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameWindowInfo {
    pub title: String,
    pub class_name: String,
    pub process_name: String,
    /// Window width at selection time. Used to size OBS canvas 1:1.
    pub width: Option<u32>,
    /// Window height at selection time.
    pub height: Option<u32>,
}

impl GameWindowInfo {
    /// Build OBS's "title:class:executable" window matcher string.
    fn obs_window_string(&self) -> String {
        format!("{}:{}:{}", self.title, self.class_name, self.process_name)
    }

    /// Reasonable default if the user hasn't picked a window.
    pub fn default_slippi() -> Self {
        Self {
            title: "Slippi Dolphin".to_string(),
            class_name: "wxWindowNR".to_string(),
            process_name: "Slippi Dolphin.exe".to_string(),
            width: None,
            height: None,
        }
    }
}

/// Wrapper around an OBS websocket connection.
pub struct ObsConnection {
    client: obws::Client,
}

impl ObsConnection {
    /// Connect to OBS websocket.
    pub async fn connect(port: u16, password: &str) -> Result<Self, String> {
        let client = obws::Client::connect("127.0.0.1", port, Some(password))
            .await
            .map_err(|e| format!("Failed to connect to OBS: {e}"))?;
        Ok(Self { client })
    }

    /// Get the currently active scene collection name.
    pub async fn current_scene_collection(&self) -> Result<String, String> {
        self.client
            .scene_collections()
            .current()
            .await
            .map_err(|e| format!("Failed to get current scene collection: {e}"))
    }

    /// Switch OBS to a specific scene collection.
    pub async fn set_scene_collection(&self, name: &str) -> Result<(), String> {
        self.client
            .scene_collections()
            .set_current(name)
            .await
            .map_err(|e| format!("Failed to switch to scene collection {name}: {e}"))
    }

    /// Ensure the Peppi scene collection + scene + game_capture source exist,
    /// switch to that collection, and apply the given window target.
    pub async fn ensure_peppi_setup(
        &self,
        window: &GameWindowInfo,
    ) -> Result<(), String> {
        // Try switching to Peppi collection; create it if it doesn't exist yet.
        if let Err(_) = self
            .client
            .scene_collections()
            .set_current(PEPPI_COLLECTION)
            .await
        {
            log::info!("Peppi scene collection missing, creating it");
            self.client
                .scene_collections()
                .create(PEPPI_COLLECTION)
                .await
                .map_err(|e| {
                    format!("Failed to create Peppi scene collection: {e}")
                })?;
            // create() auto-switches to it
        }

        // OBS might still be finishing the switch — a brief delay helps.
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Ensure the Peppi Recording scene exists.
        let scenes = self
            .client
            .scenes()
            .list()
            .await
            .map_err(|e| format!("Failed to list scenes: {e}"))?;

        let scene_exists = scenes
            .scenes
            .iter()
            .any(|s| s.id.name == PEPPI_SCENE);

        if !scene_exists {
            log::info!("Creating {} scene", PEPPI_SCENE);
            self.client
                .scenes()
                .create(PEPPI_SCENE)
                .await
                .map_err(|e| format!("Failed to create Peppi scene: {e}"))?;
        }

        // Make it the active scene
        self.client
            .scenes()
            .set_current_program_scene(obws::requests::scenes::SceneId::Name(PEPPI_SCENE))
            .await
            .map_err(|e| format!("Failed to set current scene: {e}"))?;

        // Build source settings
        let settings = game_capture_settings(window);

        // Check if the Slippi Dolphin input exists; create it if not, else update settings.
        let input_exists = self
            .client
            .inputs()
            .list(None)
            .await
            .map(|inputs| inputs.iter().any(|i| i.id.name == PEPPI_SOURCE))
            .unwrap_or(false);

        if input_exists {
            self.client
                .inputs()
                .set_settings(obws::requests::inputs::SetSettings {
                    input: obws::requests::inputs::InputId::Name(PEPPI_SOURCE),
                    settings: &settings,
                    overlay: Some(true),
                })
                .await
                .map_err(|e| format!("Failed to update Slippi Dolphin source: {e}"))?;
        } else {
            log::info!("Creating {} source", PEPPI_SOURCE);
            self.client
                .inputs()
                .create(obws::requests::inputs::Create {
                    scene: obws::requests::scenes::SceneId::Name(PEPPI_SCENE),
                    input: PEPPI_SOURCE,
                    kind: "game_capture",
                    settings: Some(settings),
                    enabled: Some(true),
                })
                .await
                .map_err(|e| format!("Failed to create Slippi Dolphin source: {e}"))?;
        }

        // Ensure a desktop audio source exists so we actually record sound.
        // game_capture doesn't capture audio, so we add a separate WASAPI output
        // capture of the default playback device.
        let audio_exists = self
            .client
            .inputs()
            .list(None)
            .await
            .map(|inputs| inputs.iter().any(|i| i.id.name == PEPPI_AUDIO_SOURCE))
            .unwrap_or(false);

        if !audio_exists {
            log::info!("Creating {} source", PEPPI_AUDIO_SOURCE);
            let audio_settings = serde_json::json!({ "device_id": "default" });
            if let Err(e) = self
                .client
                .inputs()
                .create(obws::requests::inputs::Create {
                    scene: obws::requests::scenes::SceneId::Name(PEPPI_SCENE),
                    input: PEPPI_AUDIO_SOURCE,
                    kind: "wasapi_output_capture",
                    settings: Some(audio_settings),
                    enabled: Some(true),
                })
                .await
            {
                log::warn!("Failed to create desktop audio source: {e}");
            }
        }

        Ok(())
    }

    /// Lock OBS to 60fps. Called on every recording start so a user who
    /// fiddled with OBS's FPS setting doesn't end up with a 30fps recording.
    pub async fn set_fps_60(&self) -> Result<(), String> {
        self.client
            .config()
            .set_video_settings(obws::requests::config::SetVideoSettings {
                fps_numerator: Some(PEPPI_TARGET_FPS),
                fps_denominator: Some(1),
                base_width: None,
                base_height: None,
                output_width: None,
                output_height: None,
            })
            .await
            .map_err(|e| format!("Failed to set OBS FPS to {PEPPI_TARGET_FPS}: {e}"))
    }

    /// Set OBS base canvas and output resolution to match the capture window
    /// exactly, and lock FPS to 60. This way the recorded video is the window's
    /// pixels 1:1 (no letterboxing) at 60fps regardless of OBS's prior settings.
    ///
    /// Dimensions must be even (H.264 requires it) and at least 8x8 per OBS.
    pub async fn set_canvas_to_window(&self, width: u32, height: u32) -> Result<(), String> {
        // Round to even, clamp to OBS-safe minimums
        let w = ((width / 2) * 2).max(16);
        let h = ((height / 2) * 2).max(16);

        self.client
            .config()
            .set_video_settings(obws::requests::config::SetVideoSettings {
                fps_numerator: Some(PEPPI_TARGET_FPS),
                fps_denominator: Some(1),
                base_width: Some(w),
                base_height: Some(h),
                output_width: Some(w),
                output_height: Some(h),
            })
            .await
            .map_err(|e| {
                format!("Failed to set OBS video settings to {w}x{h}@{PEPPI_TARGET_FPS}fps: {e}")
            })
    }

    /// Set the directory OBS will write recordings to.
    pub async fn set_record_directory(&self, directory: &str) -> Result<(), String> {
        self.client
            .config()
            .set_record_directory(directory)
            .await
            .map_err(|e| format!("Failed to set OBS record directory: {e}"))
    }

    /// Start recording.
    pub async fn start_recording(&self) -> Result<(), String> {
        self.client
            .recording()
            .start()
            .await
            .map_err(|e| format!("Failed to start OBS recording: {e}"))
    }

    /// Stop recording and return the output file path.
    pub async fn stop_recording(&self) -> Result<String, String> {
        self.client
            .recording()
            .stop()
            .await
            .map_err(|e| format!("Failed to stop OBS recording: {e}"))
    }

    /// Check if OBS is currently recording.
    pub async fn is_recording(&self) -> Result<bool, String> {
        let status = self
            .client
            .recording()
            .status()
            .await
            .map_err(|e| format!("Failed to get OBS recording status: {e}"))?;
        Ok(status.active)
    }
}

/// Build the OBS game_capture source settings for a target window.
fn game_capture_settings(window: &GameWindowInfo) -> serde_json::Value {
    serde_json::json!({
        "capture_mode": "window",
        "window": window.obs_window_string(),
        "priority": 2, // match by executable (most stable)
        "capture_cursor": false,
        "allow_transparency": false,
        "limit_framerate": false,
        "capture_overlays": false,
        "anti_cheat_hook": true,
        "hook_rate": 1,
    })
}

/// Poll until OBS websocket is ready, with timeout.
pub async fn wait_for_obs_ready(
    port: u16,
    password: &str,
    timeout: Duration,
) -> Result<ObsConnection, String> {
    let start = std::time::Instant::now();
    let poll_interval = Duration::from_millis(500);

    loop {
        match ObsConnection::connect(port, password).await {
            Ok(conn) => {
                log::info!("Connected to OBS websocket on port {}", port);
                return Ok(conn);
            }
            Err(e) => {
                if start.elapsed() > timeout {
                    return Err(format!(
                        "Timed out waiting for OBS websocket after {:?}: {e}",
                        timeout
                    ));
                }
                log::debug!("OBS not ready yet, retrying in {:?}...", poll_interval);
                tokio::time::sleep(poll_interval).await;
            }
        }
    }
}
