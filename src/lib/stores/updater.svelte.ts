import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

interface UpdaterState {
	checking: boolean;
	available: boolean;
	downloading: boolean;
	progress: number;
	update: Update | null;
	error: string | null;
}

function createUpdaterStore() {
	let state = $state<UpdaterState>({
		checking: false,
		available: false,
		downloading: false,
		progress: 0,
		update: null,
		error: null
	});

	async function checkForUpdates() {
		state.checking = true;
		state.error = null;

		try {
			const update = await check();
			
			if (update) {
				state.available = true;
				state.update = update;
				console.log(`Update available: ${update.version}`);
			} else {
				state.available = false;
				state.update = null;
				console.log('No updates available');
			}
		} catch (error) {
			console.error('Failed to check for updates:', error);
			state.error = error instanceof Error ? error.message : 'Failed to check for updates';
		} finally {
			state.checking = false;
		}
	}

	async function downloadAndInstall() {
		if (!state.update) return;

		state.downloading = true;
		state.progress = 0;
		state.error = null;

		try {
			let downloaded = 0;
			let contentLength = 0;

			await state.update.downloadAndInstall((event) => {
				switch (event.event) {
					case 'Started':
						contentLength = event.data.contentLength ?? 0;
						console.log(`Download started, size: ${contentLength}`);
						break;
					case 'Progress':
						downloaded += event.data.chunkLength;
						if (contentLength > 0) {
							state.progress = Math.round((downloaded / contentLength) * 100);
						}
						break;
					case 'Finished':
						console.log('Download finished');
						state.progress = 100;
						break;
				}
			});

			// Relaunch the app to apply the update
			await relaunch();
		} catch (error) {
			console.error('Failed to download/install update:', error);
			state.error = error instanceof Error ? error.message : 'Failed to install update';
		} finally {
			state.downloading = false;
		}
	}

	function dismiss() {
		state.available = false;
		state.update = null;
	}

	return {
		get state() {
			return state;
		},
		checkForUpdates,
		downloadAndInstall,
		dismiss
	};
}

export const updater = createUpdaterStore();
