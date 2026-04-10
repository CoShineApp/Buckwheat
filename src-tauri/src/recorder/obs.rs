use super::{Error, Recorder, RecordingQuality};
use crate::obs::{config, connection, install, process};
use std::path::PathBuf;
use std::process::Child;
use std::time::Duration;

const MANAGED_WS_PORT: u16 = 4456;

/// OBS operating mode.
pub enum ObsMode {
    /// Peppi manages its own portable OBS instance.
    Managed,
    /// User connects Peppi to their existing OBS.
    Connect {
        port: u16,
        password: Option<String>,
    },
}

/// Recorder implementation that controls OBS via websocket.
pub struct ObsRecorder {
    obs_exe: Option<PathBuf>,
    config_dir: Option<PathBuf>,
    process: Option<Child>,
    connection: Option<connection::ObsConnection>,
    is_recording: bool,
    websocket_port: u16,
    websocket_password: String,
    mode: ObsMode,
}

impl ObsRecorder {
    /// Create a recorder in managed mode (Peppi controls OBS lifecycle).
    pub fn managed_mode() -> Self {
        Self {
            obs_exe: None,
            config_dir: None,
            process: None,
            connection: None,
            is_recording: false,
            websocket_port: MANAGED_WS_PORT,
            websocket_password: uuid::Uuid::new_v4().to_string(),
            mode: ObsMode::Managed,
        }
    }

    /// Create a recorder in connect mode (user's existing OBS).
    pub fn connect_mode(port: u16, password: Option<String>) -> Self {
        Self {
            obs_exe: None,
            config_dir: None,
            process: None,
            connection: None,
            is_recording: false,
            websocket_port: port,
            websocket_password: password.unwrap_or_default(),
            mode: ObsMode::Connect { port, password: None },
        }
    }

    fn get_config_dir() -> Result<PathBuf, Error> {
        let dir = dirs::data_local_dir()
            .ok_or_else(|| Error::InitializationError("Cannot find app data dir".into()))?
            .join("com.peppi.app")
            .join("obs-portable");
        Ok(dir)
    }

    fn ensure_managed_obs(&mut self) -> Result<(), Error> {
        // Find or install OBS
        if self.obs_exe.is_none() {
            let obs_exe = tokio::runtime::Handle::current()
                .block_on(install::ensure_obs_available())
                .map_err(|e| Error::InitializationError(format!("OBS setup failed: {e}")))?;
            self.obs_exe = Some(obs_exe);
        }

        // Generate config if needed
        if self.config_dir.is_none() {
            let config_dir = Self::get_config_dir()?;
            self.config_dir = Some(config_dir);
        }

        Ok(())
    }

    fn ensure_config_generated(
        &self,
        output_path: &str,
        quality: RecordingQuality,
    ) -> Result<(), Error> {
        let config_dir = self
            .config_dir
            .as_ref()
            .ok_or_else(|| Error::InitializationError("No config dir".into()))?;

        // Derive the recording directory from the output path
        let recording_dir = std::path::Path::new(output_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| output_path.to_string());

        let settings = config::ObsSettings {
            recording_path: recording_dir,
            quality,
            websocket_port: self.websocket_port,
            websocket_password: self.websocket_password.clone(),
        };

        config::generate_portable_config(config_dir, &settings)
            .map_err(|e| Error::InitializationError(format!("Config generation failed: {e}")))?;

        Ok(())
    }

    fn spawn_if_needed(&mut self) -> Result<(), Error> {
        // Check if already running
        if let Some(ref mut child) = self.process {
            if process::is_obs_running(child) {
                return Ok(());
            }
        }

        let obs_exe = self
            .obs_exe
            .as_ref()
            .ok_or_else(|| Error::InitializationError("OBS exe not found".into()))?;
        let config_dir = self
            .config_dir
            .as_ref()
            .ok_or_else(|| Error::InitializationError("Config dir not set".into()))?;

        let child = process::spawn_obs(obs_exe, config_dir)
            .map_err(|e| Error::InitializationError(e))?;
        self.process = Some(child);

        Ok(())
    }

    fn connect(&mut self) -> Result<(), Error> {
        let conn = tokio::runtime::Handle::current()
            .block_on(connection::wait_for_obs_ready(
                self.websocket_port,
                &self.websocket_password,
                Duration::from_secs(30),
            ))
            .map_err(|e| Error::InitializationError(format!("OBS connection failed: {e}")))?;
        self.connection = Some(conn);
        Ok(())
    }
}

impl Recorder for ObsRecorder {
    fn start_recording(
        &mut self,
        output_path: &str,
        quality: RecordingQuality,
    ) -> Result<(), Error> {
        if self.is_recording {
            return Err(Error::RecordingFailed("Already recording".into()));
        }

        match &self.mode {
            ObsMode::Managed => {
                self.ensure_managed_obs()?;
                self.ensure_config_generated(output_path, quality)?;
                self.spawn_if_needed()?;
                self.connect()?;
            }
            ObsMode::Connect { .. } => {
                self.connect()?;
            }
        }

        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| Error::RecordingFailed("No OBS connection".into()))?;

        tokio::runtime::Handle::current()
            .block_on(conn.start_recording())
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

        let output_path = tokio::runtime::Handle::current()
            .block_on(conn.stop_recording())
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
        if let Some(ref mut child) = self.process {
            log::info!("Shutting down managed OBS process");
            let _ = process::kill_obs(child);
        }
    }
}
