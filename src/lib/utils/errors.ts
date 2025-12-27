/**
 * Error handling utilities for Tauri commands.
 * Provides user-friendly error messages and toast notifications.
 *
 * @example
 * import { handleTauriError, showSuccess } from '$lib/utils/errors';
 *
 * try {
 *   await invoke('some_command');
 *   showSuccess('Operation completed!');
 * } catch (e) {
 *   handleTauriError(e, 'Failed to do something');
 * }
 *
 * @module utils/errors
 */

import { toast } from "svelte-sonner";

/**
 * Error structure returned by Tauri commands from Rust backend.
 */
interface TauriError {
	message: string;
	name: string;
}

/**
 * User-friendly error messages for common Tauri errors
 */
const ERROR_MESSAGES: Record<string, string> = {
	initializationError: "Failed to initialize",
	recordingFailed: "Recording operation failed",
	windowNotFound: "Dolphin window not found",
	unsupportedPlatform: "This feature is not supported on your platform",
	watchError: "Failed to watch folder",
	invalidPath: "Invalid file path",
	permissionError: "Permission denied"
};

/**
 * Extract a user-friendly message from various error types.
 * @param error - Error from Tauri command, JS Error, or unknown
 * @returns Human-readable error message
 */
function getErrorMessage(error: TauriError | Error | unknown): string {
	// Handle Tauri error objects
	if (typeof error === "object" && error !== null && "name" in error && "message" in error) {
		const tauriError = error as TauriError;
		
		// Check if it's a permission error
		if (tauriError.message.toLowerCase().includes("permission")) {
			return `Permission Error: ${tauriError.message}`;
		}
		
		// Use custom message if available
		const customMessage = ERROR_MESSAGES[tauriError.name];
		if (customMessage) {
			return `${customMessage}: ${tauriError.message}`;
		}
		
		return tauriError.message;
	}
	
	// Handle regular Error objects
	if (error instanceof Error) {
		return error.message;
	}
	
	// Fallback
	return String(error);
}

/**
 * Handle Tauri errors with user feedback.
 * Logs the error to console and shows a toast notification.
 *
 * @param error - The caught error (from Tauri command or other source)
 * @param context - Optional context message to prepend (e.g., "Failed to start recording")
 *
 * @example
 * try {
 *   await invoke("start_recording", { outputPath });
 * } catch (e) {
 *   handleTauriError(e, "Failed to start recording");
 * }
 */
export function handleTauriError(
	error: unknown,
	context?: string
): void {
	const message = getErrorMessage(error);
	const fullMessage = context ? `${context}: ${message}` : message;
	
	console.error(fullMessage, error);
	
	toast.error(fullMessage, {
		duration: 5000
	});
}

/**
 * Show a success toast notification.
 * @param message - Success message to display
 */
export function showSuccess(message: string): void {
	toast.success(message, {
		duration: 3000
	});
}

/**
 * Show an informational toast notification.
 * @param message - Info message to display
 */
export function showInfo(message: string): void {
	toast.info(message, {
		duration: 3000
	});
}

