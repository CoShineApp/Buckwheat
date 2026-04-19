//! Window detection and capture module
//!
//! This module handles detecting game windows (Slippi/Dolphin) and capturing
//! preview screenshots. Platform-specific implementations are in submodules.

mod types;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
mod capture;

// Re-export public types
pub use types::GameWindow;

// Re-export platform-specific implementations
#[cfg(target_os = "windows")]
pub use capture::capture_window_preview;
#[cfg(target_os = "windows")]
pub use windows::{
    check_game_window_open, check_window_by_process, find_all_windows, find_game_windows,
    highlight_window,
};

// Stubs for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub fn find_game_windows() -> Vec<GameWindow> {
    Vec::new()
}

#[cfg(not(target_os = "windows"))]
pub fn find_all_windows() -> Vec<GameWindow> {
    Vec::new()
}

#[cfg(not(target_os = "windows"))]
pub fn check_window_by_process(_process_name: &str) -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn check_game_window_open(_stored_id: Option<&str>) -> bool {
    false
}

#[cfg(not(target_os = "windows"))]
pub fn capture_window_preview(_identifier: &str) -> Result<Vec<u8>, String> {
    Err("Window capture not supported on this platform".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn highlight_window(_process_id: u32) -> Result<(), String> {
    Err("Window highlighting not supported on this platform".to_string())
}

