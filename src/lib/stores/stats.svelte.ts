/**
 * @module stores/stats
 * @description Manages player game statistics and aggregates.
 * Provides methods for calculating, querying, and syncing stats to cloud.
 */

import { invoke } from '@tauri-apps/api/core';
import type { PlayerGameStats, AggregateStats, SyncResult } from '$lib/types/stats';
import { auth } from './auth.svelte';
import { handleTauriError, showSuccess, showInfo } from '$lib/utils/errors';

/**
 * A Svelte store that manages player statistics.
 * Provides methods to calculate, query, and sync stats.
 */
class StatsStore {
	/** Reactive state holding per-game stats records */
	gameStats = $state<PlayerGameStats[]>([]);
	/** Reactive state holding aggregate stats for a player */
	aggregateStats = $state<AggregateStats | null>(null);
	/** Reactive state indicating if stats are being loaded */
	loading = $state(false);
	/** Reactive state indicating if stats are being synced */
	syncing = $state(false);
	/** Reactive state holding any error messages */
	error = $state<string | null>(null);

	/**
	 * Calculate stats for a completed game.
	 * This is typically called automatically after a recording ends.
	 * @param slpPath Path to the .slp replay file
	 * @param recordingId ID of the recording session
	 * @returns Promise resolving to calculated stats for all players
	 */
	async calculateStatsForGame(
		slpPath: string,
		recordingId: string
	): Promise<PlayerGameStats[]> {
		this.loading = true;
		this.error = null;

		try {
			const stats = await invoke<PlayerGameStats[]>('calculate_game_stats', {
				slpPath,
				recordingId,
			});

			showSuccess(`Stats calculated for ${stats.length} player(s)`);
			return stats;
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to calculate stats';
			handleTauriError(err, 'Failed to calculate game stats');
			throw err;
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Get stats for a specific recording.
	 * @param recordingId The ID of the recording
	 * @returns Promise resolving to stats for all players in that recording
	 */
	async getRecordingStats(recordingId: string): Promise<PlayerGameStats[]> {
		this.loading = true;
		this.error = null;

		try {
			const stats = await invoke<PlayerGameStats[]>('get_recording_stats', {
				recordingId,
			});

			return stats;
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to load stats';
			handleTauriError(err, 'Failed to load recording stats');
			return [];
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Load player stats with optional filters.
	 * @param filters Optional filters for player tag, character, and result limit
	 */
	async loadPlayerStats(filters?: {
		playerTag?: string;
		characterId?: number;
		limit?: number;
	}): Promise<void> {
		this.loading = true;
		this.error = null;

		try {
			this.gameStats = await invoke<PlayerGameStats[]>('get_player_stats', {
				playerTag: filters?.playerTag || null,
				characterId: filters?.characterId || null,
				limit: filters?.limit || 50,
			});
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to load stats';
			handleTauriError(err, 'Failed to load player stats');
			this.gameStats = [];
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Load aggregate stats for a player.
	 * Calculates averages and totals across all games.
	 * @param playerTag The player's tag/name
	 */
	async loadAggregateStats(playerTag: string): Promise<void> {
		this.loading = true;
		this.error = null;

		try {
			this.aggregateStats = await invoke<AggregateStats>('get_aggregate_stats', {
				playerTag,
			});
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to load aggregate stats';
			handleTauriError(err, 'Failed to load aggregate stats');
			this.aggregateStats = null;
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Sync local stats to Supabase cloud.
	 * Only works for authenticated users.
	 */
	async syncToCloud(): Promise<void> {
		if (!auth.isAuthenticated) {
			showInfo('Sign in to sync stats to cloud');
			return;
		}

		this.syncing = true;
		this.error = null;

		try {
			const result = await invoke<SyncResult>('sync_stats_to_cloud');

			if (result.synced_count > 0) {
				showSuccess(`Synced ${result.synced_count} stats record(s) to cloud`);
			} else {
				showInfo('No new stats to sync');
			}

			if (result.failed_count > 0) {
				showInfo(`${result.failed_count} record(s) failed to sync`);
			}
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to sync stats';
			handleTauriError(err, 'Failed to sync stats to cloud');
		} finally {
			this.syncing = false;
		}
	}

	/**
	 * Clear all loaded stats from the store.
	 */
	clear(): void {
		this.gameStats = [];
		this.aggregateStats = null;
		this.error = null;
	}
}

/**
 * Singleton instance of the StatsStore.
 */
export const statsStore = new StatsStore();

