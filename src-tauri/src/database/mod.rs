// Local SQLite database for player stats storage

pub mod stats_store;

use crate::commands::errors::Error;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Database connection wrapper with thread-safe access
pub struct StatsDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl StatsDatabase {
    /// Create a new database connection and initialize schema
    pub fn new(db_path: PathBuf) -> Result<Self, Error> {
        log::info!("📊 Initializing stats database at: {:?}", db_path);
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::InitializationError(format!("Failed to create db directory: {}", e)))?;
        }
        
        let conn = Connection::open(&db_path)
            .map_err(|e| Error::InitializationError(format!("Failed to open database: {}", e)))?;
        
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        db.initialize_schema()?;
        
        log::info!("✅ Stats database initialized successfully");
        Ok(db)
    }
    
    /// Initialize database schema
    fn initialize_schema(&self) -> Result<(), Error> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS player_game_stats (
                id TEXT PRIMARY KEY,
                user_id TEXT,
                device_id TEXT NOT NULL,
                slp_file_path TEXT NOT NULL,
                recording_id TEXT NOT NULL,
                
                -- Game metadata
                game_date TEXT NOT NULL,
                stage_id INTEGER NOT NULL,
                game_duration_frames INTEGER NOT NULL,
                
                -- Player info
                player_port INTEGER NOT NULL,
                player_tag TEXT NOT NULL,
                character_id INTEGER NOT NULL,
                opponent_character_id INTEGER,
                
                -- L-Cancel stats
                l_cancel_hit INTEGER NOT NULL DEFAULT 0,
                l_cancel_missed INTEGER NOT NULL DEFAULT 0,
                
                -- Neutral & opening stats
                neutral_wins INTEGER NOT NULL DEFAULT 0,
                neutral_losses INTEGER NOT NULL DEFAULT 0,
                openings INTEGER NOT NULL DEFAULT 0,
                damage_per_opening REAL,
                openings_per_kill REAL,
                
                -- Kill stats
                kills INTEGER NOT NULL DEFAULT 0,
                deaths INTEGER NOT NULL DEFAULT 0,
                avg_kill_percent REAL,
                total_damage_dealt REAL NOT NULL DEFAULT 0,
                total_damage_taken REAL NOT NULL DEFAULT 0,
                
                -- Tech skill stats
                successful_techs INTEGER NOT NULL DEFAULT 0,
                missed_techs INTEGER NOT NULL DEFAULT 0,
                wavedash_count INTEGER NOT NULL DEFAULT 0,
                dashdance_count INTEGER NOT NULL DEFAULT 0,
                
                -- Input stats
                apm REAL NOT NULL DEFAULT 0,
                grab_attempts INTEGER NOT NULL DEFAULT 0,
                grab_success INTEGER NOT NULL DEFAULT 0,
                
                -- Metadata
                synced_to_cloud INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| Error::InitializationError(format!("Failed to create table: {}", e)))?;
        
        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_player_tag ON player_game_stats(player_tag)",
            [],
        )
        .map_err(|e| Error::InitializationError(format!("Failed to create index: {}", e)))?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_character ON player_game_stats(character_id)",
            [],
        )
        .map_err(|e| Error::InitializationError(format!("Failed to create index: {}", e)))?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_game_date ON player_game_stats(game_date DESC)",
            [],
        )
        .map_err(|e| Error::InitializationError(format!("Failed to create index: {}", e)))?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_recording_id ON player_game_stats(recording_id)",
            [],
        )
        .map_err(|e| Error::InitializationError(format!("Failed to create index: {}", e)))?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_synced ON player_game_stats(synced_to_cloud)",
            [],
        )
        .map_err(|e| Error::InitializationError(format!("Failed to create index: {}", e)))?;
        
        Ok(())
    }
    
    /// Get a reference to the database connection
    pub fn connection(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }
}

