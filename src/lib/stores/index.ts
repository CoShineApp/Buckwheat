/**
 * Centralized store exports for the Bunbun application.
 * Import stores from this module for cleaner imports.
 *
 * @example
 * import { auth, recording, settings } from '$lib/stores';
 *
 * @module stores
 */

// Authentication store
export { auth } from './auth.svelte';
export type { Profile } from './auth.svelte';

// Clips store
export { clipsStore } from './clips.svelte';
export type { ClipMarker, ClipSession } from './clips.svelte';

// Cloud storage store
export { cloudStorage } from './cloud-storage.svelte';
export type { Upload, CloudClip, UploadQueueItem } from './cloud-storage.svelte';

// Navigation store
export { navigation } from './navigation.svelte';

// Current recording state store
export { recording } from './recording.svelte';

// Recordings list store
export { recordingsStore } from './recordings.svelte';

// Settings store
export { settings } from './settings.svelte';
export type { Settings } from './settings.svelte';

