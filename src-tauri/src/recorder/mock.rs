use super::{Error, Recorder};
use std::time::Instant;

pub struct MockRecorder {
    is_recording: bool,
    start_time: Option<Instant>,
    output_path: Option<String>,
}

impl MockRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: false,
            start_time: None,
            output_path: None,
        }
    }
}

impl Recorder for MockRecorder {
    fn start_recording(
        &mut self,
        output_path: &str,
        quality: super::RecordingQuality,
    ) -> Result<(), Error> {
        if self.is_recording {
            return Err(Error::RecordingFailed("Already recording".to_string()));
        }

        println!(
            "🎥 [MOCK] Starting recording to: {} with {:?} quality (bitrate: {} Mbps)",
            output_path,
            quality,
            quality.bitrate() / 1_000_000
        );
        self.is_recording = true;
        self.start_time = Some(Instant::now());
        self.output_path = Some(output_path.to_string());

        Ok(())
    }

    fn stop_recording(&mut self) -> Result<String, Error> {
        if !self.is_recording {
            return Err(Error::RecordingFailed(
                "Not currently recording".to_string(),
            ));
        }

        let duration = self
            .start_time
            .map(|start| start.elapsed().as_secs())
            .unwrap_or(0);

        let output_path = self
            .output_path
            .clone()
            .unwrap_or_else(|| "unknown.mp4".to_string());

        println!(
            "⏹️  [MOCK] Stopped recording. Duration: {}s. Saved to: {}",
            duration, output_path
        );

        self.is_recording = false;
        self.start_time = None;

        Ok(output_path)
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }
}

impl Default for MockRecorder {
    fn default() -> Self {
        Self::new()
    }
}
