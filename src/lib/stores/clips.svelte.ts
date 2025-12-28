/**
 * Clips store for managing clip markers and saved clips.
 * Clips are short video segments created by marking moments during recording.
 *
 * @example
 * // Mark a clip during recording
 * clipsStore.markClip(currentTimestamp, videoPath);
 *
 * // Refresh clips list from backend
 * await clipsStore.refresh();
 *
 * // Access clips
 * clipsStore.clips.forEach(clip => console.log(clip.filename));
 *
 * @module stores/clips
 */

import { invoke } from '@tauri-apps/api/core';
import type { RecordingSession } from '$lib/types/recording';
import type { ClipMarker, ClipSession } from '$lib/types/clip';

// Re-export types for convenience
export type { ClipMarker, ClipSession } from '$lib/types/clip';

/**
 * Map a backend RecordingSession to a ClipSession for display.
 * @param session - Raw recording session from the backend
 * @returns Formatted clip session for UI display
 */
function mapRecordingSessionToClip(session: RecordingSession): ClipSession {
	// Extract filename from video_path or use id
	const filename = session.video_path 
		? session.video_path.split(/[/\\]/).pop() || session.id
		: session.id;
	
	// slp_path is empty string when no Slippi file exists, convert to null
	const slp_path = session.slp_path && session.slp_path.trim() !== '' 
		? session.slp_path 
		: null;
	
	return {
		id: session.id,
		filename,
		video_path: session.video_path || '',
		thumbnail_path: session.thumbnail_path || null,
		start_time: session.start_time,
		duration: session.duration,
		file_size: session.file_size,
		slp_path,
		slippi_metadata: session.slippi_metadata,
	};
}

/**
 * Manages clip creation markers and saved clips list.
 */
class ClipsStore {
	/** List of saved clips */
	clips = $state<ClipSession[]>([]);
	/** Markers placed during recording for clip creation */
	clipMarkers = $state<ClipMarker[]>([]);
	/** Whether clips are currently being loaded */
	loading = $state(false);

	/**
	 * Mark a timestamp during recording for clip creation.
	 * Clips are extracted after recording ends.
	 * @param timestamp - Seconds from start of recording
	 * @param recordingFile - Path to the recording file
	 */
	markClip(timestamp: number, recordingFile: string) {
		this.clipMarkers.push({ timestamp, recordingFile });
		console.log(`📌 Clip marked at ${timestamp}s for ${recordingFile}`);
	}

	/**
	 * Get all markers for a specific recording file.
	 * @param recordingFile - Path to the recording file
	 * @returns Array of clip markers for the file
	 */
	getMarkers(recordingFile: string): ClipMarker[] {
		return this.clipMarkers.filter(m => m.recordingFile === recordingFile);
	}

	/**
	 * Clear all markers for a specific recording file.
	 * Called after clips have been processed.
	 * @param recordingFile - Path to the recording file
	 */
	clearMarkers(recordingFile: string) {
		this.clipMarkers = this.clipMarkers.filter(m => m.recordingFile !== recordingFile);
		console.log(`🧹 Cleared clip markers for ${recordingFile}`);
	}

	/**
	 * Refresh the clips list from the backend.
	 * Fetches all saved clips and updates the store.
	 */
	async refresh() {
		try {
			this.loading = true;
			const sessions = await invoke<RecordingSession[]>('get_clips');
			// Map RecordingSession to ClipSession, filtering out any without video_path
			this.clips = sessions
				.filter(session => session.video_path) // Only include clips with video_path
				.map(mapRecordingSessionToClip);
			console.log(`✅ Loaded ${this.clips.length} clip(s)`);
		} catch (error) {
			console.error('Failed to fetch clips:', error);
			this.clips = [];
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Delete a clip and its video file.
	 * @param clipId - ID of the clip to delete
	 * @param videoPath - Path to the video file
	 * @throws Error if deletion fails
	 */
	async deleteClip(clipId: string, videoPath: string) {
		try {
			await invoke('delete_recording', {
				videoPath,
				slpPath: '' // Empty string since clips don't have .slp files
			});
			await this.refresh();
		} catch (error) {
			console.error('Failed to delete clip:', error);
			throw error;
		}
	}
}

/** Singleton clips store instance */
export const clipsStore = new ClipsStore();

