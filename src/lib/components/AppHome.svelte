<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card";
	import { recording } from "$lib/stores/recording.svelte";
	import { handleTauriError, showSuccess } from "$lib/utils/errors";
	import { invoke } from "@tauri-apps/api/core";
	import { Play, Square } from "@lucide/svelte";

	let isStarting = $state(false);
	let isStopping = $state(false);
	let lastRecordingPath = $state<string | null>(null);

	async function startRecording() {
		isStarting = true;

		try {
			// Get recording directory (handles defaults and creates directory if needed)
			const recordingDir = await invoke<string>("get_recording_directory");
			
			// Generate filename with timestamp
			const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
			const filename = `recording-${timestamp}.mp4`;
			const outputPath = `${recordingDir}/${filename}`;
			
			await invoke("start_recording", { outputPath });
			recording.start();
			lastRecordingPath = outputPath;
			showSuccess("Recording started");
		} catch (e) {
			handleTauriError(e, "Failed to start recording");
		} finally {
			isStarting = false;
		}
	}

	async function stopRecording() {
		isStopping = true;

		try {
			const path = await invoke<string>("stop_recording");
			recording.stop();
			lastRecordingPath = path;
			showSuccess("Recording stopped");
		} catch (e) {
			handleTauriError(e, "Failed to stop recording");
		} finally {
			isStopping = false;
		}
	}
</script>

<div class="flex h-full items-center justify-center">
	<Card class="w-full max-w-2xl">
		<CardHeader>
			<CardTitle>Screen Recording</CardTitle>
			<CardDescription>
				Test the screen recording functionality
			</CardDescription>
		</CardHeader>
		<CardContent class="space-y-4">
			<div class="flex gap-3">
				<Button
					onclick={startRecording}
					disabled={recording.isRecording || isStarting}
					class="flex-1"
				>
					<Play class="size-4" />
					{isStarting ? "Starting..." : "Start Recording"}
				</Button>
				
				<Button
					onclick={stopRecording}
					disabled={!recording.isRecording || isStopping}
					variant="destructive"
					class="flex-1"
				>
					<Square class="size-4" />
					{isStopping ? "Stopping..." : "Stop Recording"}
				</Button>
			</div>

			{#if recording.isRecording}
				<div class="rounded-lg border border-red-500/20 bg-red-500/10 p-4">
					<div class="flex items-center gap-2">
						<div class="size-2 animate-pulse rounded-full bg-red-500"></div>
						<span class="font-semibold text-red-500">Recording in progress...</span>
					</div>
					<p class="mt-1 text-sm text-muted-foreground">
						Check the sidebar for the live indicator
					</p>
				</div>
			{/if}

			{#if lastRecordingPath && !recording.isRecording}
				<div class="rounded-lg border bg-muted p-4">
					<p class="text-sm font-medium">Last recording saved:</p>
					<p class="mt-1 break-all text-xs text-muted-foreground">{lastRecordingPath}</p>
				</div>
			{/if}
		</CardContent>
	</Card>
</div>

