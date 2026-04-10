use super::{Error, Recorder, RecordingQuality};
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
}

impl ObsRecorder {
    pub fn new(port: u16, password: String) -> Self {
        Self {
            managed_process: None,
            connection: None,
            is_recording: false,
            websocket_port: port,
            websocket_password: password,
        }
    }

    /// Try to connect to an already-running OBS, or spawn one if needed.
    fn ensure_connected(&mut self) -> Result<(), Error> {
        if self.connection.is_some() {
            return Ok(());
        }

        // First, try connecting to an already-running OBS (quick timeout).
        let quick = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(
                connection::ObsConnection::connect(
                    self.websocket_port,
                    &self.websocket_password,
                ),
            )
        });

        if let Ok(conn) = quick {
            log::info!(
                "Connected to existing OBS on port {}",
                self.websocket_port
            );
            self.connection = Some(conn);
            return Ok(());
        }

        // Nobody home — spawn a managed OBS instance.
        log::info!("No OBS detected on port {}, launching managed instance", self.websocket_port);
        self.spawn_managed_obs()?;

        // Wait for the freshly-spawned OBS to be ready.
        let conn = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(connection::wait_for_obs_ready(
                self.websocket_port,
                &self.websocket_password,
                Duration::from_secs(30),
            ))
        })
        .map_err(|e| {
            Error::InitializationError(format!("OBS launched but websocket connection failed: {e}"))
        })?;

        self.connection = Some(conn);
        Ok(())
    }

    /// Find OBS, generate a Peppi profile, and spawn it.
    fn spawn_managed_obs(&mut self) -> Result<(), Error> {
        // Find OBS installation
        let obs_exe = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(install::ensure_obs_available())
        })
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

        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| Error::RecordingFailed("No OBS connection".into()))?;

        // Point OBS output to Peppi's recording directory before starting.
        let recording_dir = std::path::Path::new(output_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| output_path.to_string());

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(conn.set_record_directory(&recording_dir))
        })
        .map_err(|e| Error::RecordingFailed(e))?;

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(conn.start_recording())
        })
        .map_err(|e| Error::RecordingFailed(e))?;

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

        let output_path = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(conn.stop_recording())
        })
        .map_err(|e| Error::RecordingFailed(e))?;

        self.is_recording = false;
        log::info!("OBS recording stopped: {}", output_path);
        Ok(output_path)
    }

    fn is_recording(&self) -> bool {
        self.is_recording
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
