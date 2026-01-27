//! Database schema initialization
//!
//! Simple approach: drop and recreate tables if schema doesn't match.

use rusqlite::Connection;

/// Current schema version - bump this to force a recreate
const SCHEMA_VERSION: i32 = 7;

/// Initialize the database schema
/// Drops and recreates all tables if version doesn't match
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
    
    // If version doesn't match, drop everything and recreate
    if current_version != SCHEMA_VERSION {
        log::info!("📦 Schema version mismatch ({} != {}), recreating database...", current_version, SCHEMA_VERSION);
        recreate_schema(conn)?;
    }
    
    Ok(())
}

/// Drop all tables and recreate with current schema
fn recreate_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    log::info!("🗑️ Dropping existing tables...");
    
    conn.execute_batch(
        "
        DROP TABLE IF EXISTS player_stats;
        DROP TABLE IF EXISTS game_stats;
        DROP TABLE IF EXISTS recordings;
        DROP TABLE IF EXISTS schema_version;
        "
    )?;
    
    log::info!("📦 Creating fresh schema v{}...", SCHEMA_VERSION);
    
    conn.execute_batch(
        "
        -- Schema version tracking
        CREATE TABLE schema_version (
            version INTEGER PRIMARY KEY
        );
        
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
        CREATE INDEX idx_recordings_start_time ON recordings(start_time DESC);
        
        -- Index for finding by video path
        CREATE INDEX idx_recordings_video_path ON recordings(video_path);
        
        -- Game stats table (linked to recordings or standalone for historical games)
        CREATE TABLE game_stats (
            id TEXT PRIMARY KEY,  -- UUID (same as recordings.id for recorded games)
            
            -- Player identifiers (connect codes)
            player1_id TEXT,
            player2_id TEXT,
            
            -- Port assignments
            player1_port INTEGER,
            player2_port INTEGER,
            
            -- Characters
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
            played_on TEXT,
            
            -- Match info
            match_id TEXT,
            game_number INTEGER,
            game_end_method TEXT,
            
            -- Timestamps
            created_at TEXT,  -- ISO 8601 timestamp when game was played
            
            -- For deduplication of historical games
            slp_path TEXT UNIQUE
        );
        
        -- Indexes for game_stats
        CREATE INDEX idx_game_stats_player1 ON game_stats(player1_id);
        CREATE INDEX idx_game_stats_player2 ON game_stats(player2_id);
        CREATE INDEX idx_game_stats_characters ON game_stats(player1_character, player2_character);
        CREATE INDEX idx_game_stats_stage ON game_stats(stage);
        CREATE INDEX idx_game_stats_slp_path ON game_stats(slp_path);
        CREATE INDEX idx_game_stats_created_at ON game_stats(created_at DESC);
        
        -- Player stats table (one-to-many: one game has multiple players)
        CREATE TABLE player_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            recording_id TEXT NOT NULL,
            player_index INTEGER NOT NULL,
            
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
            
            -- Action counts
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
            
            -- For historical games
            slp_path TEXT,
            
            -- Constraints
            UNIQUE(recording_id, player_index)
        );
        
        -- Indexes for player_stats
        CREATE INDEX idx_player_stats_recording ON player_stats(recording_id);
        CREATE INDEX idx_player_stats_connect_code ON player_stats(connect_code);
        CREATE INDEX idx_player_stats_character ON player_stats(character_id);
        CREATE INDEX idx_player_stats_slp_path ON player_stats(slp_path);
        "
    )?;
    
    // Set the version
    conn.execute(
        "INSERT INTO schema_version (version) VALUES (?)",
        [SCHEMA_VERSION],
    )?;
    
    log::info!("✅ Database schema v{} created", SCHEMA_VERSION);
    Ok(())
}

