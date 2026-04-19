//! Recording, game stats, and player stats database operations

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

/// Core recording row from the recordings table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingRow {
    pub id: String,
    pub video_path: String,
    pub slp_path: Option<String>,
    pub file_size: Option<i64>,
    pub file_modified_at: Option<String>,
    pub thumbnail_path: Option<String>,
    pub start_time: Option<String>,
    pub cached_at: String,
    pub needs_reparse: bool,
}

/// Game stats row from the game_stats table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStatsRow {
    pub id: String,
    pub player1_id: Option<String>,
    pub player2_id: Option<String>,
    pub player1_port: Option<i32>,
    pub player2_port: Option<i32>,
    pub player1_character: Option<i32>,
    pub player2_character: Option<i32>,
    pub player1_color: Option<i32>,
    pub player2_color: Option<i32>,
    pub winner_port: Option<i32>,
    pub loser_port: Option<i32>,
    pub stage: Option<i32>,
    pub game_duration: Option<i32>,
    pub total_frames: Option<i32>,
    pub is_pal: Option<bool>,
    pub played_on: Option<String>,
    /// ISO 8601 timestamp when game was played
    pub created_at: Option<String>,
    /// Match ID for Bo3/Bo5 sets
    pub match_id: Option<String>,
    /// Game number within a match/set
    pub game_number: Option<i32>,
    /// How the game ended (e.g., "game_end", "lras")
    pub game_end_method: Option<String>,
    /// Path to .slp file - used for deduplication of historical games
    pub slp_path: Option<String>,
    /// User-editable notes about the game
    pub notes: Option<String>,
}

/// Combined recording with its stats (for paginated queries)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingWithStats {
    pub recording: RecordingRow,
    pub stats: Option<GameStatsRow>,
    pub player_stats: Vec<PlayerStatsRow>,
}

/// Player stats row from the player_stats table
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerStatsRow {
    pub id: Option<i64>,
    pub recording_id: String,
    pub player_index: i32,
    pub connect_code: Option<String>,
    pub display_name: Option<String>,
    pub character_id: i32,
    pub character_color: i32,
    pub port: i32,
    pub total_damage: f64,
    pub kill_count: i32,
    pub conversion_count: i32,
    pub successful_conversions: i32,
    pub openings_per_kill: Option<f64>,
    pub damage_per_opening: Option<f64>,
    pub neutral_win_ratio: Option<f64>,
    pub counter_hit_ratio: Option<f64>,
    pub beneficial_trade_ratio: Option<f64>,
    pub inputs_total: i32,
    pub inputs_per_minute: Option<f64>,
    pub avg_kill_percent: Option<f64>,
    pub wavedash_count: i32,
    pub waveland_count: i32,
    pub air_dodge_count: i32,
    pub dash_dance_count: i32,
    pub spot_dodge_count: i32,
    pub ledgegrab_count: i32,
    pub roll_count: i32,
    pub grab_count: i32,
    pub throw_count: i32,
    pub ground_tech_count: i32,
    pub wall_tech_count: i32,
    pub wall_jump_tech_count: i32,
    pub l_cancel_success_count: i32,
    pub l_cancel_fail_count: i32,
    // L-Cancel detailed breakdown: per aerial x target x outcome
    // Nair
    pub l_cancel_nair_shield_success: i32,
    pub l_cancel_nair_shield_fail: i32,
    pub l_cancel_nair_whiff_success: i32,
    pub l_cancel_nair_whiff_fail: i32,
    pub l_cancel_nair_hit_success: i32,
    pub l_cancel_nair_hit_fail: i32,
    // Fair
    pub l_cancel_fair_shield_success: i32,
    pub l_cancel_fair_shield_fail: i32,
    pub l_cancel_fair_whiff_success: i32,
    pub l_cancel_fair_whiff_fail: i32,
    pub l_cancel_fair_hit_success: i32,
    pub l_cancel_fair_hit_fail: i32,
    // Bair
    pub l_cancel_bair_shield_success: i32,
    pub l_cancel_bair_shield_fail: i32,
    pub l_cancel_bair_whiff_success: i32,
    pub l_cancel_bair_whiff_fail: i32,
    pub l_cancel_bair_hit_success: i32,
    pub l_cancel_bair_hit_fail: i32,
    // Uair
    pub l_cancel_uair_shield_success: i32,
    pub l_cancel_uair_shield_fail: i32,
    pub l_cancel_uair_whiff_success: i32,
    pub l_cancel_uair_whiff_fail: i32,
    pub l_cancel_uair_hit_success: i32,
    pub l_cancel_uair_hit_fail: i32,
    // Dair
    pub l_cancel_dair_shield_success: i32,
    pub l_cancel_dair_shield_fail: i32,
    pub l_cancel_dair_whiff_success: i32,
    pub l_cancel_dair_whiff_fail: i32,
    pub l_cancel_dair_hit_success: i32,
    pub l_cancel_dair_hit_fail: i32,
    // Shield grab (placeholder for now)
    pub shield_grab_count: i32,
    pub stocks_remaining: i32,
    pub final_percent: Option<f64>,
    /// Path to .slp file - for historical games that don't have a recording
    pub slp_path: Option<String>,
}

// ============================================================================
// RECORDING OPERATIONS
// ============================================================================

/// Get all recordings (no pagination, for clips filtering etc)
pub fn get_all_recordings(conn: &Connection) -> rusqlite::Result<Vec<RecordingRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, video_path, slp_path, file_size, file_modified_at, 
                thumbnail_path, start_time, cached_at, needs_reparse
         FROM recordings 
         ORDER BY start_time DESC"
    )?;
    
    let rows = stmt.query_map([], |row| {
        Ok(RecordingRow {
            id: row.get(0)?,
            video_path: row.get(1)?,
            slp_path: row.get(2)?,
            file_size: row.get(3)?,
            file_modified_at: row.get(4)?,
            thumbnail_path: row.get(5)?,
            start_time: row.get(6)?,
            cached_at: row.get(7)?,
            needs_reparse: row.get::<_, i32>(8)? != 0,
        })
    })?;
    
    rows.collect()
}

/// Get recordings with pagination, joined with game_stats and player_stats
/// Excludes clips (videos in the Clips folder) - those are fetched separately via get_clips
pub fn get_recordings_paginated(
    conn: &Connection, 
    limit: i32, 
    offset: i32
) -> rusqlite::Result<(Vec<RecordingWithStats>, i32)> {
    // Get total count (excluding clips)
    let total: i32 = conn.query_row(
        "SELECT COUNT(*) FROM recordings WHERE video_path NOT LIKE '%Clips%'",
        [],
        |row| row.get(0),
    )?;
    
    // Get paginated rows with game stats (excluding clips)
    let mut stmt = conn.prepare(
        "SELECT r.id, r.video_path, r.slp_path, r.file_size, r.file_modified_at, 
                r.thumbnail_path, r.start_time, r.cached_at, r.needs_reparse,
                g.player1_id, g.player2_id, g.player1_port, g.player2_port,
                g.player1_character, g.player2_character, g.player1_color, g.player2_color,
                g.winner_port, g.loser_port, g.stage, g.game_duration, g.total_frames,
                g.is_pal, g.played_on, g.created_at, g.match_id, g.game_number,
                g.game_end_method, g.slp_path, g.notes
         FROM recordings r
         LEFT JOIN game_stats g ON r.id = g.id
         WHERE r.video_path NOT LIKE '%Clips%'
         ORDER BY r.start_time DESC
         LIMIT ? OFFSET ?"
    )?;
    
    let rows = stmt.query_map(params![limit, offset], |row| {
        let recording = RecordingRow {
            id: row.get(0)?,
            video_path: row.get(1)?,
            slp_path: row.get(2)?,
            file_size: row.get(3)?,
            file_modified_at: row.get(4)?,
            thumbnail_path: row.get(5)?,
            start_time: row.get(6)?,
            cached_at: row.get(7)?,
            needs_reparse: row.get::<_, i32>(8)? != 0,
        };
        
        // Check if we have stats (by checking if player1_character is not null)
        let has_stats = row.get::<_, Option<i32>>(13)?.is_some();
        let stats = if has_stats {
            Some(GameStatsRow {
                id: row.get(0)?,
                player1_id: row.get(9)?,
                player2_id: row.get(10)?,
                player1_port: row.get(11)?,
                player2_port: row.get(12)?,
                player1_character: row.get(13)?,
                player2_character: row.get(14)?,
                player1_color: row.get(15)?,
                player2_color: row.get(16)?,
                winner_port: row.get(17)?,
                loser_port: row.get(18)?,
                stage: row.get(19)?,
                game_duration: row.get(20)?,
                total_frames: row.get(21)?,
                is_pal: row.get::<_, Option<i32>>(22)?.map(|v| v != 0),
                played_on: row.get(23)?,
                created_at: row.get(24)?,
                match_id: row.get(25)?,
                game_number: row.get(26)?,
                game_end_method: row.get(27)?,
                slp_path: row.get(28)?,
                notes: row.get(29)?,
            })
        } else {
            None
        };
        
        // Player stats will be fetched separately - start with empty
        Ok(RecordingWithStats { recording, stats, player_stats: Vec::new() })
    })?;
    
    let mut results: Vec<RecordingWithStats> = rows.collect::<Result<Vec<_>, _>>()?;
    
    // Fetch player_stats for all recordings in one query
    if !results.is_empty() {
        let recording_ids: Vec<String> = results.iter().map(|r| r.recording.id.clone()).collect();
        let placeholders: String = recording_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        
        let query = format!(
            "SELECT id, recording_id, player_index, connect_code, display_name, 
                    character_id, character_color, port, total_damage, kill_count,
                    conversion_count, successful_conversions, openings_per_kill, 
                    damage_per_opening, neutral_win_ratio, counter_hit_ratio, 
                    beneficial_trade_ratio, inputs_total, inputs_per_minute, avg_kill_percent,
                    wavedash_count, waveland_count, air_dodge_count, dash_dance_count,
                    spot_dodge_count, ledgegrab_count, roll_count, grab_count, throw_count,
                    ground_tech_count, wall_tech_count, wall_jump_tech_count,
                    l_cancel_success_count, l_cancel_fail_count,
                    l_cancel_nair_shield_success, l_cancel_nair_shield_fail,
                    l_cancel_nair_whiff_success, l_cancel_nair_whiff_fail,
                    l_cancel_nair_hit_success, l_cancel_nair_hit_fail,
                    l_cancel_fair_shield_success, l_cancel_fair_shield_fail,
                    l_cancel_fair_whiff_success, l_cancel_fair_whiff_fail,
                    l_cancel_fair_hit_success, l_cancel_fair_hit_fail,
                    l_cancel_bair_shield_success, l_cancel_bair_shield_fail,
                    l_cancel_bair_whiff_success, l_cancel_bair_whiff_fail,
                    l_cancel_bair_hit_success, l_cancel_bair_hit_fail,
                    l_cancel_uair_shield_success, l_cancel_uair_shield_fail,
                    l_cancel_uair_whiff_success, l_cancel_uair_whiff_fail,
                    l_cancel_uair_hit_success, l_cancel_uair_hit_fail,
                    l_cancel_dair_shield_success, l_cancel_dair_shield_fail,
                    l_cancel_dair_whiff_success, l_cancel_dair_whiff_fail,
                    l_cancel_dair_hit_success, l_cancel_dair_hit_fail,
                    shield_grab_count, stocks_remaining, final_percent, slp_path
             FROM player_stats 
             WHERE recording_id IN ({})
             ORDER BY recording_id, player_index",
            placeholders
        );
        
        let mut stmt = conn.prepare(&query)?;
        let params: Vec<&dyn rusqlite::ToSql> = recording_ids.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        
        let player_rows = stmt.query_map(params.as_slice(), |row| {
            Ok(PlayerStatsRow {
                id: row.get(0)?,
                recording_id: row.get(1)?,
                player_index: row.get(2)?,
                connect_code: row.get(3)?,
                display_name: row.get(4)?,
                character_id: row.get(5)?,
                character_color: row.get(6)?,
                port: row.get(7)?,
                total_damage: row.get(8)?,
                kill_count: row.get(9)?,
                conversion_count: row.get(10)?,
                successful_conversions: row.get(11)?,
                openings_per_kill: row.get(12)?,
                damage_per_opening: row.get(13)?,
                neutral_win_ratio: row.get(14)?,
                counter_hit_ratio: row.get(15)?,
                beneficial_trade_ratio: row.get(16)?,
                inputs_total: row.get(17)?,
                inputs_per_minute: row.get(18)?,
                avg_kill_percent: row.get(19)?,
                wavedash_count: row.get(20)?,
                waveland_count: row.get(21)?,
                air_dodge_count: row.get(22)?,
                dash_dance_count: row.get(23)?,
                spot_dodge_count: row.get(24)?,
                ledgegrab_count: row.get(25)?,
                roll_count: row.get(26)?,
                grab_count: row.get(27)?,
                throw_count: row.get(28)?,
                ground_tech_count: row.get(29)?,
                wall_tech_count: row.get(30)?,
                wall_jump_tech_count: row.get(31)?,
                l_cancel_success_count: row.get(32)?,
                l_cancel_fail_count: row.get(33)?,
                // L-Cancel detailed breakdown
                l_cancel_nair_shield_success: row.get(34)?,
                l_cancel_nair_shield_fail: row.get(35)?,
                l_cancel_nair_whiff_success: row.get(36)?,
                l_cancel_nair_whiff_fail: row.get(37)?,
                l_cancel_nair_hit_success: row.get(38)?,
                l_cancel_nair_hit_fail: row.get(39)?,
                l_cancel_fair_shield_success: row.get(40)?,
                l_cancel_fair_shield_fail: row.get(41)?,
                l_cancel_fair_whiff_success: row.get(42)?,
                l_cancel_fair_whiff_fail: row.get(43)?,
                l_cancel_fair_hit_success: row.get(44)?,
                l_cancel_fair_hit_fail: row.get(45)?,
                l_cancel_bair_shield_success: row.get(46)?,
                l_cancel_bair_shield_fail: row.get(47)?,
                l_cancel_bair_whiff_success: row.get(48)?,
                l_cancel_bair_whiff_fail: row.get(49)?,
                l_cancel_bair_hit_success: row.get(50)?,
                l_cancel_bair_hit_fail: row.get(51)?,
                l_cancel_uair_shield_success: row.get(52)?,
                l_cancel_uair_shield_fail: row.get(53)?,
                l_cancel_uair_whiff_success: row.get(54)?,
                l_cancel_uair_whiff_fail: row.get(55)?,
                l_cancel_uair_hit_success: row.get(56)?,
                l_cancel_uair_hit_fail: row.get(57)?,
                l_cancel_dair_shield_success: row.get(58)?,
                l_cancel_dair_shield_fail: row.get(59)?,
                l_cancel_dair_whiff_success: row.get(60)?,
                l_cancel_dair_whiff_fail: row.get(61)?,
                l_cancel_dair_hit_success: row.get(62)?,
                l_cancel_dair_hit_fail: row.get(63)?,
                shield_grab_count: row.get(64)?,
                stocks_remaining: row.get(65)?,
                final_percent: row.get(66)?,
                slp_path: row.get(67)?,
            })
        })?;
        
        let all_player_stats: Vec<PlayerStatsRow> = player_rows.collect::<Result<Vec<_>, _>>()?;
        
        // Group player stats by recording_id
        for result in &mut results {
            result.player_stats = all_player_stats
                .iter()
                .filter(|ps| ps.recording_id == result.recording.id)
                .cloned()
                .collect();
        }
    }
    
    Ok((results, total))
}

/// Get a recording by video path
pub fn get_recording_by_video_path(conn: &Connection, video_path: &str) -> rusqlite::Result<Option<RecordingRow>> {
    conn.query_row(
        "SELECT id, video_path, slp_path, file_size, file_modified_at, 
                thumbnail_path, start_time, cached_at, needs_reparse
         FROM recordings WHERE video_path = ?",
        params![video_path],
        |row| {
            Ok(RecordingRow {
                id: row.get(0)?,
                video_path: row.get(1)?,
                slp_path: row.get(2)?,
                file_size: row.get(3)?,
                file_modified_at: row.get(4)?,
                thumbnail_path: row.get(5)?,
                start_time: row.get(6)?,
                cached_at: row.get(7)?,
                needs_reparse: row.get::<_, i32>(8)? != 0,
            })
        },
    ).optional()
}

/// Insert or update a recording
pub fn upsert_recording(conn: &Connection, row: &RecordingRow) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO recordings (id, video_path, slp_path, file_size, file_modified_at, 
                                 thumbnail_path, start_time, cached_at, needs_reparse)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
            video_path = excluded.video_path,
            slp_path = excluded.slp_path,
            file_size = excluded.file_size,
            file_modified_at = excluded.file_modified_at,
            thumbnail_path = excluded.thumbnail_path,
            start_time = excluded.start_time,
            cached_at = excluded.cached_at,
            needs_reparse = excluded.needs_reparse",
        params![
            row.id,
            row.video_path,
            row.slp_path,
            row.file_size,
            row.file_modified_at,
            row.thumbnail_path,
            row.start_time,
            row.cached_at,
            row.needs_reparse as i32,
        ],
    )?;
    Ok(())
}

/// Update the `.slp` file path associated with a recording. Called after stats
/// are saved so the recording knows which Slippi replay it came from (the
/// filesystem sync can't always figure this out from video/slp filenames when
/// OBS uses its own naming scheme).
pub fn set_recording_slp_path(
    conn: &Connection,
    id: &str,
    slp_path: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE recordings SET slp_path = ? WHERE id = ?",
        params![slp_path, id],
    )?;
    Ok(())
}

/// Delete a recording by ID
pub fn delete_recording(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM recordings WHERE id = ?", params![id])?;
    Ok(())
}

/// Get all cached video paths (for sync comparison)
pub fn get_cached_video_paths(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT video_path FROM recordings")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    rows.collect()
}

// ============================================================================
// GAME STATS OPERATIONS
// ============================================================================

/// Insert or update game stats (notes are preserved on update - use set_game_notes to change them)
pub fn upsert_game_stats(conn: &Connection, stats: &GameStatsRow) -> rusqlite::Result<()> {
    // Remove any existing row with the same slp_path but different id to avoid
    // UNIQUE constraint violation (the new id's row takes precedence).
    if let Some(ref slp) = stats.slp_path {
        conn.execute(
            "DELETE FROM game_stats WHERE slp_path = ? AND id != ?",
            params![slp, stats.id],
        )?;
    }

    conn.execute(
        "INSERT INTO game_stats (id, player1_id, player2_id, player1_port, player2_port,
                                  player1_character, player2_character, player1_color, player2_color,
                                  winner_port, loser_port, stage, game_duration, total_frames,
                                  is_pal, played_on, created_at, match_id, game_number,
                                  game_end_method, slp_path, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)
         ON CONFLICT(id) DO UPDATE SET
            player1_id = excluded.player1_id,
            player2_id = excluded.player2_id,
            player1_port = excluded.player1_port,
            player2_port = excluded.player2_port,
            player1_character = excluded.player1_character,
            player2_character = excluded.player2_character,
            player1_color = excluded.player1_color,
            player2_color = excluded.player2_color,
            winner_port = excluded.winner_port,
            loser_port = excluded.loser_port,
            stage = excluded.stage,
            game_duration = excluded.game_duration,
            total_frames = excluded.total_frames,
            is_pal = excluded.is_pal,
            played_on = excluded.played_on,
            created_at = excluded.created_at,
            match_id = excluded.match_id,
            game_number = excluded.game_number,
            game_end_method = excluded.game_end_method,
            slp_path = excluded.slp_path",
        params![
            stats.id,
            stats.player1_id,
            stats.player2_id,
            stats.player1_port,
            stats.player2_port,
            stats.player1_character,
            stats.player2_character,
            stats.player1_color,
            stats.player2_color,
            stats.winner_port,
            stats.loser_port,
            stats.stage,
            stats.game_duration,
            stats.total_frames,
            stats.is_pal.map(|b| b as i32),
            stats.played_on,
            stats.created_at,
            stats.match_id,
            stats.game_number,
            stats.game_end_method,
            stats.slp_path,
            stats.notes,
        ],
    )?;
    Ok(())
}

/// Get game notes for a recording/game by id (game_stats.id or recordings.id)
pub fn get_game_notes(conn: &Connection, id: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT notes FROM game_stats WHERE id = ?",
        params![id],
        |row| row.get::<_, Option<String>>(0),
    )
    .optional()
    .map(|o| o.flatten())
}

/// Set game notes for a recording/game. Creates a game_stats row with only id and notes if none exists.
pub fn set_game_notes(conn: &Connection, id: &str, notes: Option<&str>) -> rusqlite::Result<()> {
    let updated = conn.execute(
        "UPDATE game_stats SET notes = ? WHERE id = ?",
        params![notes, id],
    )?;
    if updated == 0 {
        conn.execute(
            "INSERT INTO game_stats (id, notes) VALUES (?, ?)",
            params![id, notes],
        )?;
    }
    Ok(())
}

/// Check if a game_stats entry exists for the given slp_path
pub fn game_stats_exists_by_slp_path(conn: &Connection, slp_path: &str) -> rusqlite::Result<bool> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM game_stats WHERE slp_path = ?",
        params![slp_path],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

// ============================================================================
// PLAYER STATS OPERATIONS
// ============================================================================

/// Insert or update player stats
pub fn upsert_player_stats(conn: &Connection, stats: &PlayerStatsRow) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO player_stats (
            recording_id, player_index, connect_code, display_name, character_id, character_color, port,
            total_damage, kill_count, conversion_count, successful_conversions,
            openings_per_kill, damage_per_opening, neutral_win_ratio, counter_hit_ratio, beneficial_trade_ratio,
            inputs_total, inputs_per_minute, avg_kill_percent,
            wavedash_count, waveland_count, air_dodge_count, dash_dance_count, spot_dodge_count, ledgegrab_count,
            roll_count, grab_count, throw_count, ground_tech_count, wall_tech_count, wall_jump_tech_count,
            l_cancel_success_count, l_cancel_fail_count,
            l_cancel_nair_shield_success, l_cancel_nair_shield_fail,
            l_cancel_nair_whiff_success, l_cancel_nair_whiff_fail,
            l_cancel_nair_hit_success, l_cancel_nair_hit_fail,
            l_cancel_fair_shield_success, l_cancel_fair_shield_fail,
            l_cancel_fair_whiff_success, l_cancel_fair_whiff_fail,
            l_cancel_fair_hit_success, l_cancel_fair_hit_fail,
            l_cancel_bair_shield_success, l_cancel_bair_shield_fail,
            l_cancel_bair_whiff_success, l_cancel_bair_whiff_fail,
            l_cancel_bair_hit_success, l_cancel_bair_hit_fail,
            l_cancel_uair_shield_success, l_cancel_uair_shield_fail,
            l_cancel_uair_whiff_success, l_cancel_uair_whiff_fail,
            l_cancel_uair_hit_success, l_cancel_uair_hit_fail,
            l_cancel_dair_shield_success, l_cancel_dair_shield_fail,
            l_cancel_dair_whiff_success, l_cancel_dair_whiff_fail,
            l_cancel_dair_hit_success, l_cancel_dair_hit_fail,
            shield_grab_count, stocks_remaining, final_percent, slp_path
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
            ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33,
            ?34, ?35, ?36, ?37, ?38, ?39, ?40, ?41, ?42, ?43, ?44, ?45, ?46, ?47, ?48, ?49,
            ?50, ?51, ?52, ?53, ?54, ?55, ?56, ?57, ?58, ?59, ?60, ?61, ?62, ?63, ?64, ?65, ?66, ?67
        )
        ON CONFLICT(recording_id, player_index) DO UPDATE SET
            connect_code = excluded.connect_code,
            display_name = excluded.display_name,
            character_id = excluded.character_id,
            character_color = excluded.character_color,
            port = excluded.port,
            total_damage = excluded.total_damage,
            kill_count = excluded.kill_count,
            conversion_count = excluded.conversion_count,
            successful_conversions = excluded.successful_conversions,
            openings_per_kill = excluded.openings_per_kill,
            damage_per_opening = excluded.damage_per_opening,
            neutral_win_ratio = excluded.neutral_win_ratio,
            counter_hit_ratio = excluded.counter_hit_ratio,
            beneficial_trade_ratio = excluded.beneficial_trade_ratio,
            inputs_total = excluded.inputs_total,
            inputs_per_minute = excluded.inputs_per_minute,
            avg_kill_percent = excluded.avg_kill_percent,
            wavedash_count = excluded.wavedash_count,
            waveland_count = excluded.waveland_count,
            air_dodge_count = excluded.air_dodge_count,
            dash_dance_count = excluded.dash_dance_count,
            spot_dodge_count = excluded.spot_dodge_count,
            ledgegrab_count = excluded.ledgegrab_count,
            roll_count = excluded.roll_count,
            grab_count = excluded.grab_count,
            throw_count = excluded.throw_count,
            ground_tech_count = excluded.ground_tech_count,
            wall_tech_count = excluded.wall_tech_count,
            wall_jump_tech_count = excluded.wall_jump_tech_count,
            l_cancel_success_count = excluded.l_cancel_success_count,
            l_cancel_fail_count = excluded.l_cancel_fail_count,
            l_cancel_nair_shield_success = excluded.l_cancel_nair_shield_success,
            l_cancel_nair_shield_fail = excluded.l_cancel_nair_shield_fail,
            l_cancel_nair_whiff_success = excluded.l_cancel_nair_whiff_success,
            l_cancel_nair_whiff_fail = excluded.l_cancel_nair_whiff_fail,
            l_cancel_nair_hit_success = excluded.l_cancel_nair_hit_success,
            l_cancel_nair_hit_fail = excluded.l_cancel_nair_hit_fail,
            l_cancel_fair_shield_success = excluded.l_cancel_fair_shield_success,
            l_cancel_fair_shield_fail = excluded.l_cancel_fair_shield_fail,
            l_cancel_fair_whiff_success = excluded.l_cancel_fair_whiff_success,
            l_cancel_fair_whiff_fail = excluded.l_cancel_fair_whiff_fail,
            l_cancel_fair_hit_success = excluded.l_cancel_fair_hit_success,
            l_cancel_fair_hit_fail = excluded.l_cancel_fair_hit_fail,
            l_cancel_bair_shield_success = excluded.l_cancel_bair_shield_success,
            l_cancel_bair_shield_fail = excluded.l_cancel_bair_shield_fail,
            l_cancel_bair_whiff_success = excluded.l_cancel_bair_whiff_success,
            l_cancel_bair_whiff_fail = excluded.l_cancel_bair_whiff_fail,
            l_cancel_bair_hit_success = excluded.l_cancel_bair_hit_success,
            l_cancel_bair_hit_fail = excluded.l_cancel_bair_hit_fail,
            l_cancel_uair_shield_success = excluded.l_cancel_uair_shield_success,
            l_cancel_uair_shield_fail = excluded.l_cancel_uair_shield_fail,
            l_cancel_uair_whiff_success = excluded.l_cancel_uair_whiff_success,
            l_cancel_uair_whiff_fail = excluded.l_cancel_uair_whiff_fail,
            l_cancel_uair_hit_success = excluded.l_cancel_uair_hit_success,
            l_cancel_uair_hit_fail = excluded.l_cancel_uair_hit_fail,
            l_cancel_dair_shield_success = excluded.l_cancel_dair_shield_success,
            l_cancel_dair_shield_fail = excluded.l_cancel_dair_shield_fail,
            l_cancel_dair_whiff_success = excluded.l_cancel_dair_whiff_success,
            l_cancel_dair_whiff_fail = excluded.l_cancel_dair_whiff_fail,
            l_cancel_dair_hit_success = excluded.l_cancel_dair_hit_success,
            l_cancel_dair_hit_fail = excluded.l_cancel_dair_hit_fail,
            shield_grab_count = excluded.shield_grab_count,
            stocks_remaining = excluded.stocks_remaining,
            final_percent = excluded.final_percent,
            slp_path = excluded.slp_path",
        params![
            stats.recording_id,
            stats.player_index,
            stats.connect_code,
            stats.display_name,
            stats.character_id,
            stats.character_color,
            stats.port,
            stats.total_damage,
            stats.kill_count,
            stats.conversion_count,
            stats.successful_conversions,
            stats.openings_per_kill,
            stats.damage_per_opening,
            stats.neutral_win_ratio,
            stats.counter_hit_ratio,
            stats.beneficial_trade_ratio,
            stats.inputs_total,
            stats.inputs_per_minute,
            stats.avg_kill_percent,
            stats.wavedash_count,
            stats.waveland_count,
            stats.air_dodge_count,
            stats.dash_dance_count,
            stats.spot_dodge_count,
            stats.ledgegrab_count,
            stats.roll_count,
            stats.grab_count,
            stats.throw_count,
            stats.ground_tech_count,
            stats.wall_tech_count,
            stats.wall_jump_tech_count,
            stats.l_cancel_success_count,
            stats.l_cancel_fail_count,
            stats.l_cancel_nair_shield_success,
            stats.l_cancel_nair_shield_fail,
            stats.l_cancel_nair_whiff_success,
            stats.l_cancel_nair_whiff_fail,
            stats.l_cancel_nair_hit_success,
            stats.l_cancel_nair_hit_fail,
            stats.l_cancel_fair_shield_success,
            stats.l_cancel_fair_shield_fail,
            stats.l_cancel_fair_whiff_success,
            stats.l_cancel_fair_whiff_fail,
            stats.l_cancel_fair_hit_success,
            stats.l_cancel_fair_hit_fail,
            stats.l_cancel_bair_shield_success,
            stats.l_cancel_bair_shield_fail,
            stats.l_cancel_bair_whiff_success,
            stats.l_cancel_bair_whiff_fail,
            stats.l_cancel_bair_hit_success,
            stats.l_cancel_bair_hit_fail,
            stats.l_cancel_uair_shield_success,
            stats.l_cancel_uair_shield_fail,
            stats.l_cancel_uair_whiff_success,
            stats.l_cancel_uair_whiff_fail,
            stats.l_cancel_uair_hit_success,
            stats.l_cancel_uair_hit_fail,
            stats.l_cancel_dair_shield_success,
            stats.l_cancel_dair_shield_fail,
            stats.l_cancel_dair_whiff_success,
            stats.l_cancel_dair_whiff_fail,
            stats.l_cancel_dair_hit_success,
            stats.l_cancel_dair_hit_fail,
            stats.shield_grab_count,
            stats.stocks_remaining,
            stats.final_percent,
            stats.slp_path,
        ],
    )?;
    Ok(())
}

/// Get player stats for a recording
pub fn get_player_stats_by_recording(conn: &Connection, recording_id: &str) -> rusqlite::Result<Vec<PlayerStatsRow>> {
    let mut stmt = conn.prepare(
        "SELECT id, recording_id, player_index, connect_code, display_name, character_id, character_color, port,
                total_damage, kill_count, conversion_count, successful_conversions,
                openings_per_kill, damage_per_opening, neutral_win_ratio, counter_hit_ratio, beneficial_trade_ratio,
                inputs_total, inputs_per_minute, avg_kill_percent,
                wavedash_count, waveland_count, air_dodge_count, dash_dance_count, spot_dodge_count, ledgegrab_count,
                roll_count, grab_count, throw_count, ground_tech_count, wall_tech_count, wall_jump_tech_count,
                l_cancel_success_count, l_cancel_fail_count,
                l_cancel_nair_shield_success, l_cancel_nair_shield_fail,
                l_cancel_nair_whiff_success, l_cancel_nair_whiff_fail,
                l_cancel_nair_hit_success, l_cancel_nair_hit_fail,
                l_cancel_fair_shield_success, l_cancel_fair_shield_fail,
                l_cancel_fair_whiff_success, l_cancel_fair_whiff_fail,
                l_cancel_fair_hit_success, l_cancel_fair_hit_fail,
                l_cancel_bair_shield_success, l_cancel_bair_shield_fail,
                l_cancel_bair_whiff_success, l_cancel_bair_whiff_fail,
                l_cancel_bair_hit_success, l_cancel_bair_hit_fail,
                l_cancel_uair_shield_success, l_cancel_uair_shield_fail,
                l_cancel_uair_whiff_success, l_cancel_uair_whiff_fail,
                l_cancel_uair_hit_success, l_cancel_uair_hit_fail,
                l_cancel_dair_shield_success, l_cancel_dair_shield_fail,
                l_cancel_dair_whiff_success, l_cancel_dair_whiff_fail,
                l_cancel_dair_hit_success, l_cancel_dair_hit_fail,
                shield_grab_count, stocks_remaining, final_percent, slp_path
         FROM player_stats WHERE recording_id = ? ORDER BY player_index"
    )?;
    
    let rows = stmt.query_map(params![recording_id], |row| {
        Ok(PlayerStatsRow {
            id: row.get(0)?,
            recording_id: row.get(1)?,
            player_index: row.get(2)?,
            connect_code: row.get(3)?,
            display_name: row.get(4)?,
            character_id: row.get(5)?,
            character_color: row.get(6)?,
            port: row.get(7)?,
            total_damage: row.get(8)?,
            kill_count: row.get(9)?,
            conversion_count: row.get(10)?,
            successful_conversions: row.get(11)?,
            openings_per_kill: row.get(12)?,
            damage_per_opening: row.get(13)?,
            neutral_win_ratio: row.get(14)?,
            counter_hit_ratio: row.get(15)?,
            beneficial_trade_ratio: row.get(16)?,
            inputs_total: row.get(17)?,
            inputs_per_minute: row.get(18)?,
            avg_kill_percent: row.get(19)?,
            wavedash_count: row.get(20)?,
            waveland_count: row.get(21)?,
            air_dodge_count: row.get(22)?,
            dash_dance_count: row.get(23)?,
            spot_dodge_count: row.get(24)?,
            ledgegrab_count: row.get(25)?,
            roll_count: row.get(26)?,
            grab_count: row.get(27)?,
            throw_count: row.get(28)?,
            ground_tech_count: row.get(29)?,
            wall_tech_count: row.get(30)?,
            wall_jump_tech_count: row.get(31)?,
            l_cancel_success_count: row.get(32)?,
            l_cancel_fail_count: row.get(33)?,
            // L-Cancel detailed breakdown
            l_cancel_nair_shield_success: row.get(34)?,
            l_cancel_nair_shield_fail: row.get(35)?,
            l_cancel_nair_whiff_success: row.get(36)?,
            l_cancel_nair_whiff_fail: row.get(37)?,
            l_cancel_nair_hit_success: row.get(38)?,
            l_cancel_nair_hit_fail: row.get(39)?,
            l_cancel_fair_shield_success: row.get(40)?,
            l_cancel_fair_shield_fail: row.get(41)?,
            l_cancel_fair_whiff_success: row.get(42)?,
            l_cancel_fair_whiff_fail: row.get(43)?,
            l_cancel_fair_hit_success: row.get(44)?,
            l_cancel_fair_hit_fail: row.get(45)?,
            l_cancel_bair_shield_success: row.get(46)?,
            l_cancel_bair_shield_fail: row.get(47)?,
            l_cancel_bair_whiff_success: row.get(48)?,
            l_cancel_bair_whiff_fail: row.get(49)?,
            l_cancel_bair_hit_success: row.get(50)?,
            l_cancel_bair_hit_fail: row.get(51)?,
            l_cancel_uair_shield_success: row.get(52)?,
            l_cancel_uair_shield_fail: row.get(53)?,
            l_cancel_uair_whiff_success: row.get(54)?,
            l_cancel_uair_whiff_fail: row.get(55)?,
            l_cancel_uair_hit_success: row.get(56)?,
            l_cancel_uair_hit_fail: row.get(57)?,
            l_cancel_dair_shield_success: row.get(58)?,
            l_cancel_dair_shield_fail: row.get(59)?,
            l_cancel_dair_whiff_success: row.get(60)?,
            l_cancel_dair_whiff_fail: row.get(61)?,
            l_cancel_dair_hit_success: row.get(62)?,
            l_cancel_dair_hit_fail: row.get(63)?,
            shield_grab_count: row.get(64)?,
            stocks_remaining: row.get(65)?,
            final_percent: row.get(66)?,
            slp_path: row.get(67)?,
        })
    })?;
    
    rows.collect()
}

// ============================================================================
// AGGREGATED STATS OPERATIONS
// ============================================================================

/// Filter options for aggregated stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsFilter {
    /// Filter by opponent character ID (what you played AGAINST)
    pub opponent_character_id: Option<i32>,
    /// Filter by your own character ID (what you played AS)
    pub player_character_id: Option<i32>,
    /// Filter by stage ID
    pub stage_id: Option<i32>,
    /// Filter by start time (ISO8601 format, games after this time)
    pub start_time: Option<String>,
    /// Filter by end time (ISO8601 format, games before this time)
    pub end_time: Option<String>,
    /// Filter by opponent's connect code (e.g., "MANG#0")
    pub opponent_connect_code: Option<String>,
}

/// Per-aerial breakdown of L-cancels by target type
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AerialLCancelStats {
    pub percent: f64,
    pub success: i64,
    pub total: i64,
    // Per-target breakdown for this aerial
    pub hit_success: i64,
    pub hit_fail: i64,
    pub shield_success: i64,
    pub shield_fail: i64,
    pub whiff_success: i64,
    pub whiff_fail: i64,
}

/// L-Cancel breakdown by aerial (with per-target detail for each)
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LCancelAerialBreakdown {
    pub nair: AerialLCancelStats,
    pub fair: AerialLCancelStats,
    pub bair: AerialLCancelStats,
    pub uair: AerialLCancelStats,
    pub dair: AerialLCancelStats,
}

/// L-Cancel breakdown by target type
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LCancelTargetBreakdown {
    pub hit_percent: f64,
    pub shield_percent: f64,
    pub whiff_percent: f64,
    // Raw counts for display
    pub hit_success: i64,
    pub hit_total: i64,
    pub shield_success: i64,
    pub shield_total: i64,
    pub whiff_success: i64,
    pub whiff_total: i64,
}

/// Aggregated stats for a player
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregatedPlayerStats {
    pub total_games: i64,
    pub total_wins: i64,
    pub avg_l_cancel_percent: f64,
    pub avg_rolls_per_game: f64,
    pub avg_openings_per_kill: f64,
    pub avg_damage_per_opening: f64,
    pub avg_neutral_wins: f64,
    pub avg_inputs_per_minute: f64,
    pub character_stats: Vec<CharacterWinRate>,
    pub stage_stats: Vec<StageWinRate>,
    // L-Cancel breakdown
    pub l_cancel_by_aerial: LCancelAerialBreakdown,
    pub l_cancel_by_target: LCancelTargetBreakdown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterWinRate {
    pub character_id: i32,
    pub games: i64,
    pub wins: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StageWinRate {
    pub stage_id: i32,
    pub games: i64,
    pub wins: i64,
}

/// Get aggregated stats for a specific connect code with optional filters
pub fn get_aggregated_player_stats(
    conn: &Connection, 
    connect_code: &str,
    filter: Option<StatsFilter>,
) -> rusqlite::Result<AggregatedPlayerStats> {
    let filter = filter.unwrap_or_default();
    
    // Debug: count how many player_stats exist for this connect code
    let total_player_stats: i32 = conn.query_row(
        "SELECT COUNT(*) FROM player_stats WHERE connect_code = ?",
        [connect_code],
        |row| row.get(0),
    ).unwrap_or(0);
    
    let total_game_stats: i32 = conn.query_row(
        "SELECT COUNT(*) FROM game_stats",
        [],
        |row| row.get(0),
    ).unwrap_or(0);
    
    let joined_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM player_stats p JOIN game_stats g ON p.recording_id = g.id WHERE p.connect_code = ?",
        [connect_code],
        |row| row.get(0),
    ).unwrap_or(0);
    
    log::info!("[TotalStats] connect_code={}, player_stats={}, game_stats={}, joined={}", 
        connect_code, total_player_stats, total_game_stats, joined_count);
    
    // Build dynamic WHERE clause for filters
    let mut where_clauses = vec!["p.connect_code = ?1".to_string()];
    let mut param_idx = 2;
    
    // Build params vector - start with connect_code
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(connect_code.to_string())];
    
    if let Some(stage) = filter.stage_id {
        where_clauses.push(format!("g.stage = ?{}", param_idx));
        params_vec.push(Box::new(stage));
        param_idx += 1;
    }
    
    if let Some(start) = &filter.start_time {
        where_clauses.push(format!("g.created_at >= ?{}", param_idx));
        params_vec.push(Box::new(start.clone()));
        param_idx += 1;
    }
    
    if let Some(end) = &filter.end_time {
        where_clauses.push(format!("g.created_at <= ?{}", param_idx));
        params_vec.push(Box::new(end.clone()));
        param_idx += 1;
    }
    
    if let Some(player_char) = filter.player_character_id {
        where_clauses.push(format!("p.character_id = ?{}", param_idx));
        params_vec.push(Box::new(player_char));
        param_idx += 1;
    }
    
    // Opponent filters require join with opponent player_stats
    let needs_opponent_join = filter.opponent_character_id.is_some() || filter.opponent_connect_code.is_some();
    let opponent_join = if needs_opponent_join {
        "JOIN player_stats opp_filter ON p.recording_id = opp_filter.recording_id AND opp_filter.player_index != p.player_index"
    } else {
        ""
    };
    
    if let Some(opp_char) = filter.opponent_character_id {
        where_clauses.push(format!("opp_filter.character_id = ?{}", param_idx));
        params_vec.push(Box::new(opp_char));
        param_idx += 1;
    }
    
    if let Some(ref opp_code) = filter.opponent_connect_code {
        where_clauses.push(format!("opp_filter.connect_code = ?{}", param_idx));
        params_vec.push(Box::new(opp_code.clone()));
        // param_idx not incremented since not used after this
    }
    
    let where_clause = where_clauses.join(" AND ");
    
    // 1. Overall stats
    // Winner is determined by matching connect code to the winning player's ID in game_stats
    // If winner_port=1 and player1_id=connect_code, player won. Same for port 2.
    let overall_query = format!(
        "SELECT 
            COUNT(*) as total_games,
            SUM(CASE 
                WHEN (g.winner_port = 1 AND g.player1_id = p.connect_code) THEN 1
                WHEN (g.winner_port = 2 AND g.player2_id = p.connect_code) THEN 1
                ELSE 0 
            END) as total_wins,
            AVG(
                CAST(p.l_cancel_success_count AS FLOAT) / 
                NULLIF(p.l_cancel_success_count + p.l_cancel_fail_count, 0)
            ) * 100 as avg_l_cancel,
            AVG(p.roll_count) as avg_rolls,
            AVG(p.openings_per_kill) as avg_opk,
            AVG(p.damage_per_opening) as avg_dpo,
            AVG(p.neutral_win_ratio) * 100 as avg_neutral,
            AVG(p.inputs_per_minute) as avg_ipm
         FROM player_stats p
         JOIN game_stats g ON p.recording_id = g.id
         {}
         WHERE {}",
        opponent_join, where_clause
    );
    
    log::info!("[TotalStats] Query: {}", overall_query);
    log::info!("[TotalStats] Where clause: {}", where_clause);
    log::info!("[TotalStats] Opponent join: '{}'", opponent_join);
    
    let mut stmt = conn.prepare(&overall_query)?;
    
    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    
    let (
        total_games, 
        total_wins, 
        avg_l_cancel, 
        avg_rolls,
        avg_opk,
        avg_dpo,
        avg_neutral,
        avg_ipm
    ) = stmt.query_row(
        params_slice.as_slice(),
        |row| {
            Ok((
                row.get::<_, Option<i64>>(0)?.unwrap_or(0),
                row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                row.get::<_, Option<f64>>(2)?.unwrap_or(0.0),
                row.get::<_, Option<f64>>(3)?.unwrap_or(0.0),
                row.get::<_, Option<f64>>(4)?.unwrap_or(0.0),
                row.get::<_, Option<f64>>(5)?.unwrap_or(0.0),
                row.get::<_, Option<f64>>(6)?.unwrap_or(0.0),
                row.get::<_, Option<f64>>(7)?.unwrap_or(0.0),
            ))
        }
    )?;

    // 2. Character stats (opponents faced) - with filters applied
    // Winner determined by matching connect code to winning player's ID
    // Note: This query uses 'opp' as the join alias, so replace opp_filter with opp
    let character_where = where_clause
        .replace("opp_filter.character_id", "opp.character_id")
        .replace("opp_filter.connect_code", "opp.connect_code");
    let character_query = format!(
        "SELECT 
            opp.character_id,
            COUNT(*) as games,
            SUM(CASE 
                WHEN (g.winner_port = 1 AND g.player1_id = p.connect_code) THEN 1
                WHEN (g.winner_port = 2 AND g.player2_id = p.connect_code) THEN 1
                ELSE 0 
            END) as wins
         FROM player_stats p
         JOIN game_stats g ON p.recording_id = g.id
         JOIN player_stats opp ON p.recording_id = opp.recording_id AND opp.player_index != p.player_index
         WHERE {}
         GROUP BY opp.character_id",
        character_where
    );
    
    let mut stmt = conn.prepare(&character_query)?;
    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    
    let character_stats = stmt.query_map(params_slice.as_slice(), |row| {
        Ok(CharacterWinRate {
            character_id: row.get(0)?,
            games: row.get(1)?,
            wins: row.get::<_, Option<i64>>(2)?.unwrap_or(0),
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    // 3. Stage stats - with filters applied
    // Winner determined by matching connect code to winning player's ID
    let stage_query = format!(
        "SELECT 
            g.stage,
            COUNT(*) as games,
            SUM(CASE 
                WHEN (g.winner_port = 1 AND g.player1_id = p.connect_code) THEN 1
                WHEN (g.winner_port = 2 AND g.player2_id = p.connect_code) THEN 1
                ELSE 0 
            END) as wins
         FROM player_stats p
         JOIN game_stats g ON p.recording_id = g.id
         {}
         WHERE {} AND g.stage IS NOT NULL
         GROUP BY g.stage",
        opponent_join, where_clause
    );
    
    let mut stmt = conn.prepare(&stage_query)?;
    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    
    let stage_stats = stmt.query_map(params_slice.as_slice(), |row| {
        Ok(StageWinRate {
            stage_id: row.get(0)?,
            games: row.get(1)?,
            wins: row.get::<_, Option<i64>>(2)?.unwrap_or(0),
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    // 4. L-Cancel breakdown by aerial (with per-target detail)
    // Use p. prefix on all columns to avoid ambiguity when opponent_join adds another player_stats table
    let lcancel_aerial_query = format!(
        "SELECT 
            -- Nair
            SUM(p.l_cancel_nair_hit_success) as nair_hit_success,
            SUM(p.l_cancel_nair_hit_fail) as nair_hit_fail,
            SUM(p.l_cancel_nair_shield_success) as nair_shield_success,
            SUM(p.l_cancel_nair_shield_fail) as nair_shield_fail,
            SUM(p.l_cancel_nair_whiff_success) as nair_whiff_success,
            SUM(p.l_cancel_nair_whiff_fail) as nair_whiff_fail,
            -- Fair
            SUM(p.l_cancel_fair_hit_success) as fair_hit_success,
            SUM(p.l_cancel_fair_hit_fail) as fair_hit_fail,
            SUM(p.l_cancel_fair_shield_success) as fair_shield_success,
            SUM(p.l_cancel_fair_shield_fail) as fair_shield_fail,
            SUM(p.l_cancel_fair_whiff_success) as fair_whiff_success,
            SUM(p.l_cancel_fair_whiff_fail) as fair_whiff_fail,
            -- Bair
            SUM(p.l_cancel_bair_hit_success) as bair_hit_success,
            SUM(p.l_cancel_bair_hit_fail) as bair_hit_fail,
            SUM(p.l_cancel_bair_shield_success) as bair_shield_success,
            SUM(p.l_cancel_bair_shield_fail) as bair_shield_fail,
            SUM(p.l_cancel_bair_whiff_success) as bair_whiff_success,
            SUM(p.l_cancel_bair_whiff_fail) as bair_whiff_fail,
            -- Uair
            SUM(p.l_cancel_uair_hit_success) as uair_hit_success,
            SUM(p.l_cancel_uair_hit_fail) as uair_hit_fail,
            SUM(p.l_cancel_uair_shield_success) as uair_shield_success,
            SUM(p.l_cancel_uair_shield_fail) as uair_shield_fail,
            SUM(p.l_cancel_uair_whiff_success) as uair_whiff_success,
            SUM(p.l_cancel_uair_whiff_fail) as uair_whiff_fail,
            -- Dair
            SUM(p.l_cancel_dair_hit_success) as dair_hit_success,
            SUM(p.l_cancel_dair_hit_fail) as dair_hit_fail,
            SUM(p.l_cancel_dair_shield_success) as dair_shield_success,
            SUM(p.l_cancel_dair_shield_fail) as dair_shield_fail,
            SUM(p.l_cancel_dair_whiff_success) as dair_whiff_success,
            SUM(p.l_cancel_dair_whiff_fail) as dair_whiff_fail
         FROM player_stats p
         JOIN game_stats g ON p.recording_id = g.id
         {}
         WHERE {}",
        opponent_join, where_clause
    );
    
    let mut stmt = conn.prepare(&lcancel_aerial_query)?;
    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    
    let l_cancel_by_aerial = stmt.query_row(params_slice.as_slice(), |row| {
        // Helper to build AerialLCancelStats from row data
        fn build_aerial_stats(
            hit_success: i64, hit_fail: i64,
            shield_success: i64, shield_fail: i64,
            whiff_success: i64, whiff_fail: i64,
        ) -> AerialLCancelStats {
            let success = hit_success + shield_success + whiff_success;
            let total = success + hit_fail + shield_fail + whiff_fail;
            AerialLCancelStats {
                percent: if total > 0 { (success as f64 / total as f64) * 100.0 } else { 0.0 },
                success,
                total,
                hit_success, hit_fail,
                shield_success, shield_fail,
                whiff_success, whiff_fail,
            }
        }
        
        Ok(LCancelAerialBreakdown {
            nair: build_aerial_stats(
                row.get::<_, Option<i64>>(0)?.unwrap_or(0), row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                row.get::<_, Option<i64>>(2)?.unwrap_or(0), row.get::<_, Option<i64>>(3)?.unwrap_or(0),
                row.get::<_, Option<i64>>(4)?.unwrap_or(0), row.get::<_, Option<i64>>(5)?.unwrap_or(0),
            ),
            fair: build_aerial_stats(
                row.get::<_, Option<i64>>(6)?.unwrap_or(0), row.get::<_, Option<i64>>(7)?.unwrap_or(0),
                row.get::<_, Option<i64>>(8)?.unwrap_or(0), row.get::<_, Option<i64>>(9)?.unwrap_or(0),
                row.get::<_, Option<i64>>(10)?.unwrap_or(0), row.get::<_, Option<i64>>(11)?.unwrap_or(0),
            ),
            bair: build_aerial_stats(
                row.get::<_, Option<i64>>(12)?.unwrap_or(0), row.get::<_, Option<i64>>(13)?.unwrap_or(0),
                row.get::<_, Option<i64>>(14)?.unwrap_or(0), row.get::<_, Option<i64>>(15)?.unwrap_or(0),
                row.get::<_, Option<i64>>(16)?.unwrap_or(0), row.get::<_, Option<i64>>(17)?.unwrap_or(0),
            ),
            uair: build_aerial_stats(
                row.get::<_, Option<i64>>(18)?.unwrap_or(0), row.get::<_, Option<i64>>(19)?.unwrap_or(0),
                row.get::<_, Option<i64>>(20)?.unwrap_or(0), row.get::<_, Option<i64>>(21)?.unwrap_or(0),
                row.get::<_, Option<i64>>(22)?.unwrap_or(0), row.get::<_, Option<i64>>(23)?.unwrap_or(0),
            ),
            dair: build_aerial_stats(
                row.get::<_, Option<i64>>(24)?.unwrap_or(0), row.get::<_, Option<i64>>(25)?.unwrap_or(0),
                row.get::<_, Option<i64>>(26)?.unwrap_or(0), row.get::<_, Option<i64>>(27)?.unwrap_or(0),
                row.get::<_, Option<i64>>(28)?.unwrap_or(0), row.get::<_, Option<i64>>(29)?.unwrap_or(0),
            ),
        })
    }).unwrap_or_default();

    // 5. L-Cancel breakdown by target type
    // Use p. prefix on all columns to avoid ambiguity when opponent_join adds another player_stats table
    let lcancel_target_query = format!(
        "SELECT 
            SUM(p.l_cancel_nair_hit_success + p.l_cancel_fair_hit_success + p.l_cancel_bair_hit_success + 
                p.l_cancel_uair_hit_success + p.l_cancel_dair_hit_success) as hit_success,
            SUM(p.l_cancel_nair_hit_success + p.l_cancel_fair_hit_success + p.l_cancel_bair_hit_success + 
                p.l_cancel_uair_hit_success + p.l_cancel_dair_hit_success +
                p.l_cancel_nair_hit_fail + p.l_cancel_fair_hit_fail + p.l_cancel_bair_hit_fail + 
                p.l_cancel_uair_hit_fail + p.l_cancel_dair_hit_fail) as hit_total,
            SUM(p.l_cancel_nair_shield_success + p.l_cancel_fair_shield_success + p.l_cancel_bair_shield_success + 
                p.l_cancel_uair_shield_success + p.l_cancel_dair_shield_success) as shield_success,
            SUM(p.l_cancel_nair_shield_success + p.l_cancel_fair_shield_success + p.l_cancel_bair_shield_success + 
                p.l_cancel_uair_shield_success + p.l_cancel_dair_shield_success +
                p.l_cancel_nair_shield_fail + p.l_cancel_fair_shield_fail + p.l_cancel_bair_shield_fail + 
                p.l_cancel_uair_shield_fail + p.l_cancel_dair_shield_fail) as shield_total,
            SUM(p.l_cancel_nair_whiff_success + p.l_cancel_fair_whiff_success + p.l_cancel_bair_whiff_success + 
                p.l_cancel_uair_whiff_success + p.l_cancel_dair_whiff_success) as whiff_success,
            SUM(p.l_cancel_nair_whiff_success + p.l_cancel_fair_whiff_success + p.l_cancel_bair_whiff_success + 
                p.l_cancel_uair_whiff_success + p.l_cancel_dair_whiff_success +
                p.l_cancel_nair_whiff_fail + p.l_cancel_fair_whiff_fail + p.l_cancel_bair_whiff_fail + 
                p.l_cancel_uair_whiff_fail + p.l_cancel_dair_whiff_fail) as whiff_total
         FROM player_stats p
         JOIN game_stats g ON p.recording_id = g.id
         {}
         WHERE {}",
        opponent_join, where_clause
    );
    
    let mut stmt = conn.prepare(&lcancel_target_query)?;
    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    
    let l_cancel_by_target = stmt.query_row(params_slice.as_slice(), |row| {
        let hit_success: i64 = row.get::<_, Option<i64>>(0)?.unwrap_or(0);
        let hit_total: i64 = row.get::<_, Option<i64>>(1)?.unwrap_or(0);
        let shield_success: i64 = row.get::<_, Option<i64>>(2)?.unwrap_or(0);
        let shield_total: i64 = row.get::<_, Option<i64>>(3)?.unwrap_or(0);
        let whiff_success: i64 = row.get::<_, Option<i64>>(4)?.unwrap_or(0);
        let whiff_total: i64 = row.get::<_, Option<i64>>(5)?.unwrap_or(0);
        
        Ok(LCancelTargetBreakdown {
            hit_percent: if hit_total > 0 { (hit_success as f64 / hit_total as f64) * 100.0 } else { 0.0 },
            shield_percent: if shield_total > 0 { (shield_success as f64 / shield_total as f64) * 100.0 } else { 0.0 },
            whiff_percent: if whiff_total > 0 { (whiff_success as f64 / whiff_total as f64) * 100.0 } else { 0.0 },
            hit_success, hit_total,
            shield_success, shield_total,
            whiff_success, whiff_total,
        })
    }).unwrap_or_default();

    Ok(AggregatedPlayerStats {
        total_games,
        total_wins,
        avg_l_cancel_percent: avg_l_cancel,
        avg_rolls_per_game: avg_rolls,
        avg_openings_per_kill: avg_opk,
        avg_damage_per_opening: avg_dpo,
        avg_neutral_wins: avg_neutral,
        avg_inputs_per_minute: avg_ipm,
        character_stats,
        stage_stats,
        l_cancel_by_aerial,
        l_cancel_by_target,
    })
}

/// Available filter options for stats page (only values that exist in the database)
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableFilterOptions {
    /// All unique connect codes in the database
    pub connect_codes: Vec<String>,
    /// All character IDs that players have played as
    pub player_characters: Vec<i32>,
    /// All character IDs that opponents have played as
    pub opponent_characters: Vec<i32>,
    /// All stage IDs that games have been played on
    pub stages: Vec<i32>,
}

/// Get available filter options from the database, optionally filtered by a player's connect code
pub fn get_available_filter_options(conn: &Connection, connect_code: Option<&str>) -> rusqlite::Result<AvailableFilterOptions> {
    // Get all unique connect codes
    let mut stmt = conn.prepare(
        "SELECT DISTINCT connect_code FROM player_stats WHERE connect_code IS NOT NULL ORDER BY connect_code"
    )?;
    let connect_codes: Vec<String> = stmt.query_map([], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    // Get characters and stages based on whether we have a specific player
    let (player_characters, opponent_characters, stages) = if let Some(code) = connect_code {
        // Get characters this player has played as
        let mut stmt = conn.prepare(
            "SELECT DISTINCT character_id FROM player_stats WHERE connect_code = ?1 ORDER BY character_id"
        )?;
        let player_chars: Vec<i32> = stmt.query_map([code], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        // Get characters this player has faced (opponent's characters in games where this player participated)
        let mut stmt = conn.prepare(
            "SELECT DISTINCT opp.character_id 
             FROM player_stats p
             JOIN player_stats opp ON p.recording_id = opp.recording_id AND opp.player_index != p.player_index
             WHERE p.connect_code = ?1
             ORDER BY opp.character_id"
        )?;
        let opp_chars: Vec<i32> = stmt.query_map([code], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        // Get stages this player has played on
        let mut stmt = conn.prepare(
            "SELECT DISTINCT g.stage 
             FROM player_stats p
             JOIN game_stats g ON p.recording_id = g.id
             WHERE p.connect_code = ?1 AND g.stage IS NOT NULL
             ORDER BY g.stage"
        )?;
        let player_stages: Vec<i32> = stmt.query_map([code], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        (player_chars, opp_chars, player_stages)
    } else {
        // No player filter - return all
        let mut stmt = conn.prepare(
            "SELECT DISTINCT character_id FROM player_stats ORDER BY character_id"
        )?;
        let characters: Vec<i32> = stmt.query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        let mut stmt = conn.prepare(
            "SELECT DISTINCT stage FROM game_stats WHERE stage IS NOT NULL ORDER BY stage"
        )?;
        let all_stages: Vec<i32> = stmt.query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        (characters.clone(), characters, all_stages)
    };

    Ok(AvailableFilterOptions {
        connect_codes,
        player_characters,
        opponent_characters,
        stages,
    })
}

/// Time series data point for chart visualization
#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSeriesDataPoint {
    pub date: String,
    pub l_cancel_percent: Option<f64>,
    pub win: bool,
    pub inputs_per_minute: Option<f64>,
    pub openings_per_kill: Option<f64>,
    pub damage_per_opening: Option<f64>,
    pub neutral_win_ratio: Option<f64>,
    pub roll_count: Option<f64>,
}

/// Get per-game stats as time series for chart visualization
pub fn get_player_stats_timeseries(
    conn: &Connection,
    connect_code: &str,
    filter: Option<StatsFilter>,
) -> rusqlite::Result<Vec<TimeSeriesDataPoint>> {
    let filter = filter.unwrap_or_default();
    
    // Build dynamic WHERE clause
    let mut where_clauses = vec!["p.connect_code = ?1".to_string()];
    let mut param_idx = 2;
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(connect_code.to_string())];
    
    if let Some(stage) = filter.stage_id {
        where_clauses.push(format!("g.stage = ?{}", param_idx));
        params_vec.push(Box::new(stage));
        param_idx += 1;
    }
    
    if let Some(start) = &filter.start_time {
        where_clauses.push(format!("g.created_at >= ?{}", param_idx));
        params_vec.push(Box::new(start.clone()));
        param_idx += 1;
    }
    
    if let Some(end) = &filter.end_time {
        where_clauses.push(format!("g.created_at <= ?{}", param_idx));
        params_vec.push(Box::new(end.clone()));
        param_idx += 1;
    }
    
    if let Some(player_char) = filter.player_character_id {
        where_clauses.push(format!("p.character_id = ?{}", param_idx));
        params_vec.push(Box::new(player_char));
        param_idx += 1;
    }
    
    // Handle opponent filters (character and/or connect code)
    let needs_opponent_join = filter.opponent_character_id.is_some() || filter.opponent_connect_code.is_some();
    let opponent_join = if needs_opponent_join {
        "JOIN player_stats opp_filter ON p.recording_id = opp_filter.recording_id AND opp_filter.player_index != p.player_index"
    } else {
        ""
    };
    
    if let Some(opp_char) = filter.opponent_character_id {
        where_clauses.push(format!("opp_filter.character_id = ?{}", param_idx));
        params_vec.push(Box::new(opp_char));
        param_idx += 1;
    }
    
    if let Some(ref opp_code) = filter.opponent_connect_code {
        where_clauses.push(format!("opp_filter.connect_code = ?{}", param_idx));
        params_vec.push(Box::new(opp_code.clone()));
        // param_idx not incremented since not used after this
    }
    
    let where_clause = where_clauses.join(" AND ");
    
    // Query to get per-game stats with date
    let query = format!(
        "SELECT 
            g.created_at,
            CASE WHEN p.l_cancel_success_count + p.l_cancel_fail_count > 0 
                THEN CAST(p.l_cancel_success_count AS REAL) / (p.l_cancel_success_count + p.l_cancel_fail_count) * 100 
                ELSE NULL 
            END as l_cancel_percent,
            CASE 
                WHEN g.winner_port IS NOT NULL AND (
                    (g.player1_port = p.port AND g.winner_port = g.player1_port) OR
                    (g.player2_port = p.port AND g.winner_port = g.player2_port)
                ) THEN 1
                ELSE 0
            END as win,
            p.inputs_per_minute,
            p.openings_per_kill,
            p.damage_per_opening,
            p.neutral_win_ratio,
            CAST(p.roll_count AS REAL) as roll_count
         FROM player_stats p
         JOIN game_stats g ON p.recording_id = g.id
         {}
         WHERE {}
         ORDER BY g.created_at ASC",
        opponent_join, where_clause
    );
    
    let mut stmt = conn.prepare(&query)?;
    let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    
    let results = stmt.query_map(params_slice.as_slice(), |row| {
        Ok(TimeSeriesDataPoint {
            date: row.get::<_, String>(0)?,
            l_cancel_percent: row.get(1)?,
            win: row.get::<_, i32>(2)? == 1,
            inputs_per_minute: row.get(3)?,
            openings_per_kill: row.get(4)?,
            damage_per_opening: row.get(5)?,
            neutral_win_ratio: row.get(6)?,
            roll_count: row.get(7)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    
    log::debug!("Time series query returned {} data points for {}", results.len(), connect_code);
    
    Ok(results)
}

// ============================================================================
// STORAGE MANAGEMENT QUERIES
// ============================================================================

/// Get total storage used by recordings (excluding clips)
/// Returns the sum of file_size in bytes for all recordings where video_path does not contain "Clips"
pub fn get_total_storage_bytes(conn: &Connection) -> Result<i64, rusqlite::Error> {
    conn.query_row(
        "SELECT COALESCE(SUM(file_size), 0) FROM recordings WHERE video_path NOT LIKE '%Clips%'",
        [],
        |row| row.get(0),
    )
}

/// Get count of recordings (excluding clips)
pub fn get_recording_count(conn: &Connection) -> Result<i32, rusqlite::Error> {
    conn.query_row(
        "SELECT COUNT(*) FROM recordings WHERE video_path NOT LIKE '%Clips%'",
        [],
        |row| row.get(0),
    )
}

/// Get oldest recordings ordered by start_time (excluding clips)
/// Used for storage cleanup - returns recordings that should be deleted first
pub fn get_oldest_recordings(conn: &Connection, limit: i32) -> Result<Vec<RecordingRow>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT id, video_path, slp_path, file_size, file_modified_at, thumbnail_path, start_time, cached_at, needs_reparse 
         FROM recordings 
         WHERE video_path NOT LIKE '%Clips%'
         ORDER BY start_time ASC 
         LIMIT ?"
    )?;
    
    let recordings = stmt.query_map([limit], |row| {
        Ok(RecordingRow {
            id: row.get(0)?,
            video_path: row.get(1)?,
            slp_path: row.get(2)?,
            file_size: row.get(3)?,
            file_modified_at: row.get(4)?,
            thumbnail_path: row.get(5)?,
            start_time: row.get(6)?,
            cached_at: row.get(7)?,
            needs_reparse: row.get(8)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    
    Ok(recordings)
}
