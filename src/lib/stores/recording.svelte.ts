/**
 * Recording state store
 * Tracks whether a match is currently being recorded and game window status
 */
class RecordingStore {
	isRecording = $state(false);
	gameWindowDetected = $state(false);
	gameActive = $state(false); // Will be set based on .slp file detection later
	startTimestamp = $state<number | null>(null);
	currentReplayPath = $state<string | null>(null);

	// Derived status for the indicator
	status = $derived.by(() => {
		if (this.isRecording) return "recording";
		if (this.gameActive) return "ready";
		if (this.gameWindowDetected) return "waiting";
		return "no-window";
	});

	start(timestamp: number = Date.now()) {
		this.isRecording = true;
		this.startTimestamp = timestamp;
	}

	stop() {
		this.isRecording = false;
		this.startTimestamp = null;
	}

	setGameWindow(detected: boolean) {
		this.gameWindowDetected = detected;
	}

	setGameActive(active: boolean) {
		this.gameActive = active;
	}

	setReplayPath(path: string | null) {
		this.currentReplayPath = path;
	}
}

export const recording = new RecordingStore();


