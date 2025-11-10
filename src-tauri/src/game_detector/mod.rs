pub mod slippi_paths;

use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use crate::commands::errors::Error;

pub struct GameDetector {
    slippi_path: PathBuf,
    watcher: Option<Box<dyn Watcher + Send>>,
}

impl GameDetector {
    pub fn new(slippi_path: PathBuf) -> Self {
        Self {
            slippi_path,
            watcher: None,
        }
    }

    pub fn start_watching(&mut self) -> Result<(), Error> {
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if let EventKind::Create(_) = event.kind {
                        for path in event.paths {
                            if let Some(ext) = path.extension() {
                                if ext == "slp" {
                                    log::info!("🎮 New Slippi replay detected: {:?}", path);
                                    // TODO: Implement automatic recording triggering
                                    // Need to test if .slp files are created BEFORE game finishes
                                    // For now, manual recording via start_recording/stop_recording commands
                                }
                            }
                        }
                    }
                }
                Err(e) => log::error!("❌ Watch error: {:?}", e),
            }
        })
        .map_err(|e| Error::WatchError(e.to_string()))?;

        watcher
            .watch(&self.slippi_path, RecursiveMode::NonRecursive)
            .map_err(|e| Error::WatchError(e.to_string()))?;

        self.watcher = Some(Box::new(watcher));
        log::info!("👀 Started watching for .slp files: {:?}", self.slippi_path);

        Ok(())
    }

    pub fn stop_watching(&mut self) {
        self.watcher = None;
        log::info!("⏹️  Stopped watching for .slp files");
    }
}

