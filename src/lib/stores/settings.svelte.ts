/**
 * Settings store for persisting user preferences.
 * Uses Tauri's plugin-store for cross-session persistence.
 *
 * @example
 * // Initialize on app startup
 * await settings.init();
 *
 * // Read settings reactively
 * if (settings.theme === 'dark') {
 *   document.body.classList.add('dark');
 * }
 *
 * // Update a setting
 * await settings.set('theme', 'dark');
 *
 * @module stores/settings
 */

import { Store } from "@tauri-apps/plugin-store";

/**
 * Application settings shape.
 * All settings are persisted to disk.
 */
export type Settings = {
	/** UI theme preference */
	theme: "light" | "dark" | "system";

	/** Directory where recordings are saved */
	recordingPath: string;
	/** Video quality preset for recordings */
	recordingQuality: "low" | "medium" | "high" | "ultra";
	/** Whether to auto-start recording when game is detected */
	autoStartRecording: boolean;

	/** Directory where Slippi .slp files are saved */
	slippiPath: string;
	/** Whether to watch for new .slp files */
	watchForGames: boolean;

	/** Keyboard shortcut for creating clips */
	createClipHotkey: string;
	/** Duration in seconds for clips */
	clipDuration: number;
};

/** Default settings values */
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

/**
 * Manages application settings with persistent storage.
 * Settings are reactive and automatically saved to disk.
 */
class SettingsStore {
	/** Tauri store instance for persistence */
	private store: Store | null = null;

	// Reactive state for each setting
	/** Current UI theme */
	theme = $state<Settings["theme"]>("system");
	/** Recording output directory */
	recordingPath = $state("");
	/** Video quality preset */
	recordingQuality = $state<Settings["recordingQuality"]>("high");
	/** Auto-start recording on game detection */
	autoStartRecording = $state(true);
	/** Slippi replay directory */
	slippiPath = $state("");
	/** Watch for new .slp files */
	watchForGames = $state(true);
	/** Hotkey for clip creation */
	createClipHotkey = $state("F9");
	/** Clip duration in seconds */
	clipDuration = $state(30);

	/** Whether settings are currently loading */
	isLoading = $state(true);

	/**
	 * Initialize the settings store.
	 * Loads settings from disk or uses defaults.
	 * Must be called before accessing settings.
	 */
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

	/** Load settings from persistent store */
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

	/** Reset reactive state to default values */
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

	/** Get all settings from persistent store */
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

	/**
	 * Update a setting value.
	 * Updates reactive state immediately and persists to disk.
	 * @param key - Setting key to update
	 * @param value - New value for the setting
	 */
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

	/**
	 * Reset all settings to their default values.
	 * Persists the defaults to disk.
	 */
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

/** Singleton settings store instance */
export const settings = new SettingsStore();

