/**
 * Recording state store
 * Tracks whether a match is currently being recorded
 */
class RecordingStore {
	isRecording = $state(false);

	start() {
		this.isRecording = true;
	}

	stop() {
		this.isRecording = false;
	}
}

export const recording = new RecordingStore();

