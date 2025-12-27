/**
 * Authentication store for managing user sessions with Supabase.
 * Handles sign up, sign in, sign out, and profile management.
 *
 * @example
 * // Check if user is authenticated
 * if (auth.isAuthenticated) {
 *   console.log('Logged in as:', auth.user?.email);
 * }
 *
 * // Sign in
 * const result = await auth.signIn('user@example.com', 'password');
 * if (result.success) {
 *   console.log('Signed in successfully');
 * }
 *
 * @module stores/auth
 */

import { createClient, type User, type Session } from '@supabase/supabase-js';
import type { Profile } from '$lib/types/auth';

// Re-export types for convenience
export type { Profile } from '$lib/types/auth';

const supabaseUrl = import.meta.env.VITE_SUPABASE_URL || '';
const supabaseAnonKey = import.meta.env.VITE_SUPABASE_ANON_KEY || '';

/**
 * Manages authentication state and Supabase integration.
 * Automatically initializes on construction and listens for auth changes.
 */
class AuthStore {
	/** Current authenticated user, null if not logged in */
	user = $state<User | null>(null);
	/** Current session with access token */
	session = $state<Session | null>(null);
	/** User's profile with storage quota info */
	profile = $state<Profile | null>(null);
	/** Whether auth state is being loaded */
	loading = $state(true);
	/** Last error message, null if no error */
	error = $state<string | null>(null);
	/** Supabase client instance */
	supabase = createClient(supabaseUrl, supabaseAnonKey);

	constructor() {
		this.init();
	}

	/**
	 * Initialize auth state by getting current session and setting up listeners.
	 * Called automatically on construction.
	 */
	private async init() {
		try {
			// Get initial session
			const { data: { session } } = await this.supabase.auth.getSession();
			this.session = session;
			this.user = session?.user ?? null;

			if (this.user) {
				await this.loadProfile();
			}

			// Listen for auth changes
			this.supabase.auth.onAuthStateChange(async (_event, session) => {
				console.log('Auth state changed:', _event, session?.user?.email);
				this.session = session;
				this.user = session?.user ?? null;
				
				if (this.user) {
					await this.loadProfile();
				} else {
					this.profile = null;
				}
			});
		} catch (err) {
			console.error('Auth init error:', err);
			this.error = err instanceof Error ? err.message : 'Failed to initialize auth';
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Load the user's profile from the database.
	 * Updates the profile state with storage quota information.
	 */
	async loadProfile() {
		if (!this.user) return;

		try {
			const { data, error } = await this.supabase
				.from('profiles')
				.select('*')
				.eq('id', this.user.id)
				.single();

			if (error) {
				console.error('Error loading profile:', error);
				return;
			}

			this.profile = data;
		} catch (err) {
			console.error('Error loading profile:', err);
		}
	}

	/**
	 * Create a new user account.
	 * @param email - User's email address
	 * @param password - User's password
	 * @returns Result object with success status and optional error
	 */
	async signUp(email: string, password: string) {
		try {
			this.loading = true;
			this.error = null;

			const { data, error } = await this.supabase.auth.signUp({
				email,
				password,
			});

			if (error) {
				this.error = error.message;
				return { success: false, error: error.message };
			}

			// Profile will be created automatically by database trigger
			return { success: true, data };
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to sign up';
			this.error = message;
			return { success: false, error: message };
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Sign in with email and password.
	 * @param email - User's email address
	 * @param password - User's password
	 * @returns Result object with success status and optional error
	 */
	async signIn(email: string, password: string) {
		try {
			this.loading = true;
			this.error = null;

			const { data, error } = await this.supabase.auth.signInWithPassword({
				email,
				password,
			});

			if (error) {
				this.error = error.message;
				return { success: false, error: error.message };
			}

			return { success: true, data };
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to sign in';
			this.error = message;
			return { success: false, error: message };
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Sign out the current user.
	 * Clears user, session, and profile state.
	 * @returns Result object with success status and optional error
	 */
	async signOut() {
		try {
			this.loading = true;
			this.error = null;

			const { error } = await this.supabase.auth.signOut();

			if (error) {
				this.error = error.message;
				return { success: false, error: error.message };
			}

			this.user = null;
			this.session = null;
			this.profile = null;

			return { success: true };
		} catch (err) {
			const message = err instanceof Error ? err.message : 'Failed to sign out';
			this.error = message;
			return { success: false, error: message };
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Get the current access token for API calls.
	 * @returns Access token string or undefined if not authenticated
	 */
	getToken(): string | undefined {
		return this.session?.access_token;
	}

	/** Whether user is currently authenticated */
	get isAuthenticated(): boolean {
		return this.user !== null;
	}

	/** Percentage of storage quota used (0-100) */
	get storageUsedPercent(): number {
		if (!this.profile) return 0;
		return (this.profile.storage_used / this.profile.storage_limit) * 100;
	}

	/** Storage used in gigabytes */
	get storageUsedGB(): number {
		if (!this.profile) return 0;
		return this.profile.storage_used / (1024 ** 3);
	}

	/** Storage limit in gigabytes */
	get storageLimitGB(): number {
		if (!this.profile) return 0;
		return this.profile.storage_limit / (1024 ** 3);
	}
}

/** Singleton auth store instance */
export const auth = new AuthStore();

