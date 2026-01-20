/**
 * General formatting utilities for displaying data in the UI.
 * @module utils/format
 */

/**
 * Format game duration from frames to a readable time string (MM:SS).
 * Melee runs at 60 FPS, so frames are divided by 60 to get seconds.
 *
 * @param frames - Number of frames in the game
 * @returns Formatted time string (e.g., "2:34")
 *
 * @example
 * formatGameDuration(7200) // Returns "2:00" (2 minutes)
 * formatGameDuration(9000) // Returns "2:30" (2 minutes 30 seconds)
 */
export function formatGameDuration(frames: number): string {
	const seconds = Math.floor(frames / 60); // Melee runs at 60 FPS
	const minutes = Math.floor(seconds / 60);
	const remainingSeconds = seconds % 60;
	return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
}

/**
 * Format file size in bytes to a human-readable string.
 *
 * @param bytes - File size in bytes (or null/undefined)
 * @returns Formatted size string with appropriate unit (B, KB, MB, GB), or "Unknown" if null
 *
 * @example
 * formatFileSize(1024)      // Returns "1.0 KB"
 * formatFileSize(1048576)   // Returns "1.0 MB"
 * formatFileSize(1073741824) // Returns "1.00 GB"
 * formatFileSize(null)      // Returns "Unknown"
 */
export function formatFileSize(bytes: number | null | undefined): string {
	if (bytes == null) return "Unknown";
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
	if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

/**
 * Format a timestamp as a relative time string (e.g., "2 hours ago").
 * For timestamps older than 7 days, returns a formatted date.
 *
 * @param timestamp - ISO timestamp string
 * @returns Relative time string or formatted date
 *
 * @example
 * formatRelativeTime(new Date().toISOString()) // Returns "just now"
 * formatRelativeTime(Date.now() - 3600000)     // Returns "1h ago"
 */
export function formatRelativeTime(timestamp: string): string {
	const now = new Date();
	const then = new Date(timestamp);
	const diffMs = now.getTime() - then.getTime();
	const diffSeconds = Math.floor(diffMs / 1000);
	const diffMinutes = Math.floor(diffSeconds / 60);
	const diffHours = Math.floor(diffMinutes / 60);
	const diffDays = Math.floor(diffHours / 24);

	if (diffSeconds < 60) return "just now";
	if (diffMinutes < 60) return `${diffMinutes}m ago`;
	if (diffHours < 24) return `${diffHours}h ago`;
	if (diffDays < 7) return `${diffDays}d ago`;

	// Return formatted date for older items
	return then.toLocaleDateString();
}

/**
 * Format a duration in seconds to a readable time string (MM:SS).
 *
 * @param seconds - Duration in seconds (or null/undefined)
 * @returns Formatted time string (e.g., "2:34"), or "Unknown" if null
 *
 * @example
 * formatDuration(120)  // Returns "2:00"
 * formatDuration(150)  // Returns "2:30"
 * formatDuration(null) // Returns "Unknown"
 */
export function formatDuration(seconds: number | null | undefined): string {
	if (seconds == null) return "Unknown";
	const minutes = Math.floor(seconds / 60);
	const remainingSeconds = seconds % 60;
	return `${minutes}:${String(remainingSeconds).padStart(2, "0")}`;
}

export function getLCancelPercent(success: number, fail: number): number | null {
	const total = success + fail;
	if (total === 0) return null;
	return Math.round((success / total) * 100);
}

export function formatLCancelDisplay(success: number, fail: number): string {
	const pct = getLCancelPercent(success, fail);
	if (pct === null) return "-";
	return `${pct}%`;
}

export function formatRatio(ratio: number | null | undefined): string {
	if (ratio == null) return "-";
	return `${Math.round(ratio * 100)}%`;
}

export function formatDecimal(num: number | null | undefined, decimals = 1): string {
	if (num == null) return "-";
	return num.toFixed(decimals);
}
