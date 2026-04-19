use super::{Error, Recorder, RecordingQuality};
use crate::obs::connection::GameWindowInfo;
use crate::obs::{config, connection, install, process};
use std::process::Child;
use std::time::Duration;

/// Recorder that controls OBS via websocket.
///
/// Auto-detect mode: tries to connect to an already-running OBS first.
/// If nothing is listening, spawns a managed OBS instance with a Peppi
/// profile, then connects.
pub struct ObsRecorder {
    /// Managed OBS child process (None if user's own OBS is used).
    managed_process: Option<Child>,
    connection: Option<connection::ObsConnection>,
    is_recording: bool,
    websocket_port: u16,
    websocket_password: String,
    /// Window to tell OBS to capture. None = use Slippi default.
    target_window: Option<GameWindowInfo>,
    /// Scene collection that was active before Peppi switched away from it.
    /// Restored on stop_recording.
    saved_collection: Option<String>,
}

impl ObsRecorder {
    pub fn new(port: u16, password: String) -> Self {
        Self {
            managed_process: None,
            connection: None,
            is_recording: false,
            websocket_port: port,
            websocket_password: password,
            target_window: None,
            saved_collection: None,
        }
    }


    /// Try to connect to an already-running OBS, or spawn one if needed (sync, for trait use).
    pub fn ensure_connected(&mut self) -> Result<(), Error> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.ensure_connected_async())
        })
    }

    /// Async version of ensure_connected — safe to call directly from async contexts.
    pub async fn ensure_connected_async(&mut self) -> Result<(), Error> {
        if self.connection.is_some() {
            return Ok(());
        }

        // First, try connecting to an already-running OBS.
        if let Ok(conn) = connection::ObsConnection::connect(
            self.websocket_port,
            &self.websocket_password,
        )
        .await
        {
            log::info!("Connected to existing OBS on port {}", self.websocket_port);
            self.connection = Some(conn);
            return Ok(());
        }

        // Nobody home — spawn a managed OBS instance.
        log::info!(
            "No OBS detected on port {}, launching managed instance",
            self.websocket_port
        );
        self.spawn_managed_obs_async().await?;

        let conn = connection::wait_for_obs_ready(
            self.websocket_port,
            &self.websocket_password,
            Duration::from_secs(30),
        )
        .await
        .map_err(|e| {
            Error::InitializationError(format!(
                "OBS launched but websocket connection failed: {e}"
            ))
        })?;

        self.connection = Some(conn);
        Ok(())
    }

    /// Find OBS, generate a Peppi profile, and spawn it (async).
    async fn spawn_managed_obs_async(&mut self) -> Result<(), Error> {
        let obs_exe = install::ensure_obs_available()
            .await
            .map_err(|e| Error::InitializationError(format!("OBS not found: {e}")))?;

        // Generate Peppi profile + websocket config in the standard OBS config dir
        let config_dir = dirs::data_local_dir()
            .ok_or_else(|| Error::InitializationError("Cannot find app data dir".into()))?
            .join("com.peppi.app")
            .join("obs-config");

        let settings = config::ObsSettings {
            recording_path: std::env::temp_dir().to_string_lossy().to_string(),
            quality: RecordingQuality::High,
            websocket_port: self.websocket_port,
            websocket_password: self.websocket_password.clone(),
        };
        config::generate_portable_config(&config_dir, &settings)
            .map_err(|e| Error::InitializationError(format!("Config generation failed: {e}")))?;

        // Spawn OBS
        let child = process::spawn_obs(&obs_exe, &config_dir)
            .map_err(|e| Error::InitializationError(e))?;
        self.managed_process = Some(child);

        Ok(())
    }

    /// Test that we can reach OBS. Used by the "Test Connection" button.
    pub fn test_connection(port: u16, password: &str) -> Result<(), Error> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                connection::ObsConnection::connect(port, password)
                    .await
                    .map_err(|e| {
                        Error::InitializationError(format!(
                            "Cannot connect to OBS on port {port}: {e}"
                        ))
                    })?;
                Ok(())
            })
        })
    }
}

impl Recorder for ObsRecorder {
    fn start_recording(
        &mut self,
        output_path: &str,
        _quality: RecordingQuality,
    ) -> Result<(), Error> {
        if self.is_recording {
            return Err(Error::RecordingFailed("Already recording".into()));
        }

        self.ensure_connected()?;

        let window = self
            .target_window
            .clone()
            .unwrap_or_else(GameWindowInfo::default_slippi);

        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| Error::RecordingFailed("No OBS connection".into()))?;

        let recording_dir = std::path::Path::new(output_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| output_path.to_string());

        // Full start flow: save current collection → switch to Peppi (creating
        // if needed) → set capture window → size canvas to window → set record
        // dir → start.
        let saved_collection = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Save current collection (so we can restore it on stop)
                let current = conn.current_scene_collection().await?;

                // If we're already in Peppi, don't record the "previous" as Peppi —
                // otherwise we'd never switch back to something useful.
                let saved = if current == "Peppi" { None } else { Some(current) };

                // Ensure Peppi collection/scene/source exist and switch to it
                conn.ensure_peppi_setup(&window).await?;

                // Lock to 60fps regardless of OBS's prior setting.
                if let Err(e) = conn.set_fps_60().await {
                    log::warn!("Could not force 60fps: {e}");
                }

                // Match OBS canvas to the window's exact size so the recording is
                // the window's pixels 1:1 (no letterboxing, no off-centering).
                if let (Some(w), Some(h)) = (window.width, window.height) {
                    if let Err(e) = conn.set_canvas_to_window(w, h).await {
                        log::warn!("Could not resize OBS canvas to {w}x{h}: {e}");
                    }
                }

                // Set output directory and start recording
                conn.set_record_directory(&recording_dir).await?;
                conn.start_recording().await?;

                Ok::<_, String>(saved)
            })
        })
        .map_err(|e| Error::RecordingFailed(e))?;

        self.saved_collection = saved_collection;
        self.is_recording = true;
        log::info!("OBS recording started: {}", output_path);
        Ok(())
    }

    fn stop_recording(&mut self) -> Result<String, Error> {
        if !self.is_recording {
            return Err(Error::RecordingFailed("Not currently recording".into()));
        }

        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| Error::RecordingFailed("No OBS connection".into()))?;

        let saved = self.saved_collection.take();

        let output_path = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let path = conn.stop_recording().await?;

                // Restore the user's previous scene collection (best-effort)
                if let Some(name) = saved.as_deref() {
                    if let Err(e) = conn.set_scene_collection(name).await {
                        log::warn!("Failed to restore scene collection '{name}': {e}");
                    }
                }

                Ok::<_, String>(path)
            })
        })
        .map_err(|e| Error::RecordingFailed(e))?;

        self.is_recording = false;
        log::info!("OBS recording stopped: {}", output_path);
        Ok(output_path)
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }

    fn set_target_window(&mut self, window: Option<GameWindowInfo>) {
        self.target_window = window;
    }
}

impl Drop for ObsRecorder {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.managed_process {
            log::info!("Shutting down managed OBS process");
            let _ = process::kill_obs(child);
        }
    }
}
