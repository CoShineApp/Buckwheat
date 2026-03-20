import type { GameWindow } from "$lib/commands";

export function scoreWindow(window: GameWindow): number {
	let score = 0;
	const title = window.window_title.toLowerCase();
	const className = window.class_name.toLowerCase();

	// Prioritize by class name (most reliable)
	if (className.includes("d3dproxy")) {
		score += 5000;
	}
	if (className.includes("wxwindownr")) {
		score += 1000;
	}

	// Check title for game-related keywords
	if (title.includes("slippi")) {
		score += 500;
	}
	if (title.includes("melee")) {
		score += 500;
	}
	if (title.includes("dolphin")) {
		score += 500;
	}
	if (title.includes("faster melee")) {
		score += 500;
	}

	// Prefer larger windows (actual game window vs small utility windows)
	if (window.width >= 640 && window.height >= 480) {
		score += 100;
	}

	// Penalize child windows or owned windows (they're often not the main game)
	if (window.is_child) {
		score -= 50;
	}
	if (window.has_owner) {
		score -= 50;
	}

	return score;
}
