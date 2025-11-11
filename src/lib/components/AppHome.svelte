<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card";
	import { recording } from "$lib/stores/recording.svelte";
	import { handleTauriError, showSuccess } from "$lib/utils/errors";
	import { invoke } from "@tauri-apps/api/core";
	import { Square, Settings, Loader2 } from "@lucide/svelte";
	import RecordingStats from "$lib/components/recordings/RecordingStats.svelte";
	import RecordingsTable from "$lib/components/recordings/RecordingsTable.svelte";
	import BatchActions from "$lib/components/recordings/BatchActions.svelte";
	import { listen } from "@tauri-apps/api/event";
	import { onMount } from "svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { recordingsStore } from "$lib/stores/recordings.svelte";

	let isStopping = $state(false);
	let lastReplayPath = $state<string | null>(null);

	// Listen for auto-recording events
	onMount(() => {
		console.log("🚀 AppHome mounted, setting up event listeners");
		
		// Load recordings on mount
		recordingsStore.refresh();
		
		// Get initial last replay path
		invoke<string | null>("get_last_replay_path")
			.then((path) => {
				console.log("📥 Initial last replay path:", path);
				if (path) {
					lastReplayPath = path;
					console.log("✅ Set lastReplayPath to:", lastReplayPath);
				} else {
					console.log("⚠️ No last replay path available yet");
				}
			})
			.catch((error) => {
				console.error("❌ Failed to get last replay path:", error);
			});

		const unlistenRecordingStarted = listen("recording-started", () => {
			console.log("📥 Received recording-started event");
			recording.start();
			showSuccess("Auto-recording started");
		});

		const unlistenLastReplay = listen<string>("last-replay-updated", (event) => {
			console.log("📥 Received last-replay-updated event");
			console.log("📦 Event payload:", event.payload);
			lastReplayPath = event.payload;
			console.log("✅ Updated lastReplayPath to:", lastReplayPath);
		});

		const unlistenRecordingStopped = listen("recording-stopped", () => {
			console.log("📥 Received recording-stopped event");
			recording.stop();
			showSuccess("Recording stopped automatically");
			// Refresh recordings list when a recording stops
			recordingsStore.refresh();
		});

		console.log("✅ Event listeners set up successfully");

		return () => {
			console.log("🧹 Cleaning up event listeners");
			unlistenRecordingStarted.then(fn => fn());
			unlistenLastReplay.then(fn => fn());
			unlistenRecordingStopped.then(fn => fn());
		};
	});

	function getReplayFileName(path: string): string {
		const parts = path.split(/[\\/]/);
		return parts[parts.length - 1] || path;
	}

	function formatReplayTime(path: string): string {
		// Extract timestamp from typical Slippi filename format
		// Example: Game_20250111T123456.slp
		const match = path.match(/(\d{8})T(\d{6})/);
		if (match) {
			const date = match[1]; // YYYYMMDD
			const time = match[2]; // HHMMSS
			const year = date.substring(0, 4);
			const month = date.substring(4, 6);
			const day = date.substring(6, 8);
			const hour = time.substring(0, 2);
			const minute = time.substring(2, 4);
			return `${month}/${day}/${year} ${hour}:${minute}`;
		}
		return "";
	}

	async function stopRecording() {
		isStopping = true;

		try {
			await invoke<string>("stop_recording");
			recording.stop();
			showSuccess("Recording stopped");
		} catch (e) {
			handleTauriError(e, "Failed to stop recording");
		} finally {
			isStopping = false;
		}
	}

</script>

<div class="flex h-full flex-col gap-6 p-6">
	<!-- Stats Dashboard -->
	<RecordingStats />

	<!-- Quick Actions Card -->
	<Card>
		<CardHeader>
			<div class="flex items-center justify-between">
				<div>
					<CardTitle>Quick Actions</CardTitle>
					<CardDescription>
						Control your screen recording
					</CardDescription>
				</div>
			</div>
		</CardHeader>
		<CardContent>
			<div class="space-y-4">
				{#if !settings.autoStartRecording}
					<div class="rounded-lg border border-yellow-500/20 bg-yellow-500/10 p-4">
						<p class="text-sm text-yellow-600 dark:text-yellow-400">
							Auto-recording is disabled. Enable it in settings to automatically record when games start.
						</p>
					</div>
				{:else if !recording.isRecording}
					{#if lastReplayPath}
						<div class="rounded-lg border bg-muted p-3">
							<p class="text-xs font-medium text-muted-foreground uppercase tracking-wide">Last Replay Detected</p>
							<p class="mt-1 text-sm font-mono truncate" title={lastReplayPath}>
								{getReplayFileName(lastReplayPath)}
							</p>
							{#if formatReplayTime(lastReplayPath)}
								<p class="mt-0.5 text-xs text-muted-foreground">
									{formatReplayTime(lastReplayPath)}
								</p>
							{/if}
						</div>
					{/if}
					
					<div class="rounded-lg border border-blue-500/20 bg-blue-500/10 p-4">
						<div class="flex items-center gap-2">
							<Loader2 class="size-4 animate-spin text-blue-500" />
							<span class="text-sm font-medium text-blue-600 dark:text-blue-400">
								Waiting for game to start...
							</span>
						</div>
						<p class="mt-1 text-xs text-muted-foreground">
							Recording will start automatically when a new game begins
						</p>
					</div>
				{/if}

				<div class="flex gap-3">
					<Button
						onclick={stopRecording}
						disabled={!recording.isRecording || isStopping}
						variant="destructive"
						size="lg"
						class="flex-1"
					>
						<Square class="size-4" />
						{isStopping ? "Stopping..." : "Stop Recording"}
					</Button>

					<Button
						variant="outline"
						size="lg"
						onclick={() => console.log("Open settings")}
						title="Settings"
					>
						<Settings class="size-4" />
					</Button>
				</div>
			</div>
		</CardContent>
	</Card>

	<!-- Recordings Table -->
	<RecordingsTable />

	<!-- Batch Actions Toolbar (floats at bottom when items selected) -->
	<BatchActions />
</div>

