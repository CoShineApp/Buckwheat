pub mod slippi_paths;

use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
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
        let (tx, _rx) = channel();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if let EventKind::Create(_) = event.kind {
                        for path in event.paths {
                            if let Some(ext) = path.extension() {
                                if ext == "slp" {
                                    println!("New Slippi replay detected: {:?}", path);
                                    tx.send(path).ok();
                                }
                            }
                        }
                    }
                }
                Err(e) => println!("Watch error: {:?}", e),
            }
        })
        .map_err(|e| Error::WatchError(e.to_string()))?;

        watcher
            .watch(&self.slippi_path, RecursiveMode::NonRecursive)
            .map_err(|e| Error::WatchError(e.to_string()))?;

        self.watcher = Some(Box::new(watcher));
        println!("Started watching: {:?}", self.slippi_path);

        Ok(())
    }

    pub fn stop_watching(&mut self) {
        self.watcher = None;
        println!("Stopped watching");
    }
}

