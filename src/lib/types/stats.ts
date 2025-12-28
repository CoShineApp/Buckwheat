/**
 * Player statistics type definitions.
 * Used for tracking and displaying per-game Melee stats.
 * @module types/stats
 */

/**
 * Per-game stats for a single player.
 * Extracted from .slp replay files after each match.
 */
export interface PlayerGameStats {
	/** Unique identifier for this stats record */
	id: string;
	/** User ID if authenticated and synced to cloud */
	user_id: string | null;
	/** Device ID for local tracking */
	device_id: string;
	/** Path to the .slp file */
	slp_file_path: string;
	/** ID of the recording session */
	recording_id: string;
	
	/** ISO timestamp when the game was played */
	game_date: string;
	/** Melee stage ID */
	stage_id: number;
	/** Game duration in frames (60 fps) */
	game_duration_frames: number;
	
	/** Player's port number (1-4) */
	player_port: number;
	/** Player's tag/name */
	player_tag: string;
	/** Player's character ID */
	character_id: number;
	/** Opponent's character ID (null for non-1v1) */
	opponent_character_id: number | null;
	
	// L-Cancel stats
	/** Successful L-cancels */
	l_cancel_hit: number;
	/** Missed L-cancels */
	l_cancel_missed: number;
	
	// Neutral & opening stats
	/** Number of times player won neutral */
	neutral_wins: number;
	/** Number of times opponent won neutral */
	neutral_losses: number;
	/** Total openings created */
	openings: number;
	/** Average damage dealt per opening */
	damage_per_opening: number | null;
	/** Openings needed per kill (conversion rate) */
	openings_per_kill: number | null;
	
	// Kill stats
	/** Total kills */
	kills: number;
	/** Total deaths */
	deaths: number;
	/** Average opponent percent when killed */
	avg_kill_percent: number | null;
	/** Total damage dealt to opponent */
	total_damage_dealt: number;
	/** Total damage taken from opponent */
	total_damage_taken: number;
	
	// Tech skill stats
	/** Successful techs */
	successful_techs: number;
	/** Missed techs */
	missed_techs: number;
	/** Number of wavedashes */
	wavedash_count: number;
	/** Number of dashdances */
	dashdance_count: number;
	
	// Input stats
	/** Actions per minute */
	apm: number;
	/** Grab attempts */
	grab_attempts: number;
	/** Successful grabs */
	grab_success: number;
	
	/** Whether stats have been synced to cloud */
	synced_to_cloud: boolean;
	/** ISO timestamp when stats were created */
	created_at: string;
	/** ISO timestamp when stats were last updated */
	updated_at: string;
}

/**
 * Aggregated statistics across multiple games for a player.
 */
export interface AggregateStats {
	/** Player's tag/name */
	player_tag: string;
	/** Total games played */
	total_games: number;
	/** Total wins (kills > deaths) */
	total_wins: number;
	/** Total losses (deaths > kills) */
	total_losses: number;
	/** Average L-cancel success rate (%) */
	avg_l_cancel_rate: number;
	/** Average tech success rate (%) */
	avg_tech_rate: number;
	/** Average actions per minute */
	avg_apm: number;
	/** Average openings needed per kill */
	avg_openings_per_kill: number;
	/** Average damage dealt per opening */
	avg_damage_per_opening: number;
	/** Total wavedashes across all games */
	total_wavedashes: number;
	/** Total dashdances across all games */
	total_dashdances: number;
}

/**
 * Result of a cloud sync operation.
 */
export interface SyncResult {
	/** Number of stats records successfully synced */
	synced_count: number;
	/** Number of stats records that failed to sync */
	failed_count: number;
}

/**
 * Derived/calculated stats fields that can be computed client-side.
 */
export interface DerivedStats {
	/** L-cancel success rate (%) */
	l_cancel_rate: number;
	/** Tech success rate (%) */
	tech_rate: number;
	/** Win rate (%) */
	win_rate: number;
	/** Damage efficiency (damage dealt / damage taken) */
	damage_efficiency: number;
}

/**
 * Calculate derived stats from raw player game stats.
 */
export function calculateDerivedStats(stats: PlayerGameStats): DerivedStats {
	const total_l_cancels = stats.l_cancel_hit + stats.l_cancel_missed;
	const l_cancel_rate = total_l_cancels > 0 
		? (stats.l_cancel_hit / total_l_cancels) * 100 
		: 0;
	
	const total_techs = stats.successful_techs + stats.missed_techs;
	const tech_rate = total_techs > 0 
		? (stats.successful_techs / total_techs) * 100 
		: 0;
	
	const total_stocks = stats.kills + stats.deaths;
	const win_rate = total_stocks > 0 
		? (stats.kills / total_stocks) * 100 
		: 0;
	
	const damage_efficiency = stats.total_damage_taken > 0
		? stats.total_damage_dealt / stats.total_damage_taken
		: stats.total_damage_dealt;
	
	return {
		l_cancel_rate,
		tech_rate,
		win_rate,
		damage_efficiency,
	};
}

