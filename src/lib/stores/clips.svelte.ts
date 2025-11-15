import { invoke } from '@tauri-apps/api/core';
import type { RecordingSession } from '$lib/types/recording';

export interface ClipMarker {
	timestamp: number;
	recordingFile: string;
}

export interface ClipSession {
	id: string;
	filename: string;
	video_path: string;
	thumbnail_path: string | null;
	start_time: string;
	duration: number | null;
	file_size: number | null;
	slp_path: string | null;
	slippi_metadata: any | null;
}

// Map RecordingSession from backend to ClipSession
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

class ClipsStore {
	clips = $state<ClipSession[]>([]);
	clipMarkers = $state<ClipMarker[]>([]);
	loading = $state(false);

	markClip(timestamp: number, recordingFile: string) {
		this.clipMarkers.push({ timestamp, recordingFile });
		console.log(`📌 Clip marked at ${timestamp}s for ${recordingFile}`);
	}

	getMarkers(recordingFile: string): ClipMarker[] {
		return this.clipMarkers.filter(m => m.recordingFile === recordingFile);
	}

	clearMarkers(recordingFile: string) {
		this.clipMarkers = this.clipMarkers.filter(m => m.recordingFile !== recordingFile);
		console.log(`🧹 Cleared clip markers for ${recordingFile}`);
	}

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

export const clipsStore = new ClipsStore();

