/**
 * Tauri command wrappers for game window detection and management.
 * Provides type-safe functions for invoking Rust backend commands.
 *
 * @example
 * import { checkGameWindow, listGameWindows, setGameProcessName } from '$lib/commands';
 *
 * // Check if game is running
 * const isRunning = await checkGameWindow();
 *
 * // List available game windows
 * const windows = await listGameWindows();
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
 * Check if the Slippi/Dolphin game window is currently open.
 * Uses the configured game process name for detection.
 * @returns True if game window is detected, false otherwise
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
 * Information about a detected game window.
 * Returned by listGameWindows() for user selection.
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
 * List all detected game windows for user selection.
 * Filters for windows that look like Dolphin/Slippi.
 * @returns Array of detected game windows (empty if none found)
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
 * Capture a single-frame preview of the selected game window.
 * Used for visual confirmation of window selection.
 * @returns Base64-encoded PNG string, or null if capture failed
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
 * Get the currently configured game process identifier.
 * @returns The stored process identifier, or null if not set
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
 * Set the game process identifier for detection and recording.
 * This is used to identify which window to capture.
 * @param processName - Process identifier (e.g., "Slippi Dolphin (PID: 12345)")
 * @throws Error if setting fails
 */
export async function setGameProcessName(processName: string): Promise<void> {
    try {
        await invoke('set_game_process_name', { processName });
    } catch (error) {
        console.error('Failed to set game process name:', error);
        throw error;
    }
}

