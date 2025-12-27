/**
 * Slippi replay file parsing utilities.
 * Uses slippi-js to extract metadata from .slp files.
 *
 * @example
 * import { parseSlippiFile, parseSlippiFileWithCache } from '$lib/utils/slippi';
 *
 * // Parse a replay file (with caching for repeated access)
 * const metadata = await parseSlippiFileWithCache('/path/to/replay.slp');
 * if (metadata) {
 *   console.log(`Stage: ${metadata.stage}, Players: ${metadata.players.length}`);
 * }
 *
 * @module utils/slippi
 */

import { SlippiGame } from "@slippi/slippi-js";
import { readFile } from "@tauri-apps/plugin-fs";
import type { SlippiMetadata, SlippiPlayer } from "$lib/types/recording";
import { CharacterId, StageId } from "$lib/types/recording";

/**
 * Parse a .slp replay file and extract metadata.
 * Reads the file via Tauri's filesystem API and parses with slippi-js.
 *
 * @param slpPath - Full path to the .slp file
 * @returns Parsed SlippiMetadata or null if parsing fails
 */
export async function parseSlippiFile(slpPath: string): Promise<SlippiMetadata | null> {
	try {
		// Read the .slp file using Tauri's filesystem API
		const fileBuffer = await readFile(slpPath);
		
		// Convert Uint8Array to ArrayBuffer for slippi-js
		const arrayBuffer = fileBuffer.buffer.slice(
			fileBuffer.byteOffset,
			fileBuffer.byteOffset + fileBuffer.byteLength
		);

		// Parse with slippi-js
		const game = new SlippiGame(arrayBuffer);
		const settings = game.getSettings();
		const metadata = game.getMetadata();

		if (!settings) {
			console.warn("No settings found in .slp file:", slpPath);
			return null;
		}

		// Extract player information (using snake_case to match Rust backend)
		const players: SlippiPlayer[] = settings.players
			.filter((p) => p !== null) // Filter out null players
			.map((player) => ({
				character_id: player.characterId as CharacterId,
				character_color: player.characterColor || 0,
				player_tag: player.nametag || `Player ${player.playerIndex + 1}`,
				port: player.port,
			}));

		// Get all characters played
		const characters = players.map((p) => p.character_id);

		// Calculate game duration from metadata or frames
		const lastFrame = metadata?.lastFrame || 0;
		const gameDuration = lastFrame > 0 ? lastFrame : 0;

		// Determine winner (if game ended normally)
		const gameEnd = game.getGameEnd();
		const winnerPort = gameEnd?.gameEndMethod === 2 ? gameEnd.lrasInitiatorIndex : null;

		return {
			characters,
			stage: settings.stageId as StageId,
			players,
			game_duration: gameDuration,
			start_time: metadata?.startAt || new Date().toISOString(),
			is_pal: settings.isPAL || false,
			winner_port: winnerPort !== null ? winnerPort + 1 : null, // Convert to 1-based port
			played_on: metadata?.playedOn || null,
			total_frames: lastFrame,
		};
	} catch (error) {
		console.error("Error parsing .slp file:", slpPath, error);
		return null;
	}
}

/** Cache for parsed .slp files to avoid expensive re-parsing */
const slippiCache = new Map<string, SlippiMetadata | null>();

/**
 * Parse a .slp file with caching.
 * Returns cached result if available, otherwise parses and caches.
 *
 * @param slpPath - Full path to the .slp file
 * @returns Cached or freshly parsed SlippiMetadata, null if parsing failed
 */
export async function parseSlippiFileWithCache(
	slpPath: string
): Promise<SlippiMetadata | null> {
	if (slippiCache.has(slpPath)) {
		return slippiCache.get(slpPath) || null;
	}

	const metadata = await parseSlippiFile(slpPath);
	slippiCache.set(slpPath, metadata);
	return metadata;
}

/**
 * Clear all entries from the Slippi parsing cache.
 * Call this when you need to force re-parsing of all files.
 */
export function clearSlippiCache(): void {
	slippiCache.clear();
}

/**
 * Remove a specific file from the parsing cache.
 * Useful when a file has been modified or deleted.
 * @param slpPath - Path to remove from cache
 */
export function removeFromCache(slpPath: string): void {
	slippiCache.delete(slpPath);
}

