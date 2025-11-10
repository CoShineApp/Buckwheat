import { toast } from "svelte-sonner";

/**
 * Tauri error structure from Rust
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
 * Get user-friendly error message from Tauri error
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
 * Handle Tauri errors and show toast notification
 * Use this from any component when catching Tauri command errors
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
		duration: 5000,
		closeButton: true
	});
}

/**
 * Show success toast
 */
export function showSuccess(message: string): void {
	toast.success(message, {
		duration: 3000
	});
}

/**
 * Show info toast
 */
export function showInfo(message: string): void {
	toast.info(message, {
		duration: 3000
	});
}

