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
}

/// Combined recording with its stats (for paginated queries)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingWithStats {
    pub recording: RecordingRow,
    pub stats: Option<GameStatsRow>,
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
    pub stocks_remaining: i32,
    pub final_percent: Option<f64>,
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

/// Get recordings with pagination, joined with game_stats
pub fn get_recordings_paginated(
    conn: &Connection, 
    limit: i32, 
    offset: i32
) -> rusqlite::Result<(Vec<RecordingWithStats>, i32)> {
    // Get total count
    let total: i32 = conn.query_row(
        "SELECT COUNT(*) FROM recordings",
        [],
        |row| row.get(0),
    )?;
    
    // Get paginated rows with stats
    let mut stmt = conn.prepare(
        "SELECT r.id, r.video_path, r.slp_path, r.file_size, r.file_modified_at, 
                r.thumbnail_path, r.start_time, r.cached_at, r.needs_reparse,
                g.player1_id, g.player2_id, g.player1_port, g.player2_port,
                g.player1_character, g.player2_character, g.player1_color, g.player2_color,
                g.winner_port, g.loser_port, g.stage, g.game_duration, g.total_frames,
                g.is_pal, g.played_on
         FROM recordings r
         LEFT JOIN game_stats g ON r.id = g.id
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
            })
        } else {
            None
        };
        
        Ok(RecordingWithStats { recording, stats })
    })?;
    
    let results: Vec<RecordingWithStats> = rows.collect::<Result<Vec<_>, _>>()?;
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

/// Insert or update game stats
pub fn upsert_game_stats(conn: &Connection, stats: &GameStatsRow) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO game_stats (id, player1_id, player2_id, player1_port, player2_port,
                                  player1_character, player2_character, player1_color, player2_color,
                                  winner_port, loser_port, stage, game_duration, total_frames,
                                  is_pal, played_on)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
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
            played_on = excluded.played_on",
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
        ],
    )?;
    Ok(())
}

/// Get game stats by recording ID
pub fn get_game_stats_by_id(conn: &Connection, id: &str) -> rusqlite::Result<Option<GameStatsRow>> {
    conn.query_row(
        "SELECT id, player1_id, player2_id, player1_port, player2_port,
                player1_character, player2_character, player1_color, player2_color,
                winner_port, loser_port, stage, game_duration, total_frames,
                is_pal, played_on
         FROM game_stats WHERE id = ?",
        params![id],
        |row| {
            Ok(GameStatsRow {
                id: row.get(0)?,
                player1_id: row.get(1)?,
                player2_id: row.get(2)?,
                player1_port: row.get(3)?,
                player2_port: row.get(4)?,
                player1_character: row.get(5)?,
                player2_character: row.get(6)?,
                player1_color: row.get(7)?,
                player2_color: row.get(8)?,
                winner_port: row.get(9)?,
                loser_port: row.get(10)?,
                stage: row.get(11)?,
                game_duration: row.get(12)?,
                total_frames: row.get(13)?,
                is_pal: row.get::<_, Option<i32>>(14)?.map(|v| v != 0),
                played_on: row.get(15)?,
            })
        },
    ).optional()
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
            l_cancel_success_count, l_cancel_fail_count, stocks_remaining, final_percent
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
            ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35
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
            stocks_remaining = excluded.stocks_remaining,
            final_percent = excluded.final_percent",
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
            stats.stocks_remaining,
            stats.final_percent,
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
                l_cancel_success_count, l_cancel_fail_count, stocks_remaining, final_percent
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
            stocks_remaining: row.get(34)?,
            final_percent: row.get(35)?,
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
        where_clauses.push(format!("r.start_time >= ?{}", param_idx));
        params_vec.push(Box::new(start.clone()));
        param_idx += 1;
    }
    
    if let Some(end) = &filter.end_time {
        where_clauses.push(format!("r.start_time <= ?{}", param_idx));
        params_vec.push(Box::new(end.clone()));
        param_idx += 1;
    }
    
    if let Some(player_char) = filter.player_character_id {
        where_clauses.push(format!("p.character_id = ?{}", param_idx));
        params_vec.push(Box::new(player_char));
        param_idx += 1;
    }
    
    // Opponent character filter requires join with opponent player_stats
    let opponent_join = if filter.opponent_character_id.is_some() {
        "JOIN player_stats opp_filter ON p.recording_id = opp_filter.recording_id AND opp_filter.player_index != p.player_index"
    } else {
        ""
    };
    
    if let Some(opp_char) = filter.opponent_character_id {
        where_clauses.push(format!("opp_filter.character_id = ?{}", param_idx));
        params_vec.push(Box::new(opp_char));
        // param_idx not incremented since not used after this
    }
    
    let where_clause = where_clauses.join(" AND ");
    
    // 1. Overall stats
    let overall_query = format!(
        "SELECT 
            COUNT(*) as total_games,
            SUM(CASE WHEN p.port = g.winner_port THEN 1 ELSE 0 END) as total_wins,
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
         JOIN recordings r ON p.recording_id = r.id
         {}
         WHERE {}",
        opponent_join, where_clause
    );
    
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
    let character_query = format!(
        "SELECT 
            opp.character_id,
            COUNT(*) as games,
            SUM(CASE WHEN p.port = g.winner_port THEN 1 ELSE 0 END) as wins
         FROM player_stats p
         JOIN game_stats g ON p.recording_id = g.id
         JOIN recordings r ON p.recording_id = r.id
         JOIN player_stats opp ON p.recording_id = opp.recording_id AND opp.player_index != p.player_index
         {}
         WHERE {}
         GROUP BY opp.character_id",
        if filter.opponent_character_id.is_some() { "" } else { "" }, // opponent join already handled by opp
        where_clause
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
    let stage_query = format!(
        "SELECT 
            g.stage,
            COUNT(*) as games,
            SUM(CASE WHEN p.port = g.winner_port THEN 1 ELSE 0 END) as wins
         FROM player_stats p
         JOIN game_stats g ON p.recording_id = g.id
         JOIN recordings r ON p.recording_id = r.id
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
    })
}
