<script lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { navigation } from '$lib/stores/navigation.svelte';
import { recordingsStore } from '$lib/stores/recordings.svelte';
import type { ClipSession } from '$lib/stores/clips.svelte';
import type { RecordingWithMetadata } from '$lib/types/recording';
import VideoPlayer from './VideoPlayer.svelte';
import Timeline, { type TrimRange } from './Timeline.svelte';
import StatsPanel from './StatsPanel.svelte';
import CropOverlay, { type CropRegion } from './CropOverlay.svelte';
import EditorControls from './EditorControls.svelte';
import { Button } from '$lib/components/ui/button';
import { ArrowLeft } from '@lucide/svelte';

let { recordingId, isClip }: { recordingId: string; isClip?: boolean } = $props();

let playerRef: VideoPlayer;
let cropOverlayRef: CropOverlay;
let recording = $state<ClipSession | RecordingWithMetadata | undefined>(undefined);
let currentTime = $state(0);
let duration = $state(0);

// Player dimensions for crop overlay positioning
let playerDimensions = $state<{ width: number; height: number }>({ width: 640, height: 480 });

// Edit mode state - always enabled for clips, toggleable for recordings
let editMode = $state(false);
let cropEnabled = $state(false);
let cropRegion = $state<CropRegion>({ x: 0, y: 0, width: 1, height: 1 });
let trimRange = $state<TrimRange>({ start: null, end: null });
let isProcessing = $state(false);

// For clips, always show editor (no "Edit Video" button needed)
const alwaysShowEditor = $derived(isClip === true);

const isClipOnly = $derived(recordingsStore.isClipOnly(recording));
const slippiMetadata = $derived(recording?.slippi_metadata ?? null);
const videoPath = $derived(recording?.video_path ?? null);

// Helper for floating point comparison (avoid precision issues)
function isNearlyEqual(a: number, b: number, epsilon = 0.001): boolean {
	return Math.abs(a - b) < epsilon;
}

// Check if there are pending changes
const hasChanges = $derived.by(() => {
	// Check if crop is different from full frame (using epsilon for float comparison)
	const hasCropChanges = cropEnabled && (
		!isNearlyEqual(cropRegion.x, 0) || 
		!isNearlyEqual(cropRegion.y, 0) || 
		!isNearlyEqual(cropRegion.width, 1) || 
		!isNearlyEqual(cropRegion.height, 1)
	);
	
	// Check if trim points are set
	const hasTrimChanges = trimRange.start !== null || trimRange.end !== null;
	
	return hasCropChanges || hasTrimChanges;
});

// Reactively load recording when recordingId or isClip changes
$effect(() => {
	if (!recordingId) {
		recording = undefined;
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
			return;
		}
	})();
});

function handleSeek(time: number) {
	playerRef?.seekTo(time);
}

function handleBack() {
	navigation.navigateBack();
}

// Edit mode handlers
function handleEditModeChange(enabled: boolean) {
	editMode = enabled;
	if (!enabled) {
		// Reset all edit state when exiting edit mode
		cropEnabled = false;
		cropRegion = { x: 0, y: 0, width: 1, height: 1 };
		trimRange = { start: null, end: null };
	}
}

function handleCropEnableChange(enabled: boolean) {
	cropEnabled = enabled;
	if (!enabled) {
		// Reset crop region when disabling
		cropRegion = { x: 0, y: 0, width: 1, height: 1 };
	}
}

function handleCropReset() {
	cropRegion = { x: 0, y: 0, width: 1, height: 1 };
	cropOverlayRef?.reset();
}

function handleCropRegionChange(region: CropRegion) {
	cropRegion = region;
}

function handleTrimStart(time: number) {
	trimRange = { ...trimRange, start: time };
}

function handleTrimEnd(time: number) {
	trimRange = { ...trimRange, end: time };
}

function handleTrimClear() {
	trimRange = { start: null, end: null };
}

function handleTrimChange(range: TrimRange) {
	trimRange = range;
}

async function handleApplyChanges() {
	if (!videoPath || !hasChanges) return;
	
	isProcessing = true;
	
	try {
		// Get video dimensions for crop calculation
		const videoDims = playerRef?.getVideoDimensions();
		
		// Prepare crop parameters (convert normalized to pixels)
		let cropX: number | null = null;
		let cropY: number | null = null;
		let cropWidth: number | null = null;
		let cropHeight: number | null = null;
		
		if (cropEnabled && videoDims) {
			const pixelCrop = cropOverlayRef?.getPixelCrop(videoDims.width, videoDims.height);
			if (pixelCrop) {
				cropX = pixelCrop.x;
				cropY = pixelCrop.y;
				cropWidth = pixelCrop.width;
				cropHeight = pixelCrop.height;
			}
		}
		
		// Call backend to create clip with edits
		const clipPath = await invoke<string>('apply_video_edit', {
			inputPath: videoPath,
			trimStart: trimRange.start,
			trimEnd: trimRange.end,
			cropX,
			cropY,
			cropWidth,
			cropHeight,
			replaceOriginal: false, // Always create a new clip
		});
		
		console.log('✅ Clip created:', clipPath);
		
		// Reset edit mode
		handleEditModeChange(false);
		
		// Navigate to clips tab to see the new clip
		navigation.navigateToClips();
		
	} catch (error) {
		console.error('❌ Failed to create clip:', error);
	} finally {
		isProcessing = false;
	}
}

function handleCancelEdit() {
	handleEditModeChange(false);
}

function handlePlayerDimensionsChange(dims: { width: number; height: number }) {
	playerDimensions = dims;
}

async function handleCreateClip() {
	if (!videoPath || trimRange.start === null || trimRange.end === null) return;
	
	isProcessing = true;
	
	try {
		const result = await invoke<string>('create_clip_from_range', {
			inputPath: videoPath,
			startTime: trimRange.start,
			endTime: trimRange.end,
			outputDir: null, // Use default clips directory
		});
		
		console.log('✅ Clip created:', result);
		
		// Reset trim state but keep edit mode open
		handleTrimClear();
		
	} catch (error) {
		console.error('❌ Failed to create clip:', error);
	} finally {
		isProcessing = false;
	}
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
		<div class="flex flex-1 flex-col">
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
			<!-- Video Player Container - fills available space -->
			<div class="relative flex flex-1 items-center justify-center overflow-hidden bg-black rounded-lg">
				{#if videoPath}
					<!-- VideoPlayer fills available space naturally -->
					<VideoPlayer
						bind:this={playerRef}
						videoPath={videoPath}
						oncurrenttimeupdate={(time) => (currentTime = time)}
						ondurationchange={(dur) => (duration = dur)}
						onplayerdimensionschange={handlePlayerDimensionsChange}
					/>
					<!-- Crop overlay positioned absolutely, centered, matching video dimensions -->
					<!-- Only render when crop is enabled to avoid blocking video player interactions -->
					{#if cropEnabled && !isProcessing}
						<div 
							class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2"
							style="width: {playerDimensions.width}px; height: {playerDimensions.height}px;"
						>
							<CropOverlay
								bind:this={cropOverlayRef}
								enabled={true}
								region={cropRegion}
								onregionchange={handleCropRegionChange}
							/>
						</div>
					{/if}
				{:else}
					<div
						class="flex h-full items-center justify-center rounded-lg bg-muted text-muted-foreground"
					>
						No video available
					</div>
				{/if}
			</div>

			<!-- Timeline - fixed at bottom -->
			{#if duration > 0}
				<div class="flex-shrink-0">
					<Timeline 
						events={[]}
						{duration} 
						{currentTime} 
						onseek={handleSeek}
						editMode={true}
						{trimRange}
						ontrimchange={handleTrimChange}
					/>
				</div>
			{/if}
		</div>

		<!-- Right side: Stats Panel at top, Editor Controls at bottom -->
		<div class="flex flex-col gap-3 overflow-y-auto">
			<!-- Match Stats (if available) -->
			{#if slippiMetadata}
				<StatsPanel metadata={slippiMetadata} />
			{/if}
			
			<!-- Spacer to push editor to bottom -->
			<div class="flex-1"></div>
			
			<!-- Video Editor Controls (always visible) -->
			<EditorControls
				editMode={true}
				{cropEnabled}
				{cropRegion}
				{trimRange}
				{currentTime}
				{duration}
				{isProcessing}
				{hasChanges}
				oneditmode={handleEditModeChange}
				oncropenable={handleCropEnableChange}
				oncropreset={handleCropReset}
				ontrimstart={handleTrimStart}
				ontrimend={handleTrimEnd}
				ontrimclear={handleTrimClear}
				onapply={handleApplyChanges}
				oncancel={undefined}
				oncreateclip={handleCreateClip}
			/>
		</div>
	</div>
</div>

