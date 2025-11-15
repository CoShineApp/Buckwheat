pub mod mock;

#[cfg(all(target_os = "windows", feature = "real-recording"))]
pub mod windows;

#[cfg(all(target_os = "macos", feature = "real-recording"))]
pub mod macos;

use crate::commands::errors::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingQuality {
    Low,
    Medium,
    High,
    Ultra,
}

impl RecordingQuality {
    /// Get the bitrate in bits per second for this quality level
    pub fn bitrate(&self) -> u32 {
        match self {
            RecordingQuality::Low => 5_000_000,     // 5 Mbps - good for 720p
            RecordingQuality::Medium => 10_000_000, // 10 Mbps - good for 1080p
            RecordingQuality::High => 18_000_000,   // 18 Mbps - excellent for 1080p
            RecordingQuality::Ultra => 35_000_000,  // 35 Mbps - excellent for 1440p+
        }
    }
}

impl Default for RecordingQuality {
    fn default() -> Self {
        RecordingQuality::High
    }
}

pub trait Recorder {
    fn start_recording(
        &mut self,
        output_path: &str,
        quality: RecordingQuality,
    ) -> Result<(), Error>;
    fn stop_recording(&mut self) -> Result<String, Error>;
    fn is_recording(&self) -> bool;
}

pub fn get_recorder() -> Box<dyn Recorder + Send> {
    #[cfg(all(target_os = "macos", feature = "real-recording"))]
    {
        log::info!(
            "🍎 Initializing MacOS recorder with screencapturekit-rs (real-recording enabled)"
        );
        Box::new(macos::MacOSRecorder::new())
    }

    #[cfg(all(target_os = "windows", feature = "real-recording"))]
    {
        log::info!("🪟 Initializing Windows recorder with windows-record (real-recording enabled)");
        Box::new(windows::WindowsRecorder::new())
    }

    #[cfg(not(feature = "real-recording"))]
    {
        log::info!("🧪 Initializing mock recorder (dev mode - real-recording disabled)");
        Box::new(mock::MockRecorder::new())
    }
}
