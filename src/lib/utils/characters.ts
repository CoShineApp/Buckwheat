import { CharacterId, StageId } from "$lib/types/recording";

// Character names for display
export const CHARACTER_NAMES: Record<CharacterId, string> = {
	[CharacterId.CAPTAIN_FALCON]: "Captain Falcon",
	[CharacterId.DONKEY_KONG]: "Donkey Kong",
	[CharacterId.FOX]: "Fox",
	[CharacterId.GAME_AND_WATCH]: "Mr. Game & Watch",
	[CharacterId.KIRBY]: "Kirby",
	[CharacterId.BOWSER]: "Bowser",
	[CharacterId.LINK]: "Link",
	[CharacterId.LUIGI]: "Luigi",
	[CharacterId.MARIO]: "Mario",
	[CharacterId.MARTH]: "Marth",
	[CharacterId.MEWTWO]: "Mewtwo",
	[CharacterId.NESS]: "Ness",
	[CharacterId.PEACH]: "Peach",
	[CharacterId.PIKACHU]: "Pikachu",
	[CharacterId.ICE_CLIMBERS]: "Ice Climbers",
	[CharacterId.JIGGLYPUFF]: "Jigglypuff",
	[CharacterId.SAMUS]: "Samus",
	[CharacterId.YOSHI]: "Yoshi",
	[CharacterId.ZELDA]: "Zelda",
	[CharacterId.SHEIK]: "Sheik",
	[CharacterId.FALCO]: "Falco",
	[CharacterId.YOUNG_LINK]: "Young Link",
	[CharacterId.DR_MARIO]: "Dr. Mario",
	[CharacterId.ROY]: "Roy",
	[CharacterId.PICHU]: "Pichu",
	[CharacterId.GANONDORF]: "Ganondorf",
};

// Character images are now served from /static/characters/
// No need for external URLs - all images are local!

// Stage names for display
export const STAGE_NAMES: Record<number, string> = {
	[StageId.FOUNTAIN_OF_DREAMS]: "Fountain of Dreams",
	[StageId.POKEMON_STADIUM]: "Pokémon Stadium",
	[StageId.YOSHIS_STORY]: "Yoshi's Story",
	[StageId.DREAM_LAND]: "Dream Land",
	[StageId.BATTLEFIELD]: "Battlefield",
	[StageId.FINAL_DESTINATION]: "Final Destination",
};

// Get character name by ID with fallback
export function getCharacterName(characterId: CharacterId | number): string {
	return CHARACTER_NAMES[characterId as CharacterId] || `Unknown Character (${characterId})`;
}

// Get character slug for file paths
export function getCharacterSlug(characterId: CharacterId | number): string {
	const slugs: Record<CharacterId, string> = {
		[CharacterId.CAPTAIN_FALCON]: "captain-falcon",
		[CharacterId.DONKEY_KONG]: "donkey-kong",
		[CharacterId.FOX]: "fox",
		[CharacterId.GAME_AND_WATCH]: "game-and-watch",
		[CharacterId.KIRBY]: "kirby",
		[CharacterId.BOWSER]: "bowser",
		[CharacterId.LINK]: "link",
		[CharacterId.LUIGI]: "luigi",
		[CharacterId.MARIO]: "mario",
		[CharacterId.MARTH]: "marth",
		[CharacterId.MEWTWO]: "mewtwo",
		[CharacterId.NESS]: "ness",
		[CharacterId.PEACH]: "peach",
		[CharacterId.PIKACHU]: "pikachu",
		[CharacterId.ICE_CLIMBERS]: "ice-climbers",
		[CharacterId.JIGGLYPUFF]: "jigglypuff",
		[CharacterId.SAMUS]: "samus",
		[CharacterId.YOSHI]: "yoshi",
		[CharacterId.ZELDA]: "zelda",
		[CharacterId.SHEIK]: "sheik",
		[CharacterId.FALCO]: "falco",
		[CharacterId.YOUNG_LINK]: "young-link",
		[CharacterId.DR_MARIO]: "dr-mario",
		[CharacterId.ROY]: "roy",
		[CharacterId.PICHU]: "pichu",
		[CharacterId.GANONDORF]: "ganondorf",
	};
	return slugs[characterId as CharacterId] || "unknown";
}

// Get character image URL by ID (uses local static assets)
export function getCharacterImage(characterId: CharacterId | number): string {
	const slug = getCharacterSlug(characterId);
	// Use local static asset
	return `/characters/${slug}.png`;
}

// Get stage slug for file paths
export function getStageSlug(stageId: StageId | number): string {
	const slugs: Record<number, string> = {
		[StageId.FOUNTAIN_OF_DREAMS]: "fountain-of-dreams",
		[StageId.POKEMON_STADIUM]: "pokemon-stadium",
		[StageId.YOSHIS_STORY]: "yoshis-story",
		[StageId.DREAM_LAND]: "dream-land",
		[StageId.BATTLEFIELD]: "battlefield",
		[StageId.FINAL_DESTINATION]: "final-destination",
	};
	return slugs[stageId] || "unknown";
}

// Get stage image path
export function getStageImage(stageId: StageId | number): string {
	const slug = getStageSlug(stageId);
	return `/stages/${slug}.jpg`;
}

// Get stage name by ID with fallback
export function getStageName(stageId: StageId | number): string {
	return STAGE_NAMES[stageId] || `Unknown Stage (${stageId})`;
}

// Format game duration from frames to readable time
export function formatGameDuration(frames: number): string {
	const seconds = Math.floor(frames / 60); // Melee runs at 60 FPS
	const minutes = Math.floor(seconds / 60);
	const remainingSeconds = seconds % 60;
	return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
}

// Format file size to human readable
export function formatFileSize(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
	if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

// Format relative time (e.g., "2 hours ago")
export function formatRelativeTime(timestamp: string): string {
	const now = new Date();
	const then = new Date(timestamp);
	const diffMs = now.getTime() - then.getTime();
	const diffSeconds = Math.floor(diffMs / 1000);
	const diffMinutes = Math.floor(diffSeconds / 60);
	const diffHours = Math.floor(diffMinutes / 60);
	const diffDays = Math.floor(diffHours / 24);

	if (diffSeconds < 60) return "just now";
	if (diffMinutes < 60) return `${diffMinutes}m ago`;
	if (diffHours < 24) return `${diffHours}h ago`;
	if (diffDays < 7) return `${diffDays}d ago`;
	
	// Return formatted date for older items
	return then.toLocaleDateString();
}
