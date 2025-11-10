pub mod mock;

#[cfg(target_os = "windows")]
pub mod windows;

use crate::commands::errors::Error;

pub trait Recorder {
    fn start_recording(&mut self, output_path: &str) -> Result<(), Error>;
    fn stop_recording(&mut self) -> Result<String, Error>;
    fn is_recording(&self) -> bool;
}

pub fn get_recorder() -> Box<dyn Recorder + Send> {
    #[cfg(target_os = "windows")]
    {
        Box::new(mock::MockRecorder::new())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Box::new(mock::MockRecorder::new())
    }
}

