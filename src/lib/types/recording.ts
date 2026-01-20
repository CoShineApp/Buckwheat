/**
 * Recording and Slippi type definitions.
 * Contains types for Melee game data, recordings, and events.
 * @module types/recording
 */

/**
 * Character IDs from Melee (internal IDs used by slippi-js).
 * Values match the game's internal character ordering.
 */
export enum CharacterId {
	CAPTAIN_FALCON = 0,
	DONKEY_KONG = 1,
	FOX = 2,
	GAME_AND_WATCH = 3,
	KIRBY = 4,
	BOWSER = 5,
	LINK = 6,
	LUIGI = 7,
	MARIO = 8,
	MARTH = 9,
	MEWTWO = 10,
	NESS = 11,
	PEACH = 12,
	PIKACHU = 13,
	ICE_CLIMBERS = 14,
	JIGGLYPUFF = 15,
	SAMUS = 16,
	YOSHI = 17,
	ZELDA = 18,
	SHEIK = 19,
	FALCO = 20,
	YOUNG_LINK = 21,
	DR_MARIO = 22,
	ROY = 23,
	PICHU = 24,
	GANONDORF = 25,
}

/**
 * Stage IDs from Melee.
 * Only legal tournament stages are included.
 */
export enum StageId {
	FOUNTAIN_OF_DREAMS = 2,
	POKEMON_STADIUM = 3,
	YOSHIS_STORY = 8,
	DREAM_LAND = 28,
	BATTLEFIELD = 31,
	FINAL_DESTINATION = 32,
}

/**
 * Player information extracted from a .slp file.
 * Uses snake_case to match Rust backend data.
 */
export interface SlippiPlayer {
	/** Character used by this player */
	character_id: CharacterId;
	/** Costume/color variant index (0-based) */
	character_color: number;
	/** Player's name tag */
	player_tag: string;
	/** Controller port (1-4) */
	port: number;
}

/**
 * Metadata extracted from a .slp replay file.
 * Contains game information like stage, players, and duration.
 */
export interface SlippiMetadata {
	/** All character IDs in the game */
	characters: CharacterId[];
	/** Stage ID where the game was played */
	stage: StageId | number;
	/** Player information array */
	players: SlippiPlayer[];
	/** Game duration in frames (Melee runs at 60 FPS) */
	game_duration: number;
	/** ISO timestamp when the game started */
	start_time: string;
	/** Whether the game was played on PAL version */
	is_pal: boolean;
	/** Port number of the winner (1-4), null if no winner determined */
	winner_port: number | null;
	/** Platform: "dolphin", "console", or "nintendont" */
	played_on: string | null;
	/** Total frames in the recording */
	total_frames: number;
}

/**
 * Recording session data from the Rust backend.
 * Represents a single recorded game with video and optional Slippi data.
 */
export interface RecordingSession {
	/** Unique identifier for the recording */
	id: string;
	/** ISO timestamp when recording started */
	start_time: string;
	/** ISO timestamp when recording ended, null if still recording */
	end_time: string | null;
	/** Path to the .slp replay file */
	slp_path: string;
	/** Path to the video file, null if video not available */
	video_path: string | null;
	/** Path to the thumbnail image */
	thumbnail_path: string | null;
	/** Recording duration in seconds */
	duration: number | null;
	/** Video file size in bytes */
	file_size: number | null;
	/** Parsed Slippi metadata, null if .slp file not available */
	slippi_metadata: SlippiMetadata | null;
}

/**
 * Recording session with UI state for the frontend.
 * Extends RecordingSession with loading and selection state.
 */
export interface RecordingWithMetadata extends RecordingSession {
	/** Whether metadata is being loaded */
	is_loading?: boolean;
	/** Whether this recording is selected in the UI */
	is_selected?: boolean;
}

/**
 * Paginated response from get_recordings command.
 * Backend now returns recordings in pages for better performance.
 */
export interface PaginatedRecordings {
	/** Array of recordings for the current page */
	recordings: RecordingSession[];
	/** Total number of recordings across all pages */
	total: number;
	/** Current page number (1-indexed) */
	page: number;
	/** Number of recordings per page */
	per_page: number;
	/** Total number of pages */
	total_pages: number;
}

/**
 * Types of events that can occur during a game.
 * Used for timeline markers in the replay viewer.
 */
export enum GameEventType {
	/** A player lost a stock */
	DEATH = 'death',
	// Future: 'combo', 'neutral_exchange', 'sd', etc.
}

/**
 * Base interface for all game events.
 * Contains timing information for the event.
 */
export interface GameEvent {
	/** Type of event */
	type: GameEventType;
	/** Frame number when event occurred */
	frame: number;
	/** Time in seconds (frame / 60) */
	timestamp: number;
}

/**
 * Death event - occurs when a player loses a stock.
 * Contains information about who died.
 */
export interface DeathEvent extends GameEvent {
	/** Event type discriminator */
	type: GameEventType.DEATH;
	/** Port of the player who died (1-4) */
	port: number;
	/** Player's name tag */
	player_tag: string;
}

