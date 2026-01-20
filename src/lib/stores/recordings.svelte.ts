/**
 * Recordings store for managing the list of recorded games.
 * Handles fetching, selection, deletion, and manual recording controls.
 * Also manages event listeners for auto-recording.
 *
 * @example
 * // Bootstrap the store (sets up event listeners)
 * onMount(() => {
 *   const cleanup = recordingsStore.bootstrap();
 *   return cleanup;
 * });
 *
 * // Start/stop manual recording
 * await recordingsStore.startManualRecording();
 * await recordingsStore.stopManualRecording();
 *
 * // Access recordings
 * recordingsStore.recordings.forEach(rec => console.log(rec.id));
 *
 * @module stores/recordings
 */

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { RecordingSession, RecordingWithMetadata, GameEvent, PaginatedRecordings } from "$lib/types/recording";
import { handleTauriError, showSuccess } from "$lib/utils/errors";
import { recording } from "$lib/stores/recording.svelte";
import { settings } from "$lib/stores/settings.svelte";
import { clipsStore, type ClipSession } from "$lib/stores/clips.svelte";

/**
 * Manages the recordings list, selection state, and recording controls.
 * Uses reference counting for bootstrap/teardown lifecycle.
 */
class RecordingsStore {
	/** List of all recordings with metadata */
	recordings = $state<RecordingWithMetadata[]>([]);
	/** IDs of currently selected recordings */
	selectedIds = $state<Set<string>>(new Set());
	/** Whether recordings are being loaded */
	isLoading = $state(false);
	/** Last error message */
	error = $state<string | null>(null);
	/** Whether manual recording is starting */
	isManualStarting = $state(false);
	/** Whether manual recording is stopping */
	isManualStopping = $state(false);

	// Pagination state
	/** Current page number (1-indexed) */
	currentPage = $state(1);
	/** Total number of pages */
	totalPages = $state(1);
	/** Number of recordings per page */
	perPage = $state(20);
	/** Total number of recordings */
	totalRecordings = $state(0);

	/** Whether event listeners are active */
	private listenersActive = false;
	/** Reference count for bootstrap calls */
	private bootstrapRefCount = 0;
	/** Promises for event listener cleanup */
	private eventListenerPromises: Promise<() => void>[] = [];
	/** Additional cleanup functions */
	private extraCleanupFns: Array<() => void> = [];
	/** Current .slp file path for the active recording session (for stats parsing) */
	private currentSlpPath: string | null = null;

	constructor() {
		// Start with empty recordings - will load real data on first refresh
		this.recordings = [];
	}

	/**
	 * Fetch recordings from the backend with pagination.
	 * Updates the recordings list with fresh data for the specified page.
	 * @param page - Page number to fetch (1-indexed), defaults to current page
	 */
	async refresh(page?: number) {
		this.isLoading = true;
		this.error = null;

		const targetPage = page ?? this.currentPage;

		try {
			const response = await invoke<PaginatedRecordings>("get_recordings", {
				page: targetPage,
				perPage: this.perPage,
			});
			
			this.recordings = response.recordings.map((session) => ({
				...session,
				is_selected: this.selectedIds.has(session.id),
			}));
			this.currentPage = response.page;
			this.totalPages = response.total_pages;
			this.totalRecordings = response.total;
			
			console.log(`✅ Loaded ${this.recordings.length} recordings (page ${response.page}/${response.total_pages}, total: ${response.total})`);
		} catch (e) {
			this.error = e instanceof Error ? e.message : "Failed to fetch recordings";
			console.error("Failed to fetch recordings:", e);
			this.recordings = [];
		} finally {
			this.isLoading = false;
		}
	}

	/**
	 * Go to the next page of recordings.
	 */
	async nextPage() {
		if (this.currentPage < this.totalPages) {
			await this.refresh(this.currentPage + 1);
		}
	}

	/**
	 * Go to the previous page of recordings.
	 */
	async prevPage() {
		if (this.currentPage > 1) {
			await this.refresh(this.currentPage - 1);
		}
	}

	/**
	 * Go to a specific page of recordings.
	 * @param page - Page number (1-indexed)
	 */
	async goToPage(page: number) {
		const targetPage = Math.max(1, Math.min(page, this.totalPages));
		if (targetPage !== this.currentPage) {
			await this.refresh(targetPage);
		}
	}

	/**
	 * Change the number of recordings per page.
	 * Reloads from page 1 with the new page size.
	 * @param perPage - Number of recordings per page
	 */
	async setPerPage(perPage: number) {
		this.perPage = Math.max(1, Math.min(perPage, 100));
		await this.refresh(1);
	}

	/** Whether there are more pages after the current one */
	get hasNextPage() {
		return this.currentPage < this.totalPages;
	}

	/** Whether there are pages before the current one */
	get hasPrevPage() {
		return this.currentPage > 1;
	}

	/**
	 * Toggle selection state for a recording.
	 * @param id - Recording ID to toggle
	 */
	toggleSelection(id: string) {
		if (this.selectedIds.has(id)) {
			this.selectedIds.delete(id);
		} else {
			this.selectedIds.add(id);
		}
		this.selectedIds = new Set(this.selectedIds); // Trigger reactivity
	}

	/** Select all recordings */
	selectAll() {
		this.selectedIds = new Set(this.recordings.map((r) => r.id));
	}

	/** Clear all selections */
	clearSelection() {
		this.selectedIds.clear();
		this.selectedIds = new Set(this.selectedIds); // Trigger reactivity
	}

	/** Number of selected recordings */
	get selectedCount() {
		return this.selectedIds.size;
	}

	/** Whether all recordings are selected */
	get allSelected() {
		return this.recordings.length > 0 && this.selectedIds.size === this.recordings.length;
	}

	/** Whether some (but not all) recordings are selected */
	get someSelected() {
		return this.selectedIds.size > 0 && this.selectedIds.size < this.recordings.length;
	}

	/**
	 * Delete all selected recordings.
	 * Removes video and .slp files from disk.
	 */
	async deleteSelected() {
		const idsToDelete = Array.from(this.selectedIds);
		console.log("Deleting recordings:", idsToDelete);
		
		try {
			for (const id of idsToDelete) {
				const recording = this.recordings.find((r) => r.id === id);
				if (recording) {
					await invoke("delete_recording", { 
						videoPath: recording.video_path,
						slpPath: recording.slp_path 
					});
				}
			}
			
			// Refresh the list after deletion
			await this.refresh();
			this.clearSelection();
		} catch (e) {
			this.error = e instanceof Error ? e.message : "Failed to delete recordings";
			console.error("Failed to delete recordings:", e);
		}
	}

	/** Total storage used by all recordings in bytes */
	get totalStorage() {
		return this.recordings.reduce((total, rec) => total + (rec.file_size || 0), 0);
	}

	/** Character ID of the most played character, -1 if none */
	get mostPlayedCharacter() {
		const characterCounts: Record<number, number> = {};

		this.recordings.forEach((rec) => {
			if (rec.slippi_metadata) {
				rec.slippi_metadata.characters.forEach((charId) => {
					characterCounts[charId] = (characterCounts[charId] || 0) + 1;
				});
			}
		});

		let maxCount = 0;
		let mostPlayed = -1;

		for (const [charIdStr, count] of Object.entries(characterCounts)) {
			if (count > maxCount) {
				maxCount = count;
				mostPlayed = parseInt(charIdStr, 10);
			}
		}

		return mostPlayed;
	}

	/**
	 * Bootstrap the store with event listeners.
	 * Uses reference counting - call the returned cleanup function when done.
	 * @returns Cleanup function to call on unmount
	 */
	bootstrap() {
		this.bootstrapRefCount += 1;

		if (!this.listenersActive) {
			this.listenersActive = true;
			void this.refresh();
			this.setupRecordingListeners();
		}

		return () => {
			this.bootstrapRefCount = Math.max(0, this.bootstrapRefCount - 1);
			if (this.bootstrapRefCount === 0) {
				void this.teardownRecordingListeners();
			}
		};
	}

	/**
	 * Start a manual recording (not auto-triggered by game detection).
	 * Invokes the Tauri backend to begin screen capture.
	 */
	async startManualRecording() {
		if (this.isManualStarting || recording.isRecording) {
			return;
		}

		this.isManualStarting = true;

		try {
			const outputPath = await invoke<string>("start_generic_recording");
			console.log("🎥 Manual recording started:", outputPath);
			recording.setReplayPath(outputPath);
			recording.start();
			showSuccess("Recording started");
		} catch (error) {
			handleTauriError(error, "Failed to start recording");
		} finally {
			this.isManualStarting = false;
		}
	}

	/**
	 * Stop the current manual recording.
	 * Processes any clip markers and refreshes the recordings list.
	 */
	async stopManualRecording() {
		if (this.isManualStopping || !recording.isRecording) {
			return;
		}

		this.isManualStopping = true;

		try {
			const outputPath = await invoke<string>("stop_recording");
			console.log("⏹️  Recording stopped:", outputPath);
			recording.stop();
			showSuccess("Recording stopped");
			await this.refresh();
		} catch (error) {
			handleTauriError(error, "Failed to stop recording");
		} finally {
			this.isManualStopping = false;
		}
	}

	/** Set up Tauri event listeners for recording state changes */
	private setupRecordingListeners() {
		invoke<string | null>("get_last_replay_path")
			.then((path) => {
				if (path) {
					recording.setReplayPath(path);
				}
			})
			.catch((error) => {
				console.error("Failed to get last replay path:", error);
			});

		this.eventListenerPromises.push(
			listen<string>("recording-started", (event) => {
				recording.start();
				// For auto recordings, update currentReplayPath to video output path
				// (markers need to match the video path, not .slp path)
				// The event.payload is the video output path (.mp4)
				if (event.payload) {
					recording.setReplayPath(event.payload);
				}
				showSuccess(recording.currentReplayPath ? "Auto-recording started" : "Recording started");
			})
		);

		this.eventListenerPromises.push(
			listen<string>("last-replay-updated", (event) => {
				// Always store the slp path for stats parsing later
				this.currentSlpPath = event.payload;
				console.log("[SlippiStats] Stored slp path:", this.currentSlpPath);
				
				// Only set .slp path if we're not already recording with a video path
				// (for auto recordings, recording-started will set the video path)
				if (!recording.isRecording || !recording.currentReplayPath?.endsWith('.mp4')) {
					recording.setReplayPath(event.payload);
				}
			})
		);

		this.eventListenerPromises.push(
			listen<string>("recording-stopped", async (event) => {
				console.log("[SlippiStats] recording-stopped event received, payload:", event.payload);
				recording.stop();

				// Use the video path from the event payload (guaranteed to be correct)
				const videoPath = event.payload || recording.currentReplayPath;
				console.log("[SlippiStats] Using video path:", videoPath);
				if (videoPath) {
					try {
						const clips = await invoke<string[]>("process_clip_markers", {
							recordingFile: videoPath
						});
						if (clips.length > 0) {
							showSuccess(`Recording stopped - ${clips.length} clip(s) created!`);
						} else {
							showSuccess("Recording stopped automatically");
						}
					} catch (error) {
						console.error("Failed to process clip markers:", error);
						showSuccess("Recording stopped automatically");
					}
				} else {
					showSuccess("Recording stopped automatically");
				}

				// Capture the slp path we stored earlier (from last-replay-updated event)
				const slpPath = this.currentSlpPath;
				console.log("[SlippiStats] Using stored slp path:", slpPath);
				
				recording.setReplayPath(null);
				this.currentSlpPath = null; // Clear for next recording
				
				// Trigger a full cache sync to ensure the new recording is indexed
				console.log("[SlippiStats] Triggering cache sync for new recording...");
				try {
					await invoke("refresh_recordings_cache");
					console.log("[SlippiStats] Cache sync complete");
				} catch (e) {
					console.error("[SlippiStats] Cache sync failed:", e);
				}
				
				console.log("[SlippiStats] Refreshing recordings list...");
				await this.refresh();
				console.log("[SlippiStats] Refresh complete, recordings count:", this.recordings.length);

				// Parse and save Slippi stats for the recording
				await this.parseStatsForRecording(videoPath, slpPath);
			})
		);

		const hotkeyHandler = async (event: KeyboardEvent) => {
			const configuredHotkey = settings.createClipHotkey;
			if (!configuredHotkey) return;

			const pressedKey = this.formatHotkey(event);
			if (pressedKey === configuredHotkey) {
				event.preventDefault();
				await this.handleCreateClip();
			}
		};

		document.addEventListener("keydown", hotkeyHandler);
		this.extraCleanupFns.push(() => document.removeEventListener("keydown", hotkeyHandler));
	}

	/** Clean up all event listeners */
	private async teardownRecordingListeners() {
		const unsubs = await Promise.allSettled(this.eventListenerPromises);
		for (const result of unsubs) {
			if (result.status === "fulfilled") {
				try {
					result.value();
				} catch (error) {
					console.error("Failed to unsubscribe listener:", error);
				}
			}
		}
		this.eventListenerPromises = [];

		while (this.extraCleanupFns.length > 0) {
			const cleanup = this.extraCleanupFns.pop();
			if (cleanup) {
				try {
					cleanup();
				} catch (error) {
					console.error("Failed to run cleanup:", error);
				}
			}
		}

		this.listenersActive = false;
	}

	/** Format a keyboard event into a hotkey string (e.g., "Ctrl+Shift+F9") */
	private formatHotkey(event: KeyboardEvent): string {
		const parts: string[] = [];
		if (event.ctrlKey || event.metaKey) parts.push(event.metaKey ? "Cmd" : "Ctrl");
		if (event.altKey) parts.push("Alt");
		if (event.shiftKey) parts.push("Shift");

		const key = event.key;
		if (!["Control", "Alt", "Shift", "Meta"].includes(key)) {
			const formattedKey = key.length === 1 ? key.toUpperCase() : key;
			parts.push(formattedKey);
		}

		return parts.join("+");
	}

	/** Handle the create clip hotkey press */
	private async handleCreateClip() {
		if (!recording.isRecording || !recording.startTimestamp || !recording.currentReplayPath) {
			handleTauriError(new Error("Can only create clips during active recording"), "Not recording");
			return;
		}

		try {
			const timestamp = (Date.now() - recording.startTimestamp) / 1000;
			await invoke("mark_clip_timestamp", {
				recordingFile: recording.currentReplayPath,
				timestamp
			});
			showSuccess(`Clip marked at ${Math.floor(timestamp)}s! Will be created after recording ends.`);
		} catch (error) {
			handleTauriError(error, "Failed to mark clip");
		}
	}

	/**
	 * Get a clip by ID, refreshing the clips list first.
	 * @param id - Clip ID to find
	 * @returns The clip session or undefined if not found
	 */
	async getClipRecording(id: string): Promise<ClipSession | undefined> {
		// Always refresh to ensure we have latest clips
		await clipsStore.refresh();
		const clip = clipsStore.clips.find((clip) => clip.id === id);
		if (!clip) {
			console.warn('⚠️ Clip not found:', id, 'Available clips:', clipsStore.clips.map(c => c.id));
		} else {
			console.log('✅ Found clip:', clip.id, 'video_path:', clip.video_path);
		}
		return clip;
	}

	/**
	 * Get a recording by ID from the current list.
	 * @param id - Recording ID to find
	 * @returns The recording or undefined if not found
	 */
	getSlippiRecording(id: string): RecordingWithMetadata | undefined {
		return this.recordings.find((r) => r.id === id);
	}

	/**
	 * Parse and load game events from a .slp file.
	 * @param slpPath - Path to the .slp file
	 * @returns Array of game events (deaths, etc.)
	 */
	async loadSlippiEvents(slpPath: string): Promise<GameEvent[]> {
		try {
			return await invoke<GameEvent[]>("parse_slp_events", { slpPath });
		} catch (error) {
			handleTauriError(error, "Failed to parse replay events");
			return [];
		}
	}

	/**
	 * Check if a recording is clip-only (no associated .slp file).
	 * @param recording - Recording or clip to check
	 * @returns True if the recording has no Slippi data
	 */
	isClipOnly(recording: ClipSession | RecordingWithMetadata | undefined): boolean {
		return !recording?.slp_path;
	}

	/**
	 * Parse Slippi stats for a recording and save to database.
	 * Called after a recording ends to compute L-cancel %, openings/kill, etc.
	 * @param videoPath - Path to the video file (used to find the recording)
	 * @param slpPathHint - Optional direct path to the .slp file (from recording session)
	 */
	private async parseStatsForRecording(videoPath: string | null, slpPathHint?: string | null) {
		console.log("[SlippiStats] parseStatsForRecording called with videoPath:", videoPath, "slpPathHint:", slpPathHint);
		
		if (!videoPath) {
			console.log("[SlippiStats] No video path provided, skipping stats parsing");
			return;
		}

		try {
			// Normalize path separators for comparison
			const normalizedVideoPath = videoPath.replace(/\\/g, "/");
			
			console.log("[SlippiStats] Looking for recording in list. Total recordings:", this.recordings.length);
			console.log("[SlippiStats] Normalized video path:", normalizedVideoPath);
			
			// Find the recording in our list (we just synced and refreshed)
			const rec = this.recordings.find((r) => {
				const normalizedRecPath = r.video_path?.replace(/\\/g, "/");
				return normalizedRecPath === normalizedVideoPath;
			});
			
			// Use the recording from the list, or create a minimal one with the hint
			let recordingId: string;
			let slpPath: string | null;
			
			if (rec) {
				console.log("[SlippiStats] Found recording:", rec.id, "slp_path:", rec.slp_path);
				recordingId = rec.id;
				slpPath = rec.slp_path || slpPathHint || null;
			} else {
				console.warn("[SlippiStats] Recording not found in list, using slpPathHint. Video paths in list:", 
					this.recordings.slice(0, 5).map(r => r.video_path));
				
				// If recording not found but we have the slp path hint, we can still parse stats
				// We'll need to find or create the recording ID after cache syncs
				if (!slpPathHint) {
					console.log("[SlippiStats] No slp path hint available, cannot parse stats");
					return;
				}
				
				// Try to find by slp_path instead
				const recBySlp = this.recordings.find((r) => {
					const normalizedSlpPath = r.slp_path?.replace(/\\/g, "/");
					const normalizedHint = slpPathHint?.replace(/\\/g, "/");
					return normalizedSlpPath === normalizedHint;
				});
				
				if (recBySlp) {
					console.log("[SlippiStats] Found recording by slp_path:", recBySlp.id);
					recordingId = recBySlp.id;
					slpPath = recBySlp.slp_path;
				} else {
					console.log("[SlippiStats] Recording not found by slp_path either, will retry after delay");
					// Wait a bit and retry once more
					await new Promise(resolve => setTimeout(resolve, 2000));
					await this.refresh();
					
					const retryRec = this.recordings.find((r) => {
						const normalizedRecPath = r.video_path?.replace(/\\/g, "/");
						return normalizedRecPath === normalizedVideoPath;
					});
					
					if (!retryRec) {
						console.error("[SlippiStats] Recording still not found after retry, giving up");
						return;
					}
					
					console.log("[SlippiStats] Found recording on retry:", retryRec.id);
					recordingId = retryRec.id;
					slpPath = retryRec.slp_path || slpPathHint;
				}
			}

			if (!slpPath) {
				console.log("[SlippiStats] No .slp path for recording, skipping stats parsing");
				return;
			}

			console.log("[SlippiStats] Parsing Slippi stats for recording", recordingId, "from", slpPath);

			// Import the slippi-stats service dynamically to avoid loading it on every page
			const { parseAndSaveSlippiStats } = await import("$lib/services/slippi-stats");

			const success = await parseAndSaveSlippiStats(slpPath, recordingId);
			if (success) {
				console.log("[SlippiStats] Stats saved successfully for recording", recordingId);
			} else {
				console.warn("[SlippiStats] Failed to parse stats for recording", recordingId);
			}
		} catch (error) {
			console.error("[SlippiStats] Error parsing Slippi stats:", error);
			// Don't show error to user - stats parsing is non-critical
		}
	}
}

/** Singleton recordings store instance */
export const recordingsStore = new RecordingsStore();

