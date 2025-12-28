// CRUD operations for player game stats

use crate::commands::errors::Error;
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerGameStats {
    pub id: String,
    pub user_id: Option<String>,
    pub device_id: String,
    pub slp_file_path: String,
    pub recording_id: String,
    
    // Game metadata
    pub game_date: String,
    pub stage_id: u16,
    pub game_duration_frames: i32,
    
    // Player info
    pub player_port: u8,
    pub player_tag: String,
    pub character_id: u8,
    pub opponent_character_id: Option<u8>,
    
    // L-Cancel stats
    pub l_cancel_hit: i32,
    pub l_cancel_missed: i32,
    
    // Neutral & opening stats
    pub neutral_wins: i32,
    pub neutral_losses: i32,
    pub openings: i32,
    pub damage_per_opening: Option<f64>,
    pub openings_per_kill: Option<f64>,
    
    // Kill stats
    pub kills: i32,
    pub deaths: i32,
    pub avg_kill_percent: Option<f64>,
    pub total_damage_dealt: f64,
    pub total_damage_taken: f64,
    
    // Tech skill stats
    pub successful_techs: i32,
    pub missed_techs: i32,
    pub wavedash_count: i32,
    pub dashdance_count: i32,
    
    // Input stats
    pub apm: f64,
    pub grab_attempts: i32,
    pub grab_success: i32,
    
    // Metadata
    pub synced_to_cloud: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl PlayerGameStats {
    /// Map a database row to PlayerGameStats
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            user_id: row.get(1)?,
            device_id: row.get(2)?,
            slp_file_path: row.get(3)?,
            recording_id: row.get(4)?,
            game_date: row.get(5)?,
            stage_id: row.get::<_, i64>(6)? as u16,
            game_duration_frames: row.get::<_, i64>(7)? as i32,
            player_port: row.get::<_, i64>(8)? as u8,
            player_tag: row.get(9)?,
            character_id: row.get::<_, i64>(10)? as u8,
            opponent_character_id: row.get::<_, Option<i64>>(11)?.map(|v| v as u8),
            l_cancel_hit: row.get::<_, i64>(12)? as i32,
            l_cancel_missed: row.get::<_, i64>(13)? as i32,
            neutral_wins: row.get::<_, i64>(14)? as i32,
            neutral_losses: row.get::<_, i64>(15)? as i32,
            openings: row.get::<_, i64>(16)? as i32,
            damage_per_opening: row.get(17)?,
            openings_per_kill: row.get(18)?,
            kills: row.get::<_, i64>(19)? as i32,
            deaths: row.get::<_, i64>(20)? as i32,
            avg_kill_percent: row.get(21)?,
            total_damage_dealt: row.get(22)?,
            total_damage_taken: row.get(23)?,
            successful_techs: row.get::<_, i64>(24)? as i32,
            missed_techs: row.get::<_, i64>(25)? as i32,
            wavedash_count: row.get::<_, i64>(26)? as i32,
            dashdance_count: row.get::<_, i64>(27)? as i32,
            apm: row.get(28)?,
            grab_attempts: row.get::<_, i64>(29)? as i32,
            grab_success: row.get::<_, i64>(30)? as i32,
            synced_to_cloud: row.get::<_, i64>(31)? != 0,
            created_at: row.get(32)?,
            updated_at: row.get(33)?,
        })
    }
}

/// Insert a new player stats record
pub fn insert_stats(
    conn: Arc<Mutex<Connection>>,
    stats: &PlayerGameStats,
) -> Result<(), Error> {
    let conn = conn.lock().unwrap();
    
    conn.execute(
        "INSERT INTO player_game_stats (
            id, user_id, device_id, slp_file_path, recording_id,
            game_date, stage_id, game_duration_frames,
            player_port, player_tag, character_id, opponent_character_id,
            l_cancel_hit, l_cancel_missed,
            neutral_wins, neutral_losses, openings, damage_per_opening, openings_per_kill,
            kills, deaths, avg_kill_percent, total_damage_dealt, total_damage_taken,
            successful_techs, missed_techs, wavedash_count, dashdance_count,
            apm, grab_attempts, grab_success,
            synced_to_cloud, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34)",
        params![
            stats.id,
            stats.user_id,
            stats.device_id,
            stats.slp_file_path,
            stats.recording_id,
            stats.game_date,
            stats.stage_id as i64,
            stats.game_duration_frames as i64,
            stats.player_port as i64,
            stats.player_tag,
            stats.character_id as i64,
            stats.opponent_character_id.map(|v| v as i64),
            stats.l_cancel_hit as i64,
            stats.l_cancel_missed as i64,
            stats.neutral_wins as i64,
            stats.neutral_losses as i64,
            stats.openings as i64,
            stats.damage_per_opening,
            stats.openings_per_kill,
            stats.kills as i64,
            stats.deaths as i64,
            stats.avg_kill_percent,
            stats.total_damage_dealt,
            stats.total_damage_taken,
            stats.successful_techs as i64,
            stats.missed_techs as i64,
            stats.wavedash_count as i64,
            stats.dashdance_count as i64,
            stats.apm,
            stats.grab_attempts as i64,
            stats.grab_success as i64,
            if stats.synced_to_cloud { 1 } else { 0 },
            stats.created_at,
            stats.updated_at,
        ],
    )
    .map_err(|e| Error::RecordingFailed(format!("Failed to insert stats: {}", e)))?;
    
    Ok(())
}

/// Get stats for a specific recording
pub fn get_stats_by_recording(
    conn: Arc<Mutex<Connection>>,
    recording_id: &str,
) -> Result<Vec<PlayerGameStats>, Error> {
    let conn = conn.lock().unwrap();
    
    let mut stmt = conn
        .prepare(
            "SELECT id, user_id, device_id, slp_file_path, recording_id,
                    game_date, stage_id, game_duration_frames,
                    player_port, player_tag, character_id, opponent_character_id,
                    l_cancel_hit, l_cancel_missed,
                    neutral_wins, neutral_losses, openings, damage_per_opening, openings_per_kill,
                    kills, deaths, avg_kill_percent, total_damage_dealt, total_damage_taken,
                    successful_techs, missed_techs, wavedash_count, dashdance_count,
                    apm, grab_attempts, grab_success,
                    synced_to_cloud, created_at, updated_at
             FROM player_game_stats
             WHERE recording_id = ?1
             ORDER BY player_port",
        )
        .map_err(|e| Error::RecordingFailed(format!("Failed to prepare statement: {}", e)))?;
    
    let stats_iter = stmt
        .query_map(params![recording_id], |row| PlayerGameStats::from_row(row))
        .map_err(|e| Error::RecordingFailed(format!("Failed to query stats: {}", e)))?;
    
    let mut stats = Vec::new();
    for stat_result in stats_iter {
        stats.push(stat_result.map_err(|e| {
            Error::RecordingFailed(format!("Failed to parse stat row: {}", e))
        })?);
    }
    
    Ok(stats)
}

/// Query stats with filters
pub fn query_stats(
    conn: Arc<Mutex<Connection>>,
    player_tag: Option<String>,
    character_id: Option<u8>,
    limit: Option<i32>,
) -> Result<Vec<PlayerGameStats>, Error> {
    let conn = conn.lock().unwrap();
    
    log::debug!("📊 Querying stats with filters - player_tag: {:?}, character_id: {:?}, limit: {:?}", 
        player_tag, character_id, limit);
    
    let mut query = "SELECT id, user_id, device_id, slp_file_path, recording_id,
                           game_date, stage_id, game_duration_frames,
                           player_port, player_tag, character_id, opponent_character_id,
                           l_cancel_hit, l_cancel_missed,
                           neutral_wins, neutral_losses, openings, damage_per_opening, openings_per_kill,
                           kills, deaths, avg_kill_percent, total_damage_dealt, total_damage_taken,
                           successful_techs, missed_techs, wavedash_count, dashdance_count,
                           apm, grab_attempts, grab_success,
                           synced_to_cloud, created_at, updated_at
                     FROM player_game_stats WHERE 1=1".to_string();
    
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    
    if let Some(ref tag) = player_tag {
        query.push_str(&format!(" AND player_tag = ?{}", params.len() + 1));
        params.push(Box::new(tag.clone()));
    }
    if let Some(char_id) = character_id {
        query.push_str(&format!(" AND character_id = ?{}", params.len() + 1));
        params.push(Box::new(char_id as i64));
    }
    
    query.push_str(" ORDER BY game_date DESC");
    
    if let Some(lim) = limit {
        query.push_str(&format!(" LIMIT {}", lim));
    }
    
    log::debug!("📊 Query: {}", query);
    log::debug!("📊 Params count: {}", params.len());
    
    let mut stmt = conn
        .prepare(&query)
        .map_err(|e| Error::RecordingFailed(format!("Failed to prepare statement: {}", e)))?;
    
    let stats_iter = stmt
        .query_map(
            rusqlite::params_from_iter(params.iter()),
            |row| PlayerGameStats::from_row(row),
        )
        .map_err(|e| Error::RecordingFailed(format!("Failed to query stats: {}", e)))?;
    
    let mut stats = Vec::new();
    for stat_result in stats_iter {
        stats.push(stat_result.map_err(|e| {
            Error::RecordingFailed(format!("Failed to parse stat row: {}", e))
        })?);
    }
    
    log::info!("📊 Found {} stats records", stats.len());
    
    Ok(stats)
}

/// Get unsynced stats for cloud upload
pub fn get_unsynced_stats(
    conn: Arc<Mutex<Connection>>,
) -> Result<Vec<PlayerGameStats>, Error> {
    let conn = conn.lock().unwrap();
    
    let mut stmt = conn
        .prepare(
            "SELECT id, user_id, device_id, slp_file_path, recording_id,
                    game_date, stage_id, game_duration_frames,
                    player_port, player_tag, character_id, opponent_character_id,
                    l_cancel_hit, l_cancel_missed,
                    neutral_wins, neutral_losses, openings, damage_per_opening, openings_per_kill,
                    kills, deaths, avg_kill_percent, total_damage_dealt, total_damage_taken,
                    successful_techs, missed_techs, wavedash_count, dashdance_count,
                    apm, grab_attempts, grab_success,
                    synced_to_cloud, created_at, updated_at
             FROM player_game_stats
             WHERE synced_to_cloud = 0 AND user_id IS NOT NULL
             ORDER BY game_date ASC",
        )
        .map_err(|e| Error::RecordingFailed(format!("Failed to prepare statement: {}", e)))?;
    
    let stats_iter = stmt
        .query_map([], |row| PlayerGameStats::from_row(row))
        .map_err(|e| Error::RecordingFailed(format!("Failed to query unsynced stats: {}", e)))?;
    
    let mut stats = Vec::new();
    for stat_result in stats_iter {
        stats.push(stat_result.map_err(|e| {
            Error::RecordingFailed(format!("Failed to parse stat row: {}", e))
        })?);
    }
    
    Ok(stats)
}

/// Mark stats as synced to cloud
pub fn mark_synced(
    conn: Arc<Mutex<Connection>>,
    stat_ids: &[String],
) -> Result<(), Error> {
    let conn = conn.lock().unwrap();
    
    for id in stat_ids {
        conn.execute(
            "UPDATE player_game_stats SET synced_to_cloud = 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| Error::RecordingFailed(format!("Failed to mark stat as synced: {}", e)))?;
    }
    
    Ok(())
}

