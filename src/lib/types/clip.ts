/**
 * Clip-related type definitions.
 * Used for managing clip markers and clip session data.
 * @module types/clip
 */

import type { SlippiMetadata } from './recording';

/**
 * Represents a marker placed during recording to create a clip.
 * Markers are processed after recording ends to extract clip segments.
 */
export interface ClipMarker {
	/** Timestamp in seconds from the start of the recording */
	timestamp: number;
	/** Path to the recording file this marker belongs to */
	recordingFile: string;
}

/**
 * Represents a saved clip session with its metadata.
 * Clips are short video segments extracted from full recordings.
 */
export interface ClipSession {
	/** Unique identifier for the clip */
	id: string;
	/** Display filename for the clip */
	filename: string;
	/** Full path to the video file */
	video_path: string;
	/** Path to the thumbnail image, if available */
	thumbnail_path: string | null;
	/** ISO timestamp when the clip was created */
	start_time: string;
	/** Duration of the clip in seconds */
	duration: number | null;
	/** Size of the video file in bytes */
	file_size: number | null;
	/** Path to associated .slp file, if any */
	slp_path: string | null;
	/** Slippi metadata from the associated replay file */
	slippi_metadata: SlippiMetadata | null;
}

