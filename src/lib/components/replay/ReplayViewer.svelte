<script lang="ts">
import { navigation } from '$lib/stores/navigation.svelte';
import { recordingsStore } from '$lib/stores/recordings.svelte';
import type { ClipSession } from '$lib/stores/clips.svelte';
import type { RecordingWithMetadata } from '$lib/types/recording';
import type { GameEvent } from '$lib/types/recording';
import VideoPlayer from './VideoPlayer.svelte';
import Timeline from './Timeline.svelte';
import StatsPanel from './StatsPanel.svelte';
import { Button } from '$lib/components/ui/button';
import { ArrowLeft } from '@lucide/svelte';

let { recordingId, isClip }: { recordingId: string; isClip?: boolean } = $props();

let playerRef: VideoPlayer;
let recording = $state<ClipSession | RecordingWithMetadata | undefined>(undefined);
let events = $state<GameEvent[]>([]);
let currentTime = $state(0);
let duration = $state(0);
let isLoadingEvents = $state(false);

const isClipOnly = $derived(recordingsStore.isClipOnly(recording));
const slippiMetadata = $derived(recording?.slippi_metadata ?? null);
const videoPath = $derived(recording?.video_path ?? null);

// Reactively load recording when recordingId or isClip changes
$effect(() => {
	if (!recordingId) {
		recording = undefined;
		events = [];
		return;
	}

	// Async loading function
	(async () => {
		// Get recording from appropriate store
		if (isClip) {
			recording = await recordingsStore.getClipRecording(recordingId);
			console.log('📹 Loaded clip:', recording);
		} else {
			recording = recordingsStore.getSlippiRecording(recordingId);
			console.log('📹 Loaded recording:', recording);
		}

		if (!recording) {
			console.warn('⚠️ Recording not found:', recordingId, 'isClip:', isClip);
			events = [];
			return;
		}

		// Load Slippi events if available
		if (recording.slp_path) {
			isLoadingEvents = true;
			events = await recordingsStore.loadSlippiEvents(recording.slp_path);
			console.log('📊 Loaded', events.length, 'events');
			isLoadingEvents = false;
		} else {
			events = [];
			isLoadingEvents = false;
		}
	})();
});

function handleSeek(time: number) {
	playerRef?.seekTo(time);
}

function handleBack() {
	navigation.navigateBack();
}
</script>

<!-- Replay viewer needs fixed height to prevent scrolling -->
<div class="fixed inset-0 left-auto right-0 flex flex-col gap-3 overflow-hidden bg-background p-4" style="width: calc(100vw - var(--sidebar-width, 16rem)); top: 64px;">
	<!-- Header -->
	<div class="flex flex-shrink-0 items-center gap-4">
		<Button variant="ghost" size="sm" onclick={handleBack}>
			<ArrowLeft class="size-4" />
			Back
		</Button>
		<div class="flex flex-col">
			<h1 class="text-xl font-bold">
				{#if slippiMetadata}
					{slippiMetadata.players[0]?.player_tag || 'Player 1'} vs {slippiMetadata.players[1]?.player_tag || 'Player 2'}
				{:else if isClipOnly}
					Clip Viewer
				{:else}
					Replay Viewer
				{/if}
			</h1>
			{#if isClipOnly}
				<span class="text-sm text-muted-foreground">Raw video with no replay metadata</span>
			{/if}
		</div>
	</div>

	<!-- Main content -->
	<div class="grid flex-1 grid-cols-1 gap-3 overflow-hidden lg:grid-cols-[1fr_350px]">
		<!-- Left side: Video and Timeline -->
		<div class="flex flex-col gap-3 overflow-hidden">
			<!-- Video Player - fills available space -->
			<div class="flex flex-1 items-center justify-center overflow-hidden bg-black rounded-lg">
				{#if videoPath}
					<VideoPlayer
						bind:this={playerRef}
						videoPath={videoPath}
						oncurrenttimeupdate={(time) => (currentTime = time)}
						ondurationchange={(dur) => (duration = dur)}
					/>
				{:else}
					<div
						class="flex h-full items-center justify-center rounded-lg bg-muted text-muted-foreground"
					>
						No video available
					</div>
				{/if}
			</div>

			<!-- Timeline - fixed at bottom -->
			{#if !isClipOnly && duration > 0 && events.length > 0}
				<div class="flex-shrink-0">
					<Timeline {events} {duration} {currentTime} onseek={handleSeek} />
				</div>
			{:else if isClipOnly}
				<div class="flex-shrink-0 text-center text-sm text-muted-foreground">
					Timeline unavailable for raw clips
				</div>
			{:else if isLoadingEvents}
				<div class="flex-shrink-0 text-center text-sm text-muted-foreground">Loading timeline...</div>
			{/if}
		</div>

		<!-- Right side: Stats Panel -->
		{#if slippiMetadata}
			<div class="overflow-y-auto">
				<StatsPanel metadata={slippiMetadata} />
			</div>
		{/if}
	</div>
</div>

