/**
 * Authentication type definitions.
 * Used for user profiles and session management.
 * @module types/auth
 */

/**
 * User profile stored in the database.
 * Contains storage quota information and user metadata.
 */
export interface Profile {
	/** User's unique identifier (matches Supabase auth user ID) */
	id: string;
	/** Device ID for anonymous clip uploads */
	device_id: string | null;
	/** Current storage usage in bytes */
	storage_used: number;
	/** Maximum storage limit in bytes */
	storage_limit: number;
	/** ISO timestamp when the profile was created */
	created_at: string;
	/** ISO timestamp when the profile was last updated */
	updated_at: string;
}

