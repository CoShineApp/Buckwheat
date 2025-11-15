import { Store } from "@tauri-apps/plugin-store";

export type Settings = {
	// Appearance
	theme: "light" | "dark" | "system";
	
	// Recording
	recordingPath: string;
	recordingQuality: "low" | "medium" | "high" | "ultra";
	autoStartRecording: boolean;
	
	// Slippi
	slippiPath: string;
	watchForGames: boolean;
	
	// Clips
	createClipHotkey: string;
	clipDuration: number;
};

const DEFAULT_SETTINGS: Settings = {
	theme: "system",
	recordingPath: "",
	recordingQuality: "high",
	autoStartRecording: true,
	slippiPath: "",
	watchForGames: true,
	createClipHotkey: "F9",
	clipDuration: 30,
};

class SettingsStore {
	private store: Store | null = null;
	
	// Reactive state
	theme = $state<Settings["theme"]>("system");
	recordingPath = $state("");
	recordingQuality = $state<Settings["recordingQuality"]>("high");
	autoStartRecording = $state(true);
	slippiPath = $state("");
	watchForGames = $state(true);
	createClipHotkey = $state("F9");
	clipDuration = $state(30);
	
	isLoading = $state(true);

	async init(): Promise<void> {
		// Idempotent - only initialize once
		if (this.store) {
			return;
		}

		try {
			this.store = await Store.load("settings.json");
			await this.load();
		} catch (error) {
			console.error("Failed to initialize settings store:", error);
			this.loadDefaults();
		} finally {
			this.isLoading = false;
		}
	}

	private async load(): Promise<void> {
		if (!this.store) return;

		const settings = await this.getAll();
		this.theme = settings.theme;
		this.recordingPath = settings.recordingPath;
		this.recordingQuality = settings.recordingQuality;
		this.autoStartRecording = settings.autoStartRecording;
		this.slippiPath = settings.slippiPath;
		this.watchForGames = settings.watchForGames;
		this.createClipHotkey = settings.createClipHotkey;
		this.clipDuration = settings.clipDuration;
	}

	private loadDefaults(): void {
		this.theme = DEFAULT_SETTINGS.theme;
		this.recordingPath = DEFAULT_SETTINGS.recordingPath;
		this.recordingQuality = DEFAULT_SETTINGS.recordingQuality;
		this.autoStartRecording = DEFAULT_SETTINGS.autoStartRecording;
		this.slippiPath = DEFAULT_SETTINGS.slippiPath;
		this.watchForGames = DEFAULT_SETTINGS.watchForGames;
		this.createClipHotkey = DEFAULT_SETTINGS.createClipHotkey;
		this.clipDuration = DEFAULT_SETTINGS.clipDuration;
	}

	private async getAll(): Promise<Settings> {
		if (!this.store) return DEFAULT_SETTINGS;

		return {
			theme: ((await this.store.get("theme")) as Settings["theme"]) ?? DEFAULT_SETTINGS.theme,
			recordingPath: ((await this.store.get("recordingPath")) as string) ?? DEFAULT_SETTINGS.recordingPath,
			recordingQuality: ((await this.store.get("recordingQuality")) as Settings["recordingQuality"]) ?? DEFAULT_SETTINGS.recordingQuality,
			autoStartRecording: ((await this.store.get("autoStartRecording")) as boolean) ?? DEFAULT_SETTINGS.autoStartRecording,
			slippiPath: ((await this.store.get("slippiPath")) as string) ?? DEFAULT_SETTINGS.slippiPath,
			watchForGames: ((await this.store.get("watchForGames")) as boolean) ?? DEFAULT_SETTINGS.watchForGames,
			createClipHotkey: ((await this.store.get("createClipHotkey")) as string) ?? DEFAULT_SETTINGS.createClipHotkey,
			clipDuration: ((await this.store.get("clipDuration")) as number) ?? DEFAULT_SETTINGS.clipDuration,
		};
	}

	async set<K extends keyof Settings>(key: K, value: Settings[K]): Promise<void> {
		// Update local state immediately for reactivity
		switch (key) {
			case "theme":
				this.theme = value as Settings["theme"];
				break;
			case "recordingPath":
				this.recordingPath = value as string;
				break;
			case "recordingQuality":
				this.recordingQuality = value as Settings["recordingQuality"];
				break;
			case "autoStartRecording":
				this.autoStartRecording = value as boolean;
				break;
			case "slippiPath":
				this.slippiPath = value as string;
				break;
			case "watchForGames":
				this.watchForGames = value as boolean;
				break;
			case "createClipHotkey":
				this.createClipHotkey = value as string;
				break;
			case "clipDuration":
				this.clipDuration = value as number;
				break;
		}
		
		// Persist to store if available
		if (this.store) {
			await this.store.set(key, value);
			await this.store.save();
		}
	}

	async reset(): Promise<void> {
		if (!this.store) return;

		const keys: (keyof Settings)[] = [
			"theme",
			"recordingPath",
			"recordingQuality",
			"autoStartRecording",
			"slippiPath",
			"watchForGames",
			"createClipHotkey",
			"clipDuration",
		];

		for (const key of keys) {
			await this.store.set(key, DEFAULT_SETTINGS[key]);
		}

		await this.store.save();
		this.loadDefaults();
	}
}

export const settings = new SettingsStore();

