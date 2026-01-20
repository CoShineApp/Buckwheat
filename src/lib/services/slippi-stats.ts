/**
 * Slippi Stats Service
 *
 * Uses @slippi/slippi-js to parse .slp files and compute game statistics.
 * This runs in the browser and uses the Tauri fs plugin to read files.
 *
 * @module services/slippi-stats
 */

import { SlippiGame } from "@slippi/slippi-js";
import { readFile } from "@tauri-apps/plugin-fs";
import { invoke } from "@tauri-apps/api/core";
import type { GameStatsForDB, PlayerStatsForDB, ConversionForDisplay } from "$lib/types/slippi-stats";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type SlippiStats = any;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type SlippiOverall = any;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
type SlippiActionCounts = any;

/**
 * Safely get a number from a value that might be a number or an object with count/total.
 */
function getNumber(val: unknown, fallback = 0): number {
	if (typeof val === "number") return val;
	if (val && typeof val === "object") {
		if ("count" in val && typeof (val as { count: number }).count === "number") {
			return (val as { count: number }).count;
		}
		if ("total" in val && typeof (val as { total: number }).total === "number") {
			return (val as { total: number }).total;
		}
	}
	return fallback;
}

/**
 * Safely get a ratio from a RatioType object.
 */
function getRatio(val: unknown): number | null {
	if (val && typeof val === "object" && "ratio" in val) {
		const ratio = (val as { ratio: number | null }).ratio;
		return typeof ratio === "number" ? ratio : null;
	}
	if (typeof val === "number") return val;
	return null;
}

/**
 * Sum up object values (for throwCount which has up/down/forward/back).
 */
function sumObjectValues(val: unknown, fallback = 0): number {
	if (typeof val === "number") return val;
	if (val && typeof val === "object") {
		return Object.values(val as Record<string, number>)
			.filter((v) => typeof v === "number")
			.reduce((sum, v) => sum + v, 0);
	}
	return fallback;
}

/**
 * Parse a .slp file and compute all stats using slippi-js.
 * @param slpPath - Path to the .slp file
 * @returns Computed stats ready for database storage
 */
export async function parseSlippiStats(
	slpPath: string,
	recordingId: string
): Promise<GameStatsForDB | null> {
	try {
		console.log("[SlippiStats] Parsing stats for:", slpPath);

		// Read the file as binary using Tauri fs plugin
		const fileData = await readFile(slpPath);

		// Create SlippiGame from the binary data
		const game = new SlippiGame(fileData.buffer);

		// Get all the data we need
		const settings = game.getSettings();
		const metadata = game.getMetadata();
		const stats: SlippiStats = game.getStats();
		const gameEnd = game.getGameEnd();

		if (!settings || !stats) {
			console.warn("[SlippiStats] Could not get settings or stats from .slp file");
			return null;
		}

		// Build player stats
		const players: PlayerStatsForDB[] = [];

		for (let i = 0; i < settings.players.length; i++) {
			const player = settings.players[i];
			const playerIndex = player.playerIndex;

			// Find matching stats for this player
			const overall: SlippiOverall = stats.overall?.find(
				(o: SlippiOverall) => o.playerIndex === playerIndex
			);
			const actionCounts: SlippiActionCounts = stats.actionCounts?.find(
				(a: SlippiActionCounts) => a.playerIndex === playerIndex
			);

			// Count remaining stocks at game end
			const playerStocks =
				stats.stocks?.filter(
					(s: { playerIndex: number }) => s.playerIndex === playerIndex
				) ?? [];
			const lostStocks = playerStocks.filter(
				(s: { endFrame?: number | null }) => s.endFrame != null
			).length;
			const stocksRemaining = (player.startStocks ?? 4) - lostStocks;

			// Get final percent (from last stock)
			const lastStock = playerStocks[playerStocks.length - 1];
			const finalPercent = lastStock?.endPercent ?? lastStock?.currentPercent ?? null;

			// Get netplay info
			const metadataPlayer = metadata?.players?.[playerIndex];
			const connectCode =
				player.connectCode ?? metadataPlayer?.names?.code ?? null;
			const displayName =
				player.displayName ?? metadataPlayer?.names?.netplay ?? null;

			// Handle grabCount which could be number or object
			const grabCount = getNumber(actionCounts?.grabCount);

			// Handle throwCount which is an object with up/down/forward/back
			const throwCount = sumObjectValues(actionCounts?.throwCount);

			// Handle tech counts which might be objects
			const groundTechCount = sumObjectValues(actionCounts?.groundTechCount);
			const wallTechCount = sumObjectValues(actionCounts?.wallTechCount);

			const playerStats: PlayerStatsForDB = {
				playerIndex,
				connectCode,
				displayName,
				characterId: player.characterId ?? 0,
				characterColor: player.characterColor ?? 0,
				port: player.port ?? playerIndex + 1,

				// Overall performance
				totalDamage: overall?.totalDamage ?? 0,
				killCount: overall?.killCount ?? 0,
				conversionCount: overall?.conversionCount ?? 0,
				successfulConversions: getNumber(overall?.successfulConversions),
				openingsPerKill: getRatio(overall?.openingsPerKill),
				damagePerOpening: getRatio(overall?.damagePerOpening),
				neutralWinRatio: getRatio(overall?.neutralWinRatio),
				counterHitRatio: getRatio(overall?.counterHitRatio),
				beneficialTradeRatio: getRatio(overall?.beneficialTradeRatio),

				// Input stats
				inputsTotal: overall?.inputCounts?.total ?? 0,
				inputsPerMinute: getRatio(overall?.inputsPerMinute),
				avgKillPercent: overall?.avgKillPercent ?? null,

				// Action counts
				wavedashCount: actionCounts?.wavedashCount ?? 0,
				wavelandCount: actionCounts?.wavelandCount ?? 0,
				airDodgeCount: actionCounts?.airDodgeCount ?? 0,
				dashDanceCount: actionCounts?.dashDanceCount ?? 0,
				spotDodgeCount: actionCounts?.spotDodgeCount ?? 0,
				ledgegrabCount: actionCounts?.ledgegrabCount ?? 0,
				rollCount: actionCounts?.rollCount ?? 0,
				grabCount,
				throwCount,
				groundTechCount,
				wallTechCount,
				wallJumpTechCount: 0, // Not available in slippi-js

				// L-Cancel stats
				lCancelSuccessCount: actionCounts?.lCancelCount?.success ?? 0,
				lCancelFailCount: actionCounts?.lCancelCount?.fail ?? 0,

				// Final state
				stocksRemaining,
				finalPercent,
			};

			players.push(playerStats);
		}

		// Determine winner based on game end or stock count
		let winnerIndex: number | null = null;
		let loserIndex: number | null = null;
		let gameEndMethod: string | null = null;

		if (gameEnd) {
			gameEndMethod = getGameEndMethodString(gameEnd.gameEndMethod);

			// Check placements from gameEnd
			if (gameEnd.placements && gameEnd.placements.length >= 2) {
				const sorted = [...gameEnd.placements].sort(
					(a: { position?: number }, b: { position?: number }) =>
						(a.position ?? 99) - (b.position ?? 99)
				);
				if (sorted[0]?.position === 0) {
					winnerIndex = sorted[0].playerIndex;
					loserIndex = sorted[1]?.playerIndex ?? null;
				}
			}

			// Fallback: check LRAS initiator
			if (winnerIndex === null && gameEnd.lrasInitiatorIndex != null) {
				loserIndex = gameEnd.lrasInitiatorIndex;
				winnerIndex =
					players.find((p) => p.playerIndex !== loserIndex)?.playerIndex ?? null;
			}
		}

		// Fallback: determine winner by stocks remaining
		if (winnerIndex === null && players.length === 2) {
			if (players[0].stocksRemaining > players[1].stocksRemaining) {
				winnerIndex = players[0].playerIndex;
				loserIndex = players[1].playerIndex;
			} else if (players[1].stocksRemaining > players[0].stocksRemaining) {
				winnerIndex = players[1].playerIndex;
				loserIndex = players[0].playerIndex;
			}
		}

		// Build the complete game stats
		const gameStats: GameStatsForDB = {
			recordingId,
			slpPath,

			// Game metadata
			stage: settings.stageId ?? 0,
			gameDuration: stats.lastFrame ?? metadata?.lastFrame ?? 0,
			totalFrames: stats.lastFrame ?? metadata?.lastFrame ?? 0,
			isPal: settings.isPAL ?? false,
			playedOn: metadata?.playedOn ?? null,
			matchId: settings.matchInfo?.matchId ?? null,
			gameNumber: settings.matchInfo?.gameNumber ?? null,

			// Outcome
			winnerIndex,
			loserIndex,
			gameEndMethod,

			// Player stats
			players,
		};

		console.log(
			"[SlippiStats] Parsed stats for", players.length, "players, winner:", winnerIndex
		);
		return gameStats;
	} catch (error) {
		console.error("[SlippiStats] Failed to parse Slippi stats:", error);
		return null;
	}
}

/**
 * Parse a .slp file and save stats to the database.
 * This is the main entry point called when a recording ends.
 * @param slpPath - Path to the .slp file
 * @param recordingId - ID of the recording in the database
 */
export async function parseAndSaveSlippiStats(
	slpPath: string,
	recordingId: string
): Promise<boolean> {
	try {
		console.log("[SlippiStats] parseAndSaveSlippiStats called for", slpPath, "recordingId:", recordingId);
		
		const stats = await parseSlippiStats(slpPath, recordingId);

		if (!stats) {
			console.warn("[SlippiStats] No stats to save");
			return false;
		}

		console.log("[SlippiStats] Sending stats to Rust backend...");
		
		// Send to Rust backend for database storage
		await invoke("save_computed_stats", { stats });
		console.log("[SlippiStats] Saved computed stats to database for recording", recordingId);
		return true;
	} catch (error) {
		console.error("[SlippiStats] Failed to save Slippi stats:", error);
		return false;
	}
}

/**
 * Convert game end method enum to string.
 */
function getGameEndMethodString(method: number | null | undefined): string | null {
	if (method === null || method === undefined) return null;

	switch (method) {
		case 1:
			return "TIME";
		case 2:
			return "GAME";
		case 3:
			return "RESOLVED";
		case 7:
			return "NO_CONTEST";
		default:
			return `UNKNOWN_${method}`;
	}
}

/**
 * Get L-cancel percentage for a player.
 * @returns Percentage (0-100) or null if no L-cancels attempted
 */
export function getLCancelPercent(success: number, fail: number): number | null {
	const total = success + fail;
	if (total === 0) return null;
	return Math.round((success / total) * 100);
}

/**
 * Format a frame number as "M:SS" timestamp.
 * Melee runs at 60fps.
 */
function formatFrameTime(frame: number): string {
	// Frame -123 is game start, so adjust
	const adjustedFrame = frame + 123;
	const totalSeconds = Math.max(0, Math.floor(adjustedFrame / 60));
	const minutes = Math.floor(totalSeconds / 60);
	const seconds = totalSeconds % 60;
	return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

/**
 * Get opening type string from slippi-js conversion.
 */
function getOpeningType(openingType: string | number | null | undefined): string {
	if (openingType === "neutral-win" || openingType === 1) return "Neutral";
	if (openingType === "counter-attack" || openingType === 2) return "Counter";
	if (openingType === "trade" || openingType === 3) return "Trade";
	return "Unknown";
}

/**
 * Parse conversions from a .slp file on-the-fly for display.
 * @param slpPath - Path to the .slp file
 * @returns Array of conversions grouped by player index
 */
export async function getConversionsFromSlp(
	slpPath: string
): Promise<Map<number, ConversionForDisplay[]>> {
	try {
		console.log("[SlippiStats] Getting conversions for:", slpPath);

		// Read the file as binary using Tauri fs plugin
		const fileData = await readFile(slpPath);

		// Create SlippiGame from the binary data
		const game = new SlippiGame(fileData.buffer);

		// Get conversions from stats
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const stats: any = game.getStats();

		if (!stats || !stats.conversions) {
			console.warn("[SlippiStats] No conversions in stats");
			return new Map();
		}

		// Group conversions by player who performed them
		const conversionsByPlayer = new Map<number, ConversionForDisplay[]>();

		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		for (const conv of stats.conversions as any[]) {
			const playerIndex = conv.playerIndex;
			
			if (!conversionsByPlayer.has(playerIndex)) {
				conversionsByPlayer.set(playerIndex, []);
			}

			const startFrame = conv.startFrame ?? 0;
			const endFrame = conv.endFrame ?? conv.lastHitFrame ?? startFrame;
			const startPercent = conv.startPercent ?? 0;
			const endPercent = conv.endPercent ?? 0;
			const damage = endPercent - startPercent;
			const moveCount = conv.moves?.length ?? 0;

			const conversion: ConversionForDisplay = {
				playerIndex,
				startTime: formatFrameTime(startFrame),
				endTime: formatFrameTime(endFrame),
				startFrame,
				endFrame,
				damage: `${Math.round(damage)}%`,
				damageRange: `${Math.round(startPercent)}% -> ${Math.round(endPercent)}%`,
				moves: moveCount,
				openingType: getOpeningType(conv.openingType),
				didKill: conv.didKill ?? false,
			};

			conversionsByPlayer.get(playerIndex)!.push(conversion);
		}

		console.log(
			"[SlippiStats] Parsed", stats.conversions.length, "conversions for",
			conversionsByPlayer.size, "players"
		);

		return conversionsByPlayer;
	} catch (error) {
		console.error("[SlippiStats] Failed to get conversions:", error);
		return new Map();
	}
}
