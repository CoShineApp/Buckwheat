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
				.from('clips')
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

	/**
	 * Refresh all cloud data (uploads, clips, and storage usage).
	 */
	async refreshAll() {
		this.loading = true;
		try {
			await Promise.all([
				this.refreshUploads(),
				this.refreshUserClips(),
				auth.loadProfile(), // Refresh storage usage
			]);
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Create a public clip by uploading to cloud storage.
	 * Returns the clip data including share code.
	 * @param videoPath - Local path to the video file
	 * @param deviceId - Device ID for anonymous uploads
	 * @param metadata - Optional metadata (slippi_metadata, duration, etc.)
	 */
	async createPublicClip(
		videoPath: string,
		deviceId: string,
		metadata?: { slippi_metadata?: unknown; duration?: number | null }
	): Promise<{ clip: CloudClip; alreadyExists: boolean }> {
		// Extract filename from path
		const filename = videoPath.split(/[/\\]/).pop() || 'clip.mp4';
		
		// Check if this clip was already uploaded (by filename) in local cache
		const existing = this.userClips.find(c => c.filename === filename);
		if (existing) {
			return { clip: existing, alreadyExists: true };
		}

		// Read the video file
		const videoData = await readFile(videoPath);
		
		// Generate upload URL from edge function
		// This also creates the database record and checks for duplicates
		const { data: uploadData, error: uploadError } = await auth.supabase.functions.invoke(
			'generate-clip-upload-url',
			{
				body: {
					fileName: filename,
					fileSize: videoData.byteLength,
					deviceId,
					metadata: metadata || null,
				},
			}
		);

		if (uploadError) {
			throw new Error(uploadError.message || 'Failed to get upload URL');
		}

		// Check if clip already exists (server-side check)
		if (uploadData?.alreadyExists && uploadData?.clip) {
			const existingClip: CloudClip = uploadData.clip;
			// Add to local cache if not present
			if (!this.userClips.find(c => c.id === existingClip.id)) {
				this.userClips = [existingClip, ...this.userClips];
			}
			return { clip: existingClip, alreadyExists: true };
		}

		if (!uploadData?.uploadUrl) {
			throw new Error('Failed to get upload URL');
		}

		// Upload to B2/R2 using the signed URL
		const uploadResponse = await fetch(uploadData.uploadUrl, {
			method: 'PUT',
			body: videoData,
			headers: {
				'Content-Type': 'video/mp4',
			},
		});

		if (!uploadResponse.ok) {
			throw new Error(`Upload failed: ${uploadResponse.statusText}`);
		}

		// The clip record was already created by generate-clip-upload-url
		const clipData = uploadData.clip;
		
		// Build the CloudClip object from the response
		const newClip: CloudClip = {
			id: clipData.id,
			user_id: clipData.user_id || null,
			device_id: clipData.device_id || deviceId,
			filename: clipData.filename || filename,
			b2_file_id: clipData.b2_file_id || null,
			b2_file_name: clipData.b2_file_name || null,
			file_size: clipData.file_size || videoData.byteLength,
			duration_seconds: metadata?.duration || null,
			share_code: clipData.share_code || uploadData.shareCode,
			uploaded_at: clipData.uploaded_at || new Date().toISOString(),
			metadata: clipData.metadata || metadata || null,
		};
		
		this.userClips = [newClip, ...this.userClips];
		
		// Refresh auth profile to update storage usage
		await auth.loadProfile();

		return { clip: newClip, alreadyExists: false };
	}

	/**
	 * Delete an upload from cloud storage.
	 * @param uploadId - ID of the upload to delete
	 */
	async deleteUpload(uploadId: string): Promise<void> {
		const { error } = await auth.supabase.functions.invoke('delete-upload', {
			body: { uploadId },
		});

		if (error) {
			throw new Error(error.message || 'Failed to delete upload');
		}

		// Remove from local list
		this.uploads = this.uploads.filter(u => u.id !== uploadId);
		
		// Refresh auth profile to update storage usage
		await auth.loadProfile();
	}

	/**
	 * Delete a clip from cloud storage.
	 * @param clipId - ID of the clip to delete
	 */
	async deleteClip(clipId: string): Promise<void> {
		const { error } = await auth.supabase.functions.invoke('delete-clip', {
			body: { 
				clipId,
				deviceId: this.deviceId,
			},
		});

		if (error) {
			throw new Error(error.message || 'Failed to delete clip');
		}

		// Remove from local list
		this.userClips = this.userClips.filter(c => c.id !== clipId);
		
		// Refresh auth profile to update storage usage
		await auth.loadProfile();
	}
}

export const cloudStorage = new CloudStorageStore();
