import { readFile, writeFile } from '@tauri-apps/plugin-fs';
import { invoke } from '@tauri-apps/api/core';
import { auth } from './auth.svelte';

export interface Upload {
	id: string;
	user_id: string;
	filename: string;
	b2_file_id: string | null;
	b2_file_name: string | null;
	file_size: number;
	duration_seconds: number | null;
	uploaded_at: string;
	metadata: any | null;
	status: 'UPLOADING' | 'UPLOADED' | 'FAILED';
}

export interface Clip {
	id: string;
	user_id: string | null;
	device_id: string | null;
	filename: string;
	b2_file_id: string | null;
	b2_file_name: string | null;
	file_size: number;
	duration_seconds: number | null;
	share_code: string;
	uploaded_at: string;
	metadata: any | null;
}

export interface UploadQueueItem {
	id: string;
	videoPath: string;
	status: 'pending' | 'uploading' | 'completed' | 'error' | 'cancelled';
	progress: number;
	error?: string;
	uploadId?: string;
	abortController?: AbortController;
	xhr?: XMLHttpRequest;
}

class CloudStorageStore {
	uploads = $state<Upload[]>([]);
	clips = $state<Clip[]>([]);
	uploadQueue = $state<UploadQueueItem[]>([]);
	loading = $state(false);
	error = $state<string | null>(null);

	async refreshUploads() {
		if (!auth.isAuthenticated || !auth.user) {
			this.uploads = [];
			return;
		}

		try {
			this.loading = true;
			this.error = null;

			// Only fetch UPLOADED videos
			const { data, error } = await auth.supabase
				.from('uploads')
				.select('*')
				.eq('user_id', auth.user.id)
				.eq('status', 'UPLOADED')
				.order('uploaded_at', { ascending: false});

			if (error) throw error;

			this.uploads = data || [];
		} catch (err) {
			console.error('Error fetching uploads:', err);
			this.error = err instanceof Error ? err.message : 'Failed to fetch uploads';
		} finally {
			this.loading = false;
		}
	}

	async uploadVideo(videoPath: string, metadata?: any) {
		if (!auth.isAuthenticated || !auth.user) {
			throw new Error('Must be authenticated to upload');
		}

		const token = auth.getToken();
		if (!token) {
			throw new Error('No auth token available');
		}

		// Add to queue
		const queueItem: UploadQueueItem = {
			id: crypto.randomUUID(),
			videoPath,
			status: 'pending',
			progress: 0,
		};
		this.uploadQueue = [...this.uploadQueue, queueItem]; // Trigger reactivity

		let compressedPath: string | null = null;

		try {
			queueItem.status = 'uploading';
			this.uploadQueue = [...this.uploadQueue]; // Trigger reactivity

			// Step 1: Compress video (0-30%)
			queueItem.progress = 2;
			this.uploadQueue = [...this.uploadQueue];

			try {
				compressedPath = await invoke<string>('compress_video_for_upload', {
					inputPath: videoPath
				});
				queueItem.progress = 30;
				this.uploadQueue = [...this.uploadQueue];
			} catch (err) {
				console.warn('Video compression failed, uploading original:', err);
				compressedPath = videoPath; // Fallback to original
				queueItem.progress = 30;
				this.uploadQueue = [...this.uploadQueue];
			}

			// Step 2: Read compressed file
			const fileBuffer = await readFile(compressedPath);
			const originalFileName = videoPath.split(/[\\/]/).pop()!;
			const fileName = originalFileName;
			const fileSize = fileBuffer.length;

			queueItem.progress = 35;
			this.uploadQueue = [...this.uploadQueue];

			// Step 3: Get signed upload URL (35-40%)
			let signedData;
			let signedError;
			try {
				const response = await auth.supabase.functions.invoke('generate-upload-url', {
					body: { fileName, fileSize, metadata }
				});
				signedData = response.data;
				signedError = response.error;
			} catch (err) {
				throw new Error('Failed to get upload URL: ' + (err instanceof Error ? err.message : 'timeout or network error'));
			}

			if (signedError) {
				throw new Error('Edge function error: ' + signedError.message);
			}

			if (!signedData?.uploadUrl || !signedData?.upload?.id) {
				throw new Error('No upload URL received from server');
			}

			queueItem.uploadId = signedData.upload.id;
			queueItem.progress = 40;
			this.uploadQueue = [...this.uploadQueue];

			// Step 4: Upload to B2 (40-95%)
			await new Promise<void>((resolve, reject) => {
				const xhr = new XMLHttpRequest();
				queueItem.xhr = xhr; // Store for cancellation
				
				// Set timeout for upload (20 minutes for large files)
				xhr.timeout = 1200000;

				// Track upload progress
				xhr.upload.addEventListener('progress', (e) => {
					if (e.lengthComputable) {
						const percentComplete = (e.loaded / e.total) * 100;
						// Map 40% to 95% for upload progress
						queueItem.progress = 40 + (percentComplete * 0.55);
						this.uploadQueue = [...this.uploadQueue]; // Trigger reactivity
					}
				});

				xhr.addEventListener('load', () => {
					if (xhr.status >= 200 && xhr.status < 300) {
						queueItem.progress = 95;
						this.uploadQueue = [...this.uploadQueue];
						resolve();
					} else {
						reject(new Error(`Upload failed: ${xhr.status} ${xhr.statusText}`));
					}
				});

				xhr.addEventListener('error', () => {
					reject(new Error('Network error during upload'));
				});

				xhr.addEventListener('timeout', () => {
					reject(new Error('Upload timed out after 20 minutes'));
				});

				xhr.addEventListener('abort', () => {
					reject(new Error('Upload cancelled by user'));
				});

				xhr.open('PUT', signedData.uploadUrl);
				xhr.setRequestHeader('Content-Type', 'video/mp4');
				xhr.send(fileBuffer);
			});

			// Step 5: Mark upload as complete (95-100%)
			queueItem.progress = 97;
			this.uploadQueue = [...this.uploadQueue];

			await auth.supabase.functions.invoke('complete-upload', {
				body: { uploadId: queueItem.uploadId, status: 'UPLOADED' }
			});

			// Update storage usage in profile
			await auth.loadProfile();
			
			queueItem.status = 'completed';
			queueItem.progress = 100;
			this.uploadQueue = [...this.uploadQueue];

			// Refresh uploads list
			await this.refreshUploads();

			return signedData.upload;
		} catch (err) {
			// Mark upload as FAILED in database if we have an uploadId
			if (queueItem.uploadId && err instanceof Error && !err.message.includes('cancelled')) {
				try {
					await auth.supabase.functions.invoke('complete-upload', {
						body: { uploadId: queueItem.uploadId, status: 'FAILED' }
					});
				} catch (completeErr) {
					console.error('Failed to mark upload as FAILED:', completeErr);
				}
			}

			queueItem.status = queueItem.xhr?.readyState === XMLHttpRequest.DONE && queueItem.status === 'uploading' ? 'cancelled' : 'error';
			queueItem.error = err instanceof Error ? err.message : 'Upload failed';
			this.uploadQueue = [...this.uploadQueue];
			console.error('Upload error:', err);
			throw err;
		} finally {
			// Cleanup compressed file if it was created
			if (compressedPath && compressedPath !== videoPath) {
				try {
					await invoke('delete_temp_file', { path: compressedPath });
				} catch (err) {
					console.warn('Failed to delete temp compressed file:', err);
				}
			}
		}
	}

	cancelUpload(id: string) {
		const queueItem = this.uploadQueue.find(item => item.id === id);
		if (queueItem && queueItem.xhr) {
			queueItem.xhr.abort();
			queueItem.status = 'cancelled';
			queueItem.error = 'Upload cancelled by user';
			this.uploadQueue = [...this.uploadQueue]; // Trigger reactivity
		}
	}

	async downloadVideo(uploadId: string, destPath: string) {
		if (!auth.isAuthenticated || !auth.user) {
			throw new Error('Must be authenticated to download');
		}

		const token = auth.getToken();
		if (!token) {
			throw new Error('No auth token available');
		}

		try {
			// Call Edge Function to get signed download URL
			const { data, error } = await auth.supabase.functions.invoke('generate-download-url', {
				body: { uploadId }
			});

			if (error) throw error;

			// Download from B2 using signed URL
			const response = await fetch(data.downloadUrl);
			if (!response.ok) {
				throw new Error(`Download failed: ${response.statusText}`);
			}

			const blob = await response.blob();
			const arrayBuffer = await blob.arrayBuffer();
			const uint8Array = new Uint8Array(arrayBuffer);

			// Write to destination using Tauri fs
			await writeFile(destPath, uint8Array);

		} catch (err) {
			console.error('Error downloading video:', err);
			throw err;
		}
	}

	async deleteUpload(uploadId: string) {
		if (!auth.isAuthenticated || !auth.user) {
			throw new Error('Must be authenticated to delete');
		}

		try {
			// Get upload info first
			const { data: upload, error: fetchError } = await auth.supabase
				.from('uploads')
				.select('*')
				.eq('id', uploadId)
				.eq('user_id', auth.user.id)
				.single();

			if (fetchError || !upload) {
				throw new Error('Upload not found or unauthorized');
			}

			// Delete from database (RLS will enforce user ownership)
			const { error: deleteError } = await auth.supabase
				.from('uploads')
				.delete()
				.eq('id', uploadId);

			if (deleteError) throw deleteError;

			// TODO: Delete from B2 via Edge Function (future enhancement)
			// For now, files remain in B2 but are inaccessible via database

			// Update storage usage
			const newUsage = (auth.profile?.storage_used || 0) - upload.file_size;
			await auth.supabase
				.from('profiles')
				.update({ storage_used: Math.max(0, newUsage) })
				.eq('id', auth.user.id);

			// Refresh uploads list and profile
			await this.refreshUploads();
			await auth.loadProfile();
		} catch (err) {
			console.error('Error deleting upload:', err);
			throw err;
		}
	}

	async createPublicClip(videoPath: string, deviceId: string, metadata?: any) {
		try {
			// Read file to get size
			const fileBuffer = await readFile(videoPath);
			const fileName = videoPath.split(/[\\/]/).pop()!;
			const fileSize = fileBuffer.length;

			// Step 1: Get signed upload URL and create clip record
			const { data: signedData, error: urlError } = await auth.supabase.functions.invoke('generate-clip-upload-url', {
				body: {
					fileName,
					fileSize,
					deviceId,
					metadata: metadata || null
				}
			});

			if (urlError) throw urlError;

			if (!signedData?.uploadUrl || !signedData?.clip) {
				throw new Error('No upload URL received from server');
			}

			// Step 2: Upload directly to B2 using signed URL
			await new Promise<void>((resolve, reject) => {
				const xhr = new XMLHttpRequest();
				
				// Set timeout for upload (20 minutes for large files)
				xhr.timeout = 1200000;

				xhr.addEventListener('load', () => {
					if (xhr.status >= 200 && xhr.status < 300) {
						resolve();
					} else {
						reject(new Error(`Upload failed: ${xhr.status} ${xhr.statusText}`));
					}
				});

				xhr.addEventListener('error', () => {
					reject(new Error('Network error during upload'));
				});

				xhr.addEventListener('timeout', () => {
					reject(new Error('Upload timed out after 20 minutes'));
				});

				xhr.open('PUT', signedData.uploadUrl);
				xhr.setRequestHeader('Content-Type', 'video/mp4');
				xhr.send(fileBuffer);
			});

			// Step 3: Return clip data (database record already created)
			return signedData.clip;
		} catch (err) {
			console.error('Error creating public clip:', err);
			throw err;
		}
	}

	async getClipByCode(shareCode: string) {
		try {
			const { data, error } = await auth.supabase
				.from('clips')
				.select('*')
				.eq('share_code', shareCode.toUpperCase())
				.single();

			if (error) throw error;

			return data;
		} catch (err) {
			console.error('Error fetching clip:', err);
			throw err;
		}
	}

	removeFromQueue(id: string) {
		const queueItem = this.uploadQueue.find(item => item.id === id);
		// Cancel if still uploading
		if (queueItem && queueItem.status === 'uploading') {
			this.cancelUpload(id);
		}
		this.uploadQueue = this.uploadQueue.filter(item => item.id !== id);
	}

	clearCompletedQueue() {
		this.uploadQueue = this.uploadQueue.filter(
			item => item.status !== 'completed' && item.status !== 'error' && item.status !== 'cancelled'
		);
	}
	
	clearErrorQueue() {
		this.uploadQueue = this.uploadQueue.filter(item => item.status !== 'error');
	}
}

export const cloudStorage = new CloudStorageStore();
