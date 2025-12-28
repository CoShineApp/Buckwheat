/**
 * Navigation store for SPA-style page routing.
 * Manages current page state and navigation between views.
 *
 * @example
 * // Navigate to home
 * navigation.navigateTo('home');
 *
 * // Navigate to replay viewer
 * navigation.navigateToReplay('recording-123');
 *
 * // Check current page
 * if (navigation.currentPage === 'settings') {
 *   // Show settings
 * }
 *
 * @module stores/navigation
 */

/** Available page identifiers */
type Page = "home" | "settings" | "replay" | "cloud" | "profile" | "clips" | "stats";

/** Page state with type-safe replay info */
type PageInfo<TPage extends Page = Page> = TPage extends "replay"
	? { page: "replay"; replay: { id: string; isClip?: boolean } }
	: { page: Exclude<Page, "replay">; replay?: undefined };

/**
 * Manages navigation state for the single-page application.
 * Provides type-safe navigation methods and state access.
 */
class NavigationStore {
	/** Internal navigation state */
	private _state = $state<PageInfo>({ page: "home" });

	/** Full navigation state including replay info */
	get state(): PageInfo {
		return this._state;
	}

	/** Current page identifier */
	get currentPage(): Page {
		return this._state.page;
	}

	/** Current replay ID when on replay page, null otherwise */
	get replayId(): string | null {
		return this._state.page === "replay" ? this._state.replay.id : null;
	}

	/** Whether current replay is a clip (vs full recording) */
	get isClipReplay(): boolean {
		return this._state.page === "replay" ? Boolean(this._state.replay.isClip) : false;
	}

	/**
	 * Navigate to a standard page (not replay).
	 * @param page - Target page identifier
	 */
	navigateTo(page: Exclude<Page, "replay">): void {
		this._state = { page };
	}

	/**
	 * Navigate to the replay viewer for a recording.
	 * @param id - Recording or clip ID to view
	 * @param options - Optional settings (isClip: true for clip playback)
	 */
	navigateToReplay(id: string, options?: { isClip?: boolean }): void {
		this._state = { page: "replay", replay: { id, isClip: options?.isClip ?? false } };
	}

	/**
	 * Navigate to the replay viewer for a clip.
	 * Convenience method that sets isClip to true.
	 * @param id - Clip ID to view
	 */
	navigateToClipReplay(id: string): void {
		this._state = { page: "replay", replay: { id, isClip: true } };
	}

	/** Navigate to the clips list page */
	navigateToClips(): void {
		this._state = { page: "clips" };
	}

	/** Navigate back to the home page */
	navigateBack(): void {
		this._state = { page: "home" };
	}
}

/** Singleton navigation store instance */
export const navigation = new NavigationStore();

