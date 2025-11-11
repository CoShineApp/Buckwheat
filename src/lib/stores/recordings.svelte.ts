import { invoke } from "@tauri-apps/api/core";
import type { RecordingSession, RecordingWithMetadata } from "$lib/types/recording";
import { CharacterId, StageId } from "$lib/types/recording";

class RecordingsStore {
	recordings = $state<RecordingWithMetadata[]>([]);
	selectedIds = $state<Set<string>>(new Set());
	isLoading = $state(false);
	error = $state<string | null>(null);

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
}

// Export singleton instance
export const recordingsStore = new RecordingsStore();

