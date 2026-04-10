/**
 * Tauri command wrappers for game window detection and management.
 *
 * @deprecated These commands predate the OBS sidecar integration. OBS uses
 * game_capture with any_fullscreen mode, so manual window selection is no
 * longer needed for recording. Kept for the sidebar status indicator only.
 *
 * @module commands
 */

import { invoke } from '@tauri-apps/api/core';

/**
 * Utility to wrap an event handler with preventDefault.
 * @param fn - Event handler function
 * @returns Wrapped handler that calls preventDefault before the original
 */
export const preventDefault = <T extends Event>(fn: (e: T) => void): ((e: T) => void) => {
    return (e: T) => {
        e.preventDefault();
        fn(e);
    };
};

/**
 * @deprecated No longer needed for recording — OBS handles game capture directly.
 * Kept for sidebar status indicator.
 */
export async function checkGameWindow(): Promise<boolean> {
    try {
        return await invoke<boolean>('check_game_window');
    } catch (error) {
        console.error('Failed to check game window:', error);
        return false;
    }
}

/**
 * @deprecated No longer needed for recording — OBS handles game capture directly.
 */
export interface GameWindow {
    /** Name of the process (e.g., "Dolphin.exe") */
    process_name: string;
    /** Window title text */
    window_title: string;
    /** Window width in pixels */
    width: number;
    /** Window height in pixels */
    height: number;
    /** Operating system process ID */
    process_id: number;
    /** Window class name (platform-specific) */
    class_name: string;
    /** Whether the window is cloaked/hidden (Windows) */
    is_cloaked: boolean;
    /** Whether this is a child window */
    is_child: boolean;
    /** Whether this window has an owner window */
    has_owner: boolean;
}

/**
 * @deprecated No longer needed for recording — OBS handles game capture directly.
 */
export async function listGameWindows(): Promise<GameWindow[]> {
    try {
        return await invoke<GameWindow[]>('list_game_windows');
    } catch (error) {
        console.error('Failed to list game windows:', error);
        return [];
    }
}

/**
 * @deprecated No longer needed for recording — OBS handles game capture directly.
 */
export async function captureWindowPreview(): Promise<string | null> {
    try {
        return await invoke<string | null>('capture_window_preview');
    } catch (error) {
        console.error('Failed to capture window preview:', error);
        return null;
    }
}

/**
 * @deprecated No longer needed for recording — OBS handles game capture directly.
 */
export async function highlightGameWindow(processId: number): Promise<void> {
    try {
        await invoke('highlight_game_window', { processId });
    } catch (error) {
        console.error('Failed to highlight window:', error);
    }
}

/**
 * @deprecated No longer needed for recording — OBS handles game capture directly.
 */
export async function getGameProcessName(): Promise<string | null> {
    try {
        return await invoke<string | null>('get_game_process_name');
    } catch (error) {
        console.error('Failed to get game process name:', error);
        return null;
    }
}

/**
 * @deprecated No longer needed for recording — OBS handles game capture directly.
 */
export async function setGameProcessName(processName: string): Promise<void> {
    try {
        await invoke('set_game_process_name', { processName });
    } catch (error) {
        console.error('Failed to set game process name:', error);
        throw error;
    }
}

