#[cfg(target_os = "windows")]
use super::{Error, Recorder};

#[cfg(target_os = "windows")]
use windows_record::{Recorder as WinRecorder, RecorderConfig};

#[cfg(target_os = "windows")]
pub struct WindowsRecorder {
    is_recording: bool,
    recorder: Option<WinRecorder>,
    output_path: Option<String>,
}

#[cfg(target_os = "windows")]
impl WindowsRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: false,
            recorder: None,
            output_path: None,
        }
    }

    fn initialize_recorder(&mut self, output_path: &str, window_name: &str) -> Result<(), Error> {
        // Create recorder config with windows-record builder
        let config = WinRecorder::builder()
            .fps(30, 1)
            .output_dimensions(1920, 1080)
            .capture_audio(true)
            .output_path(output_path)
            .build();

        // Create the recorder
        let recorder = WinRecorder::new(config)
            .map_err(|e| Error::InitializationError(format!("Failed to create recorder: {}", e)))?
            .with_process_name(window_name);

        self.recorder = Some(recorder);
        self.output_path = Some(output_path.to_string());

        Ok(())
    }
}

#[cfg(target_os = "windows")]
impl Recorder for WindowsRecorder {
    fn start_recording(&mut self, output_path: &str) -> Result<(), Error> {
        if self.is_recording {
            return Err(Error::RecordingFailed("Already recording".to_string()));
        }

        log::info!("🎥 [Windows] Starting recording to: {}", output_path);

        // Initialize the recorder with Dolphin window
        // TODO: Make window name configurable or detect automatically
        self.initialize_recorder(output_path, "Dolphin")?;

        // Start the recording
        if let Some(ref recorder) = self.recorder {
            recorder.start_recording().map_err(|e| {
                Error::RecordingFailed(format!("Failed to start recording: {}", e))
            })?;
        } else {
            return Err(Error::InitializationError(
                "Recorder not initialized".to_string(),
            ));
        }

        self.is_recording = true;
        log::info!("✅ [Windows] Recording started successfully");

        Ok(())
    }

    fn stop_recording(&mut self) -> Result<String, Error> {
        if !self.is_recording {
            return Err(Error::RecordingFailed("Not currently recording".to_string()));
        }

        log::info!("⏹️  [Windows] Stopping recording...");

        // Stop the recording
        if let Some(ref recorder) = self.recorder {
            recorder.stop_recording().map_err(|e| {
                Error::RecordingFailed(format!("Failed to stop recording: {}", e))
            })?;
        }

        self.is_recording = false;

        let output_path = self
            .output_path
            .clone()
            .unwrap_or_else(|| "unknown.mp4".to_string());

        self.recorder = None;
        self.output_path = None;

        log::info!("✅ [Windows] Recording stopped. Saved to: {}", output_path);

        Ok(output_path)
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }
}

#[cfg(target_os = "windows")]
impl Default for WindowsRecorder {
    fn default() -> Self {
        Self::new()
    }
}

