pub mod mock;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

use crate::commands::errors::Error;

pub trait Recorder {
    fn start_recording(&mut self, output_path: &str) -> Result<(), Error>;
    fn stop_recording(&mut self) -> Result<String, Error>;
    fn is_recording(&self) -> bool;
}

pub fn get_recorder() -> Box<dyn Recorder + Send> {
    #[cfg(target_os = "macos")]
    {
        log::info!("🍎 Initializing MacOS recorder with scap");
        Box::new(macos::MacOSRecorder::new())
    }

    #[cfg(target_os = "windows")]
    {
        log::info!("🪟 Initializing Windows recorder with windows-record");
        Box::new(windows::WindowsRecorder::new())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        log::info!("⚠️  Initializing mock recorder (unsupported platform)");
        Box::new(mock::MockRecorder::new())
    }
}

