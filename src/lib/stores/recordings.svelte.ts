import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { RecordingSession, RecordingWithMetadata, GameEvent } from "$lib/types/recording";
import { handleTauriError, showSuccess } from "$lib/utils/errors";
import { recording } from "$lib/stores/recording.svelte";
import { settings } from "$lib/stores/settings.svelte";
import { clipsStore, type ClipSession } from "$lib/stores/clips.svelte";

class RecordingsStore {
	recordings = $state<RecordingWithMetadata[]>([]);
	selectedIds = $state<Set<string>>(new Set());
	isLoading = $state(false);
	error = $state<string | null>(null);
	isManualStarting = $state(false);
	isManualStopping = $state(false);

	private listenersActive = false;
	private bootstrapRefCount = 0;
	private eventListenerPromises: Promise<() => void>[] = [];
	private extraCleanupFns: Array<() => void> = [];

	constructor() {
		// Start with empty recordings - will load real data on first refresh
		this.recordings = [];
	}

	// Fetch recordings from backend
	async refresh() {
		this.isLoading = true;
		this.error = null;

		try {
			const sessions = await invoke<RecordingSession[]>("get_recordings");
			this.recordings = sessions.map((session) => ({
				...session,
				is_selected: this.selectedIds.has(session.id),
			}));
			console.log(`✅ Loaded ${this.recordings.length} recordings`);
		} catch (e) {
			this.error = e instanceof Error ? e.message : "Failed to fetch recordings";
			console.error("Failed to fetch recordings:", e);
			this.recordings = [];
		} finally {
			this.isLoading = false;
		}
	}

	// Toggle selection for a recording
	toggleSelection(id: string) {
		if (this.selectedIds.has(id)) {
			this.selectedIds.delete(id);
		} else {
			this.selectedIds.add(id);
		}
		this.selectedIds = new Set(this.selectedIds); // Trigger reactivity
	}

	// Select all recordings
	selectAll() {
		this.selectedIds = new Set(this.recordings.map((r) => r.id));
	}

	// Clear all selections
	clearSelection() {
		this.selectedIds.clear();
		this.selectedIds = new Set(this.selectedIds); // Trigger reactivity
	}

	// Get count of selected recordings
	get selectedCount() {
		return this.selectedIds.size;
	}

	// Check if all recordings are selected
	get allSelected() {
		return this.recordings.length > 0 && this.selectedIds.size === this.recordings.length;
	}

	// Check if some (but not all) are selected
	get someSelected() {
		return this.selectedIds.size > 0 && this.selectedIds.size < this.recordings.length;
	}

	// Delete selected recordings
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

	// Get total storage used by all recordings
	get totalStorage() {
		return this.recordings.reduce((total, rec) => total + (rec.file_size || 0), 0);
	}

	// Get most played character
	get mostPlayedCharacter() {
		const characterCounts = new Map<number, number>();
		
		this.recordings.forEach((rec) => {
			if (rec.slippi_metadata) {
				rec.slippi_metadata.characters.forEach((charId) => {
					characterCounts.set(charId, (characterCounts.get(charId) || 0) + 1);
				});
			}
		});

		let maxCount = 0;
		let mostPlayed = -1;
		
		characterCounts.forEach((count, charId) => {
			if (count > maxCount) {
				maxCount = count;
				mostPlayed = charId;
			}
		});

		return mostPlayed;
	}

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
			listen("recording-started", () => {
				recording.start();
				showSuccess(recording.currentReplayPath ? "Auto-recording started" : "Recording started");
			})
		);

		this.eventListenerPromises.push(
			listen<string>("last-replay-updated", (event) => {
				recording.setReplayPath(event.payload);
			})
		);

		this.eventListenerPromises.push(
			listen("recording-stopped", async () => {
				recording.stop();

				if (recording.currentReplayPath) {
					try {
						const clips = await invoke<string[]>("process_clip_markers", {
							recordingFile: recording.currentReplayPath
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

				recording.setReplayPath(null);
				await this.refresh();
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

	// Get a clip recording by ID (ensures clips are loaded)
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

	// Get a Slippi recording by ID
	getSlippiRecording(id: string): RecordingWithMetadata | undefined {
		return this.recordings.find((r) => r.id === id);
	}

	// Load Slippi events from a .slp file
	async loadSlippiEvents(slpPath: string): Promise<GameEvent[]> {
		try {
			return await invoke<GameEvent[]>("parse_slp_events", { slpPath });
		} catch (error) {
			handleTauriError(error, "Failed to parse replay events");
			return [];
		}
	}

	// Check if a recording/clip is clip-only (no Slippi metadata)
	isClipOnly(recording: ClipSession | RecordingWithMetadata | undefined): boolean {
		return !recording?.slp_path;
	}
}

// Export singleton instance
export const recordingsStore = new RecordingsStore();

