//! SQLite database for persistent metadata caching
//!
//! This module provides fast, persistent storage for Slippi replay metadata
//! to avoid re-parsing files on every application startup.

mod schema;
mod recordings;

pub use recordings::{
    // Recording operations
    get_all_recordings, get_recordings_paginated, get_recording_by_video_path, 
    upsert_recording, delete_recording, get_cached_video_paths,
    // Game stats operations
    upsert_game_stats, get_game_stats_by_id,
    // Player stats operations
    upsert_player_stats, get_player_stats_by_recording, get_aggregated_player_stats,
    // Types
    RecordingRow, GameStatsRow, RecordingWithStats, PlayerStatsRow,
    AggregatedPlayerStats, StatsFilter,
};

use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

/// Database connection wrapper for thread-safe access
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Open or create a database at the specified path
    pub fn open(path: &PathBuf) -> Result<Self, rusqlite::Error> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        let conn = Connection::open(path)?;
        
        // Enable WAL mode for better concurrent access
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    
    /// Initialize the database schema
    pub fn init(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        schema::init_database(&conn)
    }
    
    /// Get a reference to the connection (for operations)
    pub fn connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }
}

/// Get the default database path (in app data directory)
pub fn get_database_path(app: &tauri::AppHandle) -> PathBuf {
    use tauri::Manager;
    
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("peppi.db")
}

