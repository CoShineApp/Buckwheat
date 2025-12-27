/**
 * Cloud storage type definitions.
 * Used for managing uploads, clips, and upload queue state.
 * @module types/cloud
 */

/**
 * Represents a video upload stored in cloud storage.
 * Tracks upload status and metadata for the Backblaze B2 storage.
 */
export interface Upload {
	/** Unique identifier for the upload */
	id: string;
	/** User ID who owns the upload */
	user_id: string;
	/** Original filename of the uploaded video */
	filename: string;
	/** Backblaze B2 file ID, set after successful upload */
	b2_file_id: string | null;
	/** Backblaze B2 file name/path */
	b2_file_name: string | null;
	/** Size of the file in bytes */
	file_size: number;
	/** Duration of the video in seconds */
	duration_seconds: number | null;
	/** ISO timestamp when the upload was created */
	uploaded_at: string;
	/** Additional metadata (Slippi data, recording info, etc.) */
	metadata: Record<string, unknown> | null;
	/** Current status of the upload */
	status: 'UPLOADING' | 'UPLOADED' | 'FAILED';
}

/**
 * Represents a publicly shared clip in cloud storage.
 * Clips can be shared via a short share code without authentication.
 */
export interface CloudClip {
	/** Unique identifier for the clip */
	id: string;
	/** User ID who uploaded the clip, null for anonymous uploads */
	user_id: string | null;
	/** Device ID that created the clip */
	device_id: string | null;
	/** Original filename of the clip */
	filename: string;
	/** Backblaze B2 file ID */
	b2_file_id: string | null;
	/** Backblaze B2 file name/path */
	b2_file_name: string | null;
	/** Size of the file in bytes */
	file_size: number;
	/** Duration of the clip in seconds */
	duration_seconds: number | null;
	/** Short code for sharing (e.g., "ABC123") */
	share_code: string;
	/** ISO timestamp when the clip was uploaded */
	uploaded_at: string;
	/** Additional metadata */
	metadata: Record<string, unknown> | null;
}

/**
 * Status of an item in the upload queue.
 */
export type UploadStatus = 'pending' | 'uploading' | 'completed' | 'error' | 'cancelled';

/**
 * Represents an item in the local upload queue.
 * Tracks upload progress and allows for cancellation.
 */
export interface UploadQueueItem {
	/** Unique identifier for the queue item */
	id: string;
	/** Local path to the video being uploaded */
	videoPath: string;
	/** Current status of the upload */
	status: UploadStatus;
	/** Upload progress as percentage (0-100) */
	progress: number;
	/** Error message if status is 'error' */
	error?: string;
	/** Server-side upload ID once created */
	uploadId?: string;
	/** AbortController for cancellation support */
	abortController?: AbortController;
	/** XMLHttpRequest instance for progress tracking */
	xhr?: XMLHttpRequest;
}

