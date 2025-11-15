// Character IDs from Melee (internal character IDs used by slippi-js)
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

// Stage IDs from Melee
export enum StageId {
	FOUNTAIN_OF_DREAMS = 2,
	POKEMON_STADIUM = 3,
	YOSHIS_STORY = 8,
	DREAM_LAND = 28,
	BATTLEFIELD = 31,
	FINAL_DESTINATION = 32,
}

// Player information from .slp file
export interface SlippiPlayer {
	character_id: CharacterId;
	character_color: number;
	player_tag: string;
	port: number;
}

// Metadata extracted from .slp file
export interface SlippiMetadata {
	characters: CharacterId[];
	stage: StageId | number;
	players: SlippiPlayer[];
	game_duration: number; // in frames (lastFrame)
	start_time: string;
	is_pal: boolean;
	winner_port: number | null;
	played_on: string | null; // "dolphin", "console", "nintendont"
	total_frames: number; // Total frames in recording
}

// Backend recording session (from Rust)
export interface RecordingSession {
	id: string;
	start_time: string;
	end_time: string | null;
	slp_path: string;
	video_path: string | null;
	thumbnail_path: string | null;
	duration: number | null; // in seconds
	file_size: number | null; // in bytes
	slippi_metadata: SlippiMetadata | null;
}

// Frontend recording with parsed metadata
export interface RecordingWithMetadata extends RecordingSession {
	is_loading?: boolean;
	is_selected?: boolean;
}

// Game event types
export enum GameEventType {
	DEATH = 'death',
	// Future: 'combo', 'neutral_exchange', 'sd', etc.
}

// Base game event interface
export interface GameEvent {
	type: GameEventType;
	frame: number; // Frame number when event occurred
	timestamp: number; // Time in seconds (frame / 60)
}

// Death event - when a player loses a stock
export interface DeathEvent extends GameEvent {
	type: GameEventType.DEATH;
	port: number; // Which player died (1-4)
	player_tag: string; // Player's tag/name
}

