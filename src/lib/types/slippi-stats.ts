/**
 * Type definitions for slippi-js computed stats.
 * These are the stats computed by getStats() from @slippi/slippi-js.
 * @module types/slippi-stats
 */

/**
 * Player stats to save to database (flattened for a single player).
 * This is what we send to the Rust backend.
 */
export interface PlayerStatsForDB {
	// Identification
	playerIndex: number;
	connectCode: string | null;
	displayName: string | null;
	characterId: number;
	characterColor: number;
	port: number;

	// Overall performance
	totalDamage: number;
	killCount: number;
	conversionCount: number;
	successfulConversions: number;
	openingsPerKill: number | null;
	damagePerOpening: number | null;
	neutralWinRatio: number | null;
	counterHitRatio: number | null;
	beneficialTradeRatio: number | null;

	// Input stats
	inputsTotal: number;
	inputsPerMinute: number | null;
	avgKillPercent: number | null;

	// Action counts
	wavedashCount: number;
	wavelandCount: number;
	airDodgeCount: number;
	dashDanceCount: number;
	spotDodgeCount: number;
	ledgegrabCount: number;
	rollCount: number;
	grabCount: number;
	throwCount: number;
	groundTechCount: number;
	wallTechCount: number;
	wallJumpTechCount: number;

	// L-Cancel stats
	lCancelSuccessCount: number;
	lCancelFailCount: number;

	// Final stock state
	stocksRemaining: number;
	finalPercent: number | null;
}

/**
 * A single conversion/opening for display in the UI.
 * Computed on-the-fly from the .slp file.
 */
export interface ConversionForDisplay {
	/** Player who performed the conversion */
	playerIndex: number;
	/** Start time formatted as "M:SS" */
	startTime: string;
	/** End time formatted as "M:SS" */
	endTime: string;
	/** Start frame */
	startFrame: number;
	/** End frame */
	endFrame: number;
	/** Damage dealt as percentage string (e.g., "35%") */
	damage: string;
	/** Start percent -> End percent range */
	damageRange: string;
	/** Number of moves in the conversion */
	moves: number;
	/** Opening type: "Neutral", "Counter Hit", "Trade" */
	openingType: string;
	/** Whether this conversion resulted in a kill */
	didKill: boolean;
}

/**
 * Complete game stats to save to the database.
 * Sent to Rust backend after parsing.
 */
export interface GameStatsForDB {
	// Recording linkage
	recordingId: string;
	slpPath: string;

	// Game metadata
	stage: number;
	gameDuration: number;
	totalFrames: number;
	isPal: boolean;
	playedOn: string | null;
	matchId: string | null;
	gameNumber: number | null;
	
	// Timestamp when game was played (ISO 8601)
	createdAt: string | null;

	// Outcome
	winnerIndex: number | null;
	loserIndex: number | null;
	gameEndMethod: string | null;

	// Player stats (array of 2+ players)
	players: PlayerStatsForDB[];
}
