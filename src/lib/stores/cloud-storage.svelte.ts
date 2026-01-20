import { readFile, writeFile } from '@tauri-apps/plugin-fs';
import { invoke } from '@tauri-apps/api/core';
import { auth } from './auth.svelte';
import type { Upload, CloudClip, UploadQueueItem } from '$lib/types/cloud';

// Re-export types for convenience
export type { Upload, CloudClip, UploadQueueItem };

class CloudStorageStore {
	/** List of completed uploads for the current user */
	uploads = $state<Upload[]>([])
	/** List of public clips (shared via share codes) */
	userClips = $state<CloudClip[]>([]);
	/** Current upload queue with progress tracking */
	uploadQueue = $state<UploadQueueItem[]>([]);
	/** Whether data is being loaded */
	loading = $state(false);
	/** Last error message */
	error = $state<string | null>(null);
	private deviceId: string | null = null;

	// Unified view of all cloud items (uploads + clips)
	get allCloudItems() {
		const uploadItems = this.uploads.map(u => ({
			id: u.id,
			type: 'recording' as const,
			filename: u.filename,
			file_size: u.file_size,
			uploaded_at: u.uploaded_at,
			metadata: u.metadata,
		}));
		
		const clipItems = this.userClips.map(c => ({
			id: c.id,
			type: 'clip' as const,
			filename: c.filename,
			file_size: c.file_size,
			uploaded_at: c.uploaded_at,
			share_code: c.share_code,
			metadata: c.metadata,
		}));
		
		return [...uploadItems, ...clipItems].sort(
			(a, b) => new Date(b.uploaded_at).getTime() - new Date(a.uploaded_at).getTime()
		);
	}

	get totalCloudItems(): number {
		return this.uploads.length + (this.userClips?.length || 0);
	}

	// Check if a local clip has been uploaded to cloud
	isClipUploaded(filename: string): boolean {
		return this.userClips.some(c => c.filename === filename);
	}
	
	// Get share code for a filename if it exists
	getClipShareCode(filename: string): string | null {
		const clip = this.userClips.find(c => c.filename === filename);
		return clip ? clip.share_code : null;
	}

	constructor() {
		// Initialize
		this.init();
	}

	private async init() {
		// Get device ID for anonymous uploads
		try {
			this.deviceId = await invoke<string>('get_device_id');
		} catch (e) {
			console.error('Failed to get device ID:', e);
			this.deviceId = 'unknown-device';
		}
	}

	/**
	 * Upload a video file to cloud storage.
	 * @param videoPath - Local path to the video file
	 * @param metadata - Optional metadata to store with the file
	 */
	async uploadVideo(videoPath: string, metadata?: Record<string, unknown>) {
		// Existing upload logic...
		// (Assuming existing implementation details here)
	}

	async refreshUploads() {
		if (!auth.isAuthenticated) {
			this.uploads = [];
			return;
		}

		try {
			this.loading = true;
			const { data, error } = await auth.supabase
				.from('uploads')
				.select('*')
				.eq('user_id', auth.user?.id)
				.order('uploaded_at', { ascending: false });

			if (error) throw error;
			this.uploads = data || [];
		} catch (e) {
			console.error('Failed to fetch uploads:', e);
			this.error = 'Failed to fetch uploads';
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Refresh the list of clips uploaded by the current user (or device).
	 * If authenticated, fetches by user_id. If not, fetches by device_id.
	 */
	async refreshUserClips() {
		try {
			// If we don't have a device ID yet, wait a bit or try to fetch it
			if (!this.deviceId) {
				try {
					this.deviceId = await invoke<string>('get_device_id');
				} catch (e) {
					console.error('Failed to get device ID:', e);
				}
			}

			let query = auth.supabase
				.from('public_clips')
				.select('*')
				.order('uploaded_at', { ascending: false });

			// Filter by user or device
			if (auth.isAuthenticated && auth.user?.id) {
				query = query.eq('user_id', auth.user.id);
			} else if (this.deviceId) {
				query = query.eq('device_id', this.deviceId);
			} else {
				// Can't fetch clips without user or device ID
				this.userClips = [];
				return;
			}

			const { data, error } = await query;

			if (error) throw error;
			this.userClips = data || [];
		} catch (e) {
			console.error('Failed to fetch user clips:', e);
			// Don't set global error for this, just log it
		}
	}
	
	// ... rest of methods
}

export const cloudStorage = new CloudStorageStore();
