#![cfg_attr(
    all(target_os = "windows", feature = "real-recording"),
    allow(unexpected_cfgs)
)]

#[cfg(all(target_os = "windows", feature = "real-recording"))]
use super::{Error, Recorder};

#[cfg(all(target_os = "windows", feature = "real-recording"))]
use std::sync::{Arc, Mutex};

#[cfg(all(target_os = "windows", feature = "real-recording"))]
use windows_record::Recorder as WinRecorder;

#[cfg(all(target_os = "windows", feature = "real-recording"))]
const DEFAULT_FPS: u32 = 60;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
const FPS_DENOMINATOR: u32 = 1;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
const DEFAULT_WIDTH: u32 = 1920;
#[cfg(all(target_os = "windows", feature = "real-recording"))]
const DEFAULT_HEIGHT: u32 = 1080;

#[cfg(all(target_os = "windows", feature = "real-recording"))]
pub struct WindowsRecorder {
    is_recording: bool,
    recorder: Option<Arc<Mutex<WinRecorder>>>,
    output_path: Option<String>,
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
unsafe impl Send for WindowsRecorder {}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
impl WindowsRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: false,
            recorder: None,
            output_path: None,
        }
    }

    fn find_dolphin_process_name(&self) -> Result<&'static str, Error> {
        // Windows process names to search for (Slippi Dolphin)
        // Common process names for Dolphin emulator on Windows
        // We'll try "Dolphin.exe" first, which is the most common
        // The windows-record library uses process names to find windows
        
        // Note: In a real implementation, we might want to enumerate
        // processes and find the exact one, but for now we'll use
        // the most common Dolphin process name
        Ok("Dolphin.exe")
    }

    fn initialize_recorder(&mut self, output_path: &str) -> Result<(), Error> {
        let process_name = self.find_dolphin_process_name()?;
        log::info!("🎮 Targeting process: {}", process_name);

        // Ensure output directory exists
        if let Some(parent) = std::path::Path::new(output_path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|err| {
                    Error::RecordingFailed(format!("Failed to create output directory: {err}"))
                })?;
            }
        }

        // Configure the recorder using builder pattern
        let config = WinRecorder::builder()
            .fps(DEFAULT_FPS, FPS_DENOMINATOR)
            .input_dimensions(DEFAULT_WIDTH, DEFAULT_HEIGHT)
            .output_dimensions(DEFAULT_WIDTH, DEFAULT_HEIGHT)
            .capture_audio(false) // Match macOS implementation - no audio
            .output_path(output_path)
            .build();

        // Create recorder instance and target the Dolphin process
        let recorder = WinRecorder::new(config)
            .map_err(|e| {
                Error::InitializationError(format!("Failed to create recorder: {:?}", e))
            })?
            .with_process_name(process_name);

        self.recorder = Some(Arc::new(Mutex::new(recorder)));
        self.output_path = Some(output_path.to_string());

        Ok(())
    }
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
impl Recorder for WindowsRecorder {
    fn start_recording(&mut self, output_path: &str) -> Result<(), Error> {
        if self.is_recording {
            return Err(Error::RecordingFailed("Already recording".into()));
        }

        log::info!("🎥 [Windows] Starting recording to {}", output_path);
        self.initialize_recorder(output_path)?;

        if let Some(recorder_arc) = &self.recorder {
            let recorder = recorder_arc.lock().map_err(|e| {
                Error::InitializationError(format!("Failed to lock recorder: {}", e))
            })?;

            recorder.start_recording().map_err(|e| {
                Error::RecordingFailed(format!("Failed to start recording: {:?}", e))
            })?;
        } else {
            return Err(Error::InitializationError(
                "Recorder was not initialized".into(),
            ));
        }

        self.is_recording = true;
        log::info!("✅ [Windows] Recording started");
        Ok(())
    }

    fn stop_recording(&mut self) -> Result<String, Error> {
        if !self.is_recording {
            return Err(Error::RecordingFailed("Not recording".into()));
        }

        log::info!("⏹️  [Windows] Stopping recording");

        let stop_result = (|| -> Result<(), Error> {
            if let Some(recorder_arc) = &self.recorder {
                let recorder = recorder_arc.lock().map_err(|e| {
                    Error::RecordingFailed(format!("Failed to lock recorder: {}", e))
                })?;

                recorder.stop_recording().map_err(|e| {
                    Error::RecordingFailed(format!("Failed to stop recording: {:?}", e))
                })?;
            }

            Ok(())
        })();

        let output_path = self
            .output_path
            .clone()
            .unwrap_or_else(|| "recording.mp4".into());

        self.recorder = None;
        self.output_path = None;
        self.is_recording = false;

        stop_result?;

        log::info!("✅ [Windows] Recording saved to {}", output_path);
        Ok(output_path)
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }
}

#[cfg(all(target_os = "windows", feature = "real-recording"))]
impl Default for WindowsRecorder {
    fn default() -> Self {
        Self::new()
    }
}
