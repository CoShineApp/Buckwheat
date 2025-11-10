#[cfg(target_os = "windows")]
use super::{Recorder, Error};

#[cfg(target_os = "windows")]
pub struct WindowsRecorder {
    is_recording: bool,
}

#[cfg(target_os = "windows")]
impl WindowsRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: false,
        }
    }
}

#[cfg(target_os = "windows")]
impl Recorder for WindowsRecorder {
    fn start_recording(&mut self, _output_path: &str) -> Result<(), Error> {
        // TODO: Implement with windows-record crate
        Err(Error::InitializationError(
            "Windows recording not yet implemented".to_string(),
        ))
    }

    fn stop_recording(&mut self) -> Result<String, Error> {
        Err(Error::RecordingFailed(
            "Windows recording not yet implemented".to_string(),
        ))
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

