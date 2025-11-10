pub mod mock;

#[cfg(all(target_os = "windows", feature = "real-recording"))]
pub mod windows;

#[cfg(all(target_os = "macos", feature = "real-recording"))]
pub mod macos;

use crate::commands::errors::Error;

pub trait Recorder {
    fn start_recording(&mut self, output_path: &str) -> Result<(), Error>;
    fn stop_recording(&mut self) -> Result<String, Error>;
    fn is_recording(&self) -> bool;
}

pub fn get_recorder() -> Box<dyn Recorder + Send> {
    #[cfg(all(target_os = "macos", feature = "real-recording"))]
    {
        log::info!("🍎 Initializing MacOS recorder with screencapturekit-rs (real-recording enabled)");
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

