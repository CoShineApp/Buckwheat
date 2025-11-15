pub mod slippi_paths;

use crate::commands::errors::Error;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

pub struct GameDetector {
    slippi_path: PathBuf,
    watcher: Option<Box<dyn Watcher + Send>>,
    app_handle: Option<AppHandle>,
}

impl GameDetector {
    pub fn new(slippi_path: PathBuf) -> Self {
        Self {
            slippi_path,
            watcher: None,
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }

    pub fn start_watching(&mut self) -> Result<(), Error> {
        let app_handle = self.app_handle.clone();
        let watch_path = self.slippi_path.clone();

        log::info!("🔧 Setting up file watcher for path: {:?}", watch_path);
        log::info!("🔧 Path exists: {}", watch_path.exists());
        log::info!("🔧 Path is directory: {}", watch_path.is_dir());

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    log::debug!("📂 File system event received: {:?}", event.kind);
                    log::debug!("📂 Event paths: {:?}", event.paths);

                    // Log all events for debugging
                    match event.kind {
                        EventKind::Create(_) => log::info!("✅ CREATE event detected"),
                        EventKind::Modify(_) => log::debug!("📝 MODIFY event detected"),
                        EventKind::Remove(_) => log::debug!("🗑️  REMOVE event detected"),
                        EventKind::Access(_) => log::debug!("👁️  ACCESS event detected"),
                        _ => log::debug!("❓ OTHER event: {:?}", event.kind),
                    }

                    // Handle CREATE events (new game starting)
                    if let EventKind::Create(_) = event.kind {
                        for path in &event.paths {
                            log::info!("🔍 Examining created file: {:?}", path);

                            if let Some(ext) = path.extension() {
                                log::info!("📎 File extension: {:?}", ext);

                                if ext == "slp" {
                                    log::info!("🎮 New Slippi replay detected: {:?}", path);

                                    // Emit event to trigger auto-recording
                                    if let Some(handle) = &app_handle {
                                        let path_string = path.to_string_lossy().to_string();
                                        log::info!(
                                            "📤 Emitting slp-file-created event with path: {}",
                                            path_string
                                        );

                                        match handle.emit("slp-file-created", path_string.clone()) {
                                            Ok(_) => log::info!("✅ Event emitted successfully"),
                                            Err(e) => log::error!(
                                                "❌ Failed to emit slp-file-created event: {:?}",
                                                e
                                            ),
                                        }
                                    } else {
                                        log::error!("❌ App handle is None, cannot emit event");
                                    }
                                } else {
                                    log::debug!("⏭️  Skipping non-slp file: {:?}", ext);
                                }
                            } else {
                                log::debug!("⏭️  File has no extension: {:?}", path);
                            }
                        }
                    }

                    // Handle MODIFY events (game in progress)
                    if let EventKind::Modify(_) = event.kind {
                        for path in &event.paths {
                            if let Some(ext) = path.extension() {
                                if ext == "slp" {
                                    // Emit event to update last modification time
                                    if let Some(handle) = &app_handle {
                                        let path_string = path.to_string_lossy().to_string();
                                        log::debug!("📝 .slp file modified: {}", path_string);

                                        if let Err(e) =
                                            handle.emit("slp-file-modified", path_string)
                                        {
                                            log::error!(
                                                "❌ Failed to emit slp-file-modified event: {:?}",
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => log::error!("❌ Watch error: {:?}", e),
            }
        })
        .map_err(|e| Error::WatchError(e.to_string()))?;

        log::info!("🔧 Calling watcher.watch() with RecursiveMode::Recursive");
        watcher
            .watch(&self.slippi_path, RecursiveMode::Recursive)
            .map_err(|e| Error::WatchError(e.to_string()))?;

        self.watcher = Some(Box::new(watcher));
        log::info!("👀 Started watching for .slp files: {:?}", self.slippi_path);
        log::info!("✅ File watcher is now active and monitoring for changes");

        Ok(())
    }

    pub fn stop_watching(&mut self) {
        self.watcher = None;
        log::info!("⏹️  Stopped watching for .slp files");
    }
}
