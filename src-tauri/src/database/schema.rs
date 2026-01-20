//! Database schema and migrations

use rusqlite::Connection;

/// Current schema version
const SCHEMA_VERSION: i32 = 3;

/// Initialize the database schema
pub fn init_database(conn: &Connection) -> Result<(), rusqlite::Error> {
    // Create schema version table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY
        )",
        [],
    )?;
    
    // Get current version
    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    
    // Run migrations
    if current_version < 1 {
        migrate_v1(conn)?;
    }
    if current_version < 2 {
        migrate_v2(conn)?;
    }
    if current_version < 3 {
        migrate_v3(conn)?;
    }
    
    Ok(())
}

/// Version 1: Initial schema (now deprecated, but kept for migration path)
fn migrate_v1(conn: &Connection) -> Result<(), rusqlite::Error> {
    log::info!("📦 Running database migration v1...");
    
    // This was the old schema - we'll drop and recreate in v2
    // Just mark as complete
    conn.execute("INSERT OR IGNORE INTO schema_version (version) VALUES (1)", [])?;
    
    log::info!("✅ Database migration v1 complete");
    Ok(())
}

/// Version 2: UUID-based IDs with separate game_stats table
fn migrate_v2(conn: &Connection) -> Result<(), rusqlite::Error> {
    log::info!("📦 Running database migration v2 (UUID + game_stats)...");
    
    conn.execute_batch(
        "
        -- Drop old table if exists (fresh start with new schema)
        DROP TABLE IF EXISTS recordings;
        
        -- Main recordings table with UUID primary key
        CREATE TABLE recordings (
            id TEXT PRIMARY KEY,  -- UUID
            video_path TEXT NOT NULL UNIQUE,
            slp_path TEXT,
            
            -- File metadata
            file_size INTEGER,
            file_modified_at TEXT,
            
            -- Thumbnail
            thumbnail_path TEXT,
            
            -- Timing
            start_time TEXT,
            
            -- Cache metadata
            cached_at TEXT NOT NULL,
            needs_reparse INTEGER DEFAULT 0
        );
        
        -- Index for fast sorting by start time
        CREATE INDEX IF NOT EXISTS idx_recordings_start_time 
        ON recordings(start_time DESC);
        
        -- Index for finding by video path
        CREATE INDEX IF NOT EXISTS idx_recordings_video_path
        ON recordings(video_path);
        
        -- Game stats table (one-to-one with recordings that have .slp data)
        CREATE TABLE IF NOT EXISTS game_stats (
            id TEXT PRIMARY KEY,  -- UUID, same as recordings.id
            
            -- Player identifiers (connect codes, tags, or internal IDs)
            player1_id TEXT,
            player2_id TEXT,
            
            -- Port assignments
            player1_port INTEGER,
            player2_port INTEGER,
            
            -- Characters (by port)
            player1_character INTEGER,
            player2_character INTEGER,
            player1_color INTEGER,
            player2_color INTEGER,
            
            -- Game outcome
            winner_port INTEGER,
            loser_port INTEGER,
            
            -- Stage
            stage INTEGER,
            
            -- Duration
            game_duration INTEGER,
            total_frames INTEGER,
            
            -- Game info
            is_pal INTEGER DEFAULT 0,
            played_on TEXT,  -- 'dolphin', 'console', 'nintendont'
            
            -- Foreign key to recordings
            FOREIGN KEY (id) REFERENCES recordings(id) ON DELETE CASCADE
        );
        
        -- Index for player lookups
        CREATE INDEX IF NOT EXISTS idx_game_stats_player1 
        ON game_stats(player1_id);
        
        CREATE INDEX IF NOT EXISTS idx_game_stats_player2 
        ON game_stats(player2_id);
        
        -- Index for character stats
        CREATE INDEX IF NOT EXISTS idx_game_stats_characters
        ON game_stats(player1_character, player2_character);
        
        -- Index for stage stats
        CREATE INDEX IF NOT EXISTS idx_game_stats_stage
        ON game_stats(stage);
        
        -- Update schema version
        INSERT INTO schema_version (version) VALUES (2);
        "
    )?;
    
    log::info!("✅ Database migration v2 complete");
    Ok(())
}

/// Version 3: Extended player stats from slippi-js getStats()
fn migrate_v3(conn: &Connection) -> Result<(), rusqlite::Error> {
    log::info!("📦 Running database migration v3 (computed player stats)...");
    
    conn.execute_batch(
        "
        -- Add match info to game_stats
        ALTER TABLE game_stats ADD COLUMN match_id TEXT;
        ALTER TABLE game_stats ADD COLUMN game_number INTEGER;
        ALTER TABLE game_stats ADD COLUMN game_end_method TEXT;
        
        -- Player stats table (one-to-many: one game has multiple players)
        CREATE TABLE IF NOT EXISTS player_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            recording_id TEXT NOT NULL,  -- Links to recordings.id
            player_index INTEGER NOT NULL,  -- 0-3
            
            -- Player identification
            connect_code TEXT,
            display_name TEXT,
            character_id INTEGER NOT NULL,
            character_color INTEGER DEFAULT 0,
            port INTEGER NOT NULL,
            
            -- Overall performance
            total_damage REAL DEFAULT 0,
            kill_count INTEGER DEFAULT 0,
            conversion_count INTEGER DEFAULT 0,
            successful_conversions INTEGER DEFAULT 0,
            openings_per_kill REAL,
            damage_per_opening REAL,
            neutral_win_ratio REAL,
            counter_hit_ratio REAL,
            beneficial_trade_ratio REAL,
            
            -- Input stats
            inputs_total INTEGER DEFAULT 0,
            inputs_per_minute REAL,
            avg_kill_percent REAL,
            
            -- Action counts (tech skill)
            wavedash_count INTEGER DEFAULT 0,
            waveland_count INTEGER DEFAULT 0,
            air_dodge_count INTEGER DEFAULT 0,
            dash_dance_count INTEGER DEFAULT 0,
            spot_dodge_count INTEGER DEFAULT 0,
            ledgegrab_count INTEGER DEFAULT 0,
            roll_count INTEGER DEFAULT 0,
            grab_count INTEGER DEFAULT 0,
            throw_count INTEGER DEFAULT 0,
            ground_tech_count INTEGER DEFAULT 0,
            wall_tech_count INTEGER DEFAULT 0,
            wall_jump_tech_count INTEGER DEFAULT 0,
            
            -- L-Cancel stats
            l_cancel_success_count INTEGER DEFAULT 0,
            l_cancel_fail_count INTEGER DEFAULT 0,
            
            -- Final game state
            stocks_remaining INTEGER DEFAULT 0,
            final_percent REAL,
            
            -- Constraints
            UNIQUE(recording_id, player_index),
            FOREIGN KEY (recording_id) REFERENCES recordings(id) ON DELETE CASCADE
        );
        
        -- Indexes for player_stats
        CREATE INDEX IF NOT EXISTS idx_player_stats_recording 
        ON player_stats(recording_id);
        
        CREATE INDEX IF NOT EXISTS idx_player_stats_connect_code 
        ON player_stats(connect_code);
        
        CREATE INDEX IF NOT EXISTS idx_player_stats_character 
        ON player_stats(character_id);
        
        -- Update schema version
        INSERT INTO schema_version (version) VALUES (3);
        "
    )?;
    
    log::info!("✅ Database migration v3 complete");
    Ok(())
}
