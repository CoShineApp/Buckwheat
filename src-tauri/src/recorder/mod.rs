pub mod mock;
pub mod obs;

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
            RecordingQuality::Low => 2_000_000,     // 2 Mbps - good for 360p
            RecordingQuality::Medium => 8_000_000,  // 8 Mbps - good for 720p
            RecordingQuality::High => 18_000_000,   // 18 Mbps - excellent for 1080p
            RecordingQuality::Ultra => 35_000_000,  // 35 Mbps - excellent for native resolution
        }
    }

    /// Get the target output resolution (width, height) for this quality level.
    /// Returns None for Ultra quality (use native resolution).
    pub fn target_resolution(&self) -> Option<(u32, u32)> {
        match self {
            RecordingQuality::Low => Some((640, 360)),     // 360p
            RecordingQuality::Medium => Some((1280, 720)), // 720p
            RecordingQuality::High => Some((1920, 1080)),  // 1080p
            RecordingQuality::Ultra => None,               // Native resolution
        }
    }

    /// Calculate scaled dimensions while maintaining aspect ratio.
    /// Returns the target dimensions clamped to not exceed source, with even values for H.264.
    pub fn scale_dimensions(&self, source_width: u32, source_height: u32) -> (u32, u32) {
        match self.target_resolution() {
            Some((target_w, target_h)) => {
                // Calculate scale factor to fit within target while maintaining aspect ratio
                let scale_w = target_w as f64 / source_width as f64;
                let scale_h = target_h as f64 / source_height as f64;
                let scale = scale_w.min(scale_h).min(1.0); // Don't upscale

                let new_width = (source_width as f64 * scale) as u32;
                let new_height = (source_height as f64 * scale) as u32;

                // Ensure even dimensions for H.264 encoding
                let new_width = (new_width / 2) * 2;
                let new_height = (new_height / 2) * 2;

                // Minimum size
                (new_width.max(320), new_height.max(240))
            }
            None => {
                // Ultra: use source dimensions (ensure even)
                let w = (source_width / 2) * 2;
                let h = (source_height / 2) * 2;
                (w.max(320), h.max(240))
            }
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

    /// Optional: tell the recorder which window to capture. Default impl is a no-op.
    fn set_target_window(&mut self, _window: Option<crate::obs::connection::GameWindowInfo>) {}
}

/// Create a recorder that connects to the user's running OBS instance.
pub fn get_recorder(port: u16, password: String) -> Box<dyn Recorder + Send> {
    log::info!("Initializing OBS recorder (connecting to port {})", port);
    Box::new(obs::ObsRecorder::new(port, password))
}
