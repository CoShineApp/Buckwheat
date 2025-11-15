type Page = "home" | "settings" | "replay" | "cloud" | "profile" | "clips";

type PageInfo<TPage extends Page = Page> = TPage extends "replay"
	? { page: "replay"; replay: { id: string; isClip?: boolean } }
	: TPage extends "clips"
	? { page: "clips"; replay?: undefined }
	: { page: Exclude<Page, "replay" | "clips">; replay?: undefined };

class NavigationStore {
	private _state = $state<PageInfo>({ page: "home" });

	get state(): PageInfo {
		return this._state;
	}

	get currentPage(): Page {
		return this._state.page;
	}

	get replayId(): string | null {
		return this._state.page === "replay" ? this._state.replay.id : null;
	}

	get isClipReplay(): boolean {
		return this._state.page === "replay" ? Boolean(this._state.replay.isClip) : false;
	}

	navigateTo(page: Exclude<Page, "replay" | "clips">): void {
		this._state = { page };
	}

	navigateToReplay(id: string, options?: { isClip?: boolean }): void {
		this._state = { page: "replay", replay: { id, isClip: options?.isClip ?? false } };
	}

	navigateToClipReplay(id: string): void {
		this._state = { page: "replay", replay: { id, isClip: true } };
	}

	navigateToClips(): void {
		this._state = { page: "clips" };
	}

	navigateBack(): void {
		this._state = { page: "home" };
	}
}

export const navigation = new NavigationStore();

