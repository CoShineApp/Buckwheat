/**
 * Melee character and stage data utilities.
 * Provides mappings for character/stage IDs to names, slugs, and image paths.
 *
 * @example
 * import { getCharacterName, getCharacterImage, getStageName } from '$lib/utils/characters';
 *
 * const name = getCharacterName(CharacterId.FOX); // "Fox"
 * const image = getCharacterImage(CharacterId.FOX); // "/characters/fox.png"
 *
 * @module utils/characters
 */

import { CharacterId, StageId } from "$lib/types/recording";

/** Human-readable names for each character ID */
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

/**
 * Human-readable names for each stage ID.
 * Only legal tournament stages are included.
 */
export const STAGE_NAMES: Record<number, string> = {
	[StageId.FOUNTAIN_OF_DREAMS]: "Fountain of Dreams",
	[StageId.POKEMON_STADIUM]: "Pokémon Stadium",
	[StageId.YOSHIS_STORY]: "Yoshi's Story",
	[StageId.DREAM_LAND]: "Dream Land",
	[StageId.BATTLEFIELD]: "Battlefield",
	[StageId.FINAL_DESTINATION]: "Final Destination",
};

/**
 * Get the display name for a character ID.
 * @param characterId - Character ID from Slippi data
 * @returns Human-readable character name, or "Unknown Character (id)" if not found
 */
export function getCharacterName(characterId: CharacterId | number): string {
	return CHARACTER_NAMES[characterId as CharacterId] || `Unknown Character (${characterId})`;
}

/**
 * Get the URL-safe slug for a character (used in file paths).
 * @param characterId - Character ID from Slippi data
 * @returns Kebab-case slug (e.g., "captain-falcon")
 */
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

/**
 * Get the path to a character's stock icon image.
 * Images are served from /static/characters/.
 * @param characterId - Character ID from Slippi data
 * @returns Path to character image (e.g., "/characters/fox.png")
 */
export function getCharacterImage(characterId: CharacterId | number): string {
	const slug = getCharacterSlug(characterId);
	// Use local static asset
	return `/characters/${slug}.png`;
}

/**
 * Get the URL-safe slug for a stage (used in file paths).
 * @param stageId - Stage ID from Slippi data
 * @returns Kebab-case slug (e.g., "fountain-of-dreams")
 */
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

/**
 * Get the path to a stage's preview image.
 * Images are served from /static/stages/.
 * @param stageId - Stage ID from Slippi data
 * @returns Path to stage image (e.g., "/stages/battlefield.jpg")
 */
export function getStageImage(stageId: StageId | number): string {
	const slug = getStageSlug(stageId);
	// Some stages use .png, others use .jpg
	const pngStages = ["final-destination", "yoshis-story"];
	const ext = pngStages.includes(slug) ? "png" : "jpg";
	return `/stages/${slug}.${ext}`;
}

/**
 * Get the display name for a stage ID.
 * @param stageId - Stage ID from Slippi data
 * @returns Human-readable stage name, or "Unknown Stage (id)" if not found
 */
export function getStageName(stageId: StageId | number): string {
	return STAGE_NAMES[stageId] || `Unknown Stage (${stageId})`;
}

