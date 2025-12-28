/**
 * Centralized utility exports for the Bunbun application.
 * Import utilities from this module for cleaner imports.
 *
 * @example
 * import { formatFileSize, getCharacterName, handleTauriError } from '$lib/utils';
 *
 * @module utils
 */

// Character and stage data utilities
export {
	CHARACTER_NAMES,
	STAGE_NAMES,
	getCharacterName,
	getCharacterSlug,
	getCharacterImage,
	getStageSlug,
	getStageImage,
	getStageName,
} from './characters';

// Error handling utilities
export { handleTauriError, showSuccess, showInfo } from './errors';

// Formatting utilities
export {
	formatGameDuration,
	formatFileSize,
	formatRelativeTime,
	formatDuration,
} from './format';

// Slippi file parsing utilities
export {
	parseSlippiFile,
	parseSlippiFileWithCache,
	clearSlippiCache,
	removeFromCache,
} from './slippi';

