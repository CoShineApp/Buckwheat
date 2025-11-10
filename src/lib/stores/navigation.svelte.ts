type Page = "home" | "settings";

class NavigationStore {
	currentPage = $state<Page>("home");

	navigateTo(page: Page): void {
		this.currentPage = page;
	}
}

export const navigation = new NavigationStore();

