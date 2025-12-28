/**
 * Recording state store for tracking current recording status.
 * Monitors game window detection, active game state, and recording progress.
 *
 * @example
 * // Check recording status
 * if (recording.isRecording) {
 *   console.log('Recording started at:', recording.startTimestamp);
 * }
 *
 * // React to status changes
 * $effect(() => {
 *   console.log('Recording status:', recording.status);
 * });
 *
 * @module stores/recording
 */

/** Recording status indicator values */
export type RecordingStatus = "recording" | "ready" | "waiting" | "no-window";

/**
 * Tracks the current recording state and game window status.
 * Provides derived status for UI indicators.
 */
class RecordingStore {
	/** Whether recording is currently active */
	isRecording = $state(false);
	/** Whether the Dolphin/Slippi game window is detected */
	gameWindowDetected = $state(false);
	/** Whether an active game is detected (via .slp file) */
	gameActive = $state(false);
	/** Timestamp when recording started (for duration calculation) */
	startTimestamp = $state<number | null>(null);
	/** Path to current recording/replay file */
	currentReplayPath = $state<string | null>(null);

	/**
	 * Derived status for UI indicator.
	 * - "recording": Active recording in progress
	 * - "ready": Game detected, ready to record
	 * - "waiting": Window found, waiting for game
	 * - "no-window": No game window detected
	 */
	status = $derived.by((): RecordingStatus => {
		if (this.isRecording) return "recording";
		if (this.gameActive) return "ready";
		if (this.gameWindowDetected) return "waiting";
		return "no-window";
	});

	/**
	 * Start a new recording.
	 * @param timestamp - Start timestamp (defaults to Date.now())
	 */
	start(timestamp: number = Date.now()) {
		this.isRecording = true;
		this.startTimestamp = timestamp;
	}

	/** Stop the current recording and reset state */
	stop() {
		this.isRecording = false;
		this.startTimestamp = null;
	}

	/**
	 * Update game window detection status.
	 * @param detected - Whether game window is visible
	 */
	setGameWindow(detected: boolean) {
		this.gameWindowDetected = detected;
	}

	/**
	 * Update active game detection status.
	 * @param active - Whether an active game is detected
	 */
	setGameActive(active: boolean) {
		this.gameActive = active;
	}

	/**
	 * Set the current replay/recording file path.
	 * @param path - Path to replay file, or null when not recording
	 */
	setReplayPath(path: string | null) {
		this.currentReplayPath = path;
	}
}

/** Singleton recording state store instance */
export const recording = new RecordingStore();


