<script lang="ts">
	/**
	 * EditorControls - Control panel for video editing operations
	 * 
	 * Provides controls for:
	 * - Entering/exiting edit mode
	 * - Crop controls (enable, reset)
	 * - Trim controls (set start/end points)
	 * - Apply changes / Cancel buttons
	 * - Two-step confirmation before processing
	 */
	import { Button } from '$lib/components/ui/button';
	import { 
		Scissors, 
		Crop, 
		RotateCcw, 
		Check, 
		X, 
		Play, 
		Pause,
		Film,
		Loader2,
		ArrowLeft,
		AlertTriangle
	} from '@lucide/svelte';

	export type TrimRange = {
		start: number | null;
		end: number | null;
	};

	export type CropRegion = {
		x: number;
		y: number;
		width: number;
		height: number;
	};

	let {
		editMode = false,
		cropEnabled = false,
		cropRegion = { x: 0, y: 0, width: 1, height: 1 },
		trimRange = { start: null, end: null },
		currentTime = 0,
		duration = 0,
		isProcessing = false,
		hasChanges = false,
		oneditmode,
		oncropenable,
		oncropreset,
		ontrimstart,
		ontrimend,
		ontrimclear,
		onapply,
		oncancel,
		oncreateclip,
	}: {
		editMode?: boolean;
		cropEnabled?: boolean;
		cropRegion?: CropRegion;
		trimRange?: TrimRange;
		currentTime?: number;
		duration?: number;
		isProcessing?: boolean;
		hasChanges?: boolean;
		oneditmode?: (enabled: boolean) => void;
		oncropenable?: (enabled: boolean) => void;
		oncropreset?: () => void;
		ontrimstart?: (time: number) => void;
		ontrimend?: (time: number) => void;
		ontrimclear?: () => void;
		onapply?: () => void;
		oncancel?: () => void;
		oncreateclip?: () => void;
	} = $props();

	// Confirmation step state
	let showConfirmation = $state(false);

	function formatTime(seconds: number): string {
		const mins = Math.floor(seconds / 60);
		const secs = Math.floor(seconds % 60);
		const ms = Math.floor((seconds % 1) * 100);
		return `${mins}:${String(secs).padStart(2, '0')}.${String(ms).padStart(2, '0')}`;
	}

	function formatPercent(value: number): string {
		return `${Math.round(value * 100)}%`;
	}

	const trimDuration = $derived.by(() => {
		if (trimRange.start !== null && trimRange.end !== null) {
			return trimRange.end - trimRange.start;
		}
		return null;
	});

	const canCreateClip = $derived(
		trimRange.start !== null && 
		trimRange.end !== null && 
		trimRange.end > trimRange.start
	);

	// Check if crop is actually different from full frame
	const hasCropChanges = $derived.by(() => {
		const epsilon = 0.001;
		return cropEnabled && (
			Math.abs(cropRegion.x) > epsilon || 
			Math.abs(cropRegion.y) > epsilon || 
			Math.abs(cropRegion.width - 1) > epsilon || 
			Math.abs(cropRegion.height - 1) > epsilon
		);
	});

	const hasTrimChanges = $derived(trimRange.start !== null || trimRange.end !== null);

	function handleReviewChanges() {
		showConfirmation = true;
	}

	function handleGoBack() {
		showConfirmation = false;
	}

	function handleConfirmApply() {
		onapply?.();
		// Don't reset showConfirmation here - let it stay visible during processing
		// It will reset when editMode changes (user exits edit mode after success)
	}

	// Reset confirmation when exiting edit mode
	$effect(() => {
		if (!editMode) {
			showConfirmation = false;
		}
	});
</script>

{#if !editMode}
	<!-- Simple edit button when not in edit mode -->
	<Button 
		variant="outline" 
		size="sm" 
		onclick={() => oneditmode?.(true)}
		class="gap-2"
	>
		<Scissors class="size-4" />
		Edit Video
	</Button>
{:else if isProcessing}
	<!-- Processing state - show progress -->
	<div class="flex flex-col gap-4 rounded-lg border border-border bg-card p-4">
		<div class="flex items-center gap-3">
			<Loader2 class="size-6 animate-spin text-primary" />
			<div>
				<h3 class="text-sm font-semibold text-foreground">Creating Clip</h3>
				<p class="text-xs text-muted-foreground">Please wait while your clip is being created...</p>
			</div>
		</div>
		
		<div class="space-y-2 text-xs">
			{#if hasCropChanges}
				<div class="flex items-center gap-2">
					<Crop class="size-3 text-muted-foreground" />
					<span>Cropping video...</span>
				</div>
			{/if}
			{#if hasTrimChanges}
				<div class="flex items-center gap-2">
					<Scissors class="size-3 text-muted-foreground" />
					<span>Trimming video...</span>
				</div>
			{/if}
		</div>
	</div>
{:else if showConfirmation}
	<!-- Confirmation step - review changes before applying -->
	<div class="flex flex-col gap-3 rounded-lg border border-primary/50 bg-card p-3">
		<!-- Header -->
		<div class="flex items-center gap-2">
			<Film class="size-5 text-primary" />
			<h3 class="text-sm font-semibold text-foreground">Create Clip</h3>
		</div>

		<p class="text-xs text-muted-foreground">
			A new clip will be created with the following edits (original video is preserved):
		</p>

		<!-- Summary of changes -->
		<div class="space-y-2 rounded bg-muted/50 p-2">
			{#if hasCropChanges}
				<div class="flex items-start gap-2 text-xs">
					<Crop class="size-3 mt-0.5 text-cyan-400" />
					<div>
						<span class="font-medium">Crop</span>
						<div class="text-muted-foreground">
							Position: {formatPercent(cropRegion.x)}, {formatPercent(cropRegion.y)}<br/>
							Size: {formatPercent(cropRegion.width)} × {formatPercent(cropRegion.height)}
						</div>
					</div>
				</div>
			{/if}
			
			{#if hasTrimChanges}
				<div class="flex items-start gap-2 text-xs">
					<Scissors class="size-3 mt-0.5 text-cyan-400" />
					<div>
						<span class="font-medium">Trim</span>
						<div class="text-muted-foreground">
							{#if trimRange.start !== null && trimRange.end !== null}
								{formatTime(trimRange.start)} → {formatTime(trimRange.end)}
								<span class="text-primary">({formatTime(trimDuration ?? 0)})</span>
							{:else if trimRange.start !== null}
								From {formatTime(trimRange.start)} to end
							{:else if trimRange.end !== null}
								From start to {formatTime(trimRange.end)}
							{/if}
						</div>
					</div>
				</div>
			{/if}
		</div>

		<p class="text-xs text-primary/80">
			The clip will be saved to your Clips folder.
		</p>

		<!-- Action Buttons -->
		<div class="flex gap-2">
			<Button
				variant="outline"
				size="sm"
				onclick={handleGoBack}
				class="flex-1 gap-1"
			>
				<ArrowLeft class="size-4" />
				Go Back
			</Button>
			<Button
				variant="default"
				size="sm"
				onclick={handleConfirmApply}
				class="flex-1 gap-1"
			>
				<Film class="size-4" />
				Create Clip
			</Button>
		</div>
	</div>
{:else}
	<!-- Full editor controls panel -->
	<div class="flex flex-col gap-3 rounded-lg border border-border bg-card p-3">
		<!-- Header -->
		<div class="flex items-center justify-between">
			<h3 class="text-sm font-semibold text-foreground">Video Editor</h3>
			{#if oncancel}
				<Button
					variant="ghost"
					size="icon"
					onclick={() => oncancel?.()}
					class="h-6 w-6"
				>
					<X class="size-4" />
				</Button>
			{/if}
		</div>

		<!-- Crop Section -->
		<div class="space-y-2">
			<div class="flex items-center justify-between">
				<span class="text-xs font-medium text-muted-foreground">Crop</span>
				<div class="flex gap-1">
					<Button
						variant={cropEnabled ? 'default' : 'outline'}
						size="sm"
						onclick={() => oncropenable?.(!cropEnabled)}
						class="h-7 gap-1.5 px-2 text-xs"
					>
						<Crop class="size-3" />
						{cropEnabled ? 'Enabled' : 'Enable'}
					</Button>
					{#if cropEnabled}
						<Button
							variant="ghost"
							size="sm"
							onclick={() => oncropreset?.()}
							class="h-7 gap-1 px-2 text-xs"
						>
							<RotateCcw class="size-3" />
							Reset
						</Button>
					{/if}
				</div>
			</div>

		</div>

		<!-- Trim Section -->
		<div class="space-y-2">
			<div class="flex items-center justify-between">
				<span class="text-xs font-medium text-muted-foreground">Trim</span>
				<span class="text-xs text-muted-foreground">
					Current: {formatTime(currentTime)}
				</span>
			</div>
			
			<div class="flex gap-1">
				<Button
					variant="outline"
					size="sm"
					onclick={() => ontrimstart?.(currentTime)}
					class="h-7 flex-1 gap-1 px-2 text-xs"
				>
					<Play class="size-3" />
					Set Start
				</Button>
				<Button
					variant="outline"
					size="sm"
					onclick={() => ontrimend?.(currentTime)}
					class="h-7 flex-1 gap-1 px-2 text-xs"
				>
					<Pause class="size-3" />
					Set End
				</Button>
				{#if trimRange.start !== null || trimRange.end !== null}
					<Button
						variant="ghost"
						size="sm"
						onclick={() => ontrimclear?.()}
						class="h-7 gap-1 px-2 text-xs"
					>
						<X class="size-3" />
					</Button>
				{/if}
			</div>

			<!-- Trim range display -->
			{#if trimRange.start !== null || trimRange.end !== null}
				<div class="rounded bg-muted/50 px-2 py-1.5 text-xs">
					<div class="flex justify-between">
						<span class="text-muted-foreground">Start:</span>
						<span class="font-mono">
							{trimRange.start !== null ? formatTime(trimRange.start) : '--:--'}
						</span>
					</div>
					<div class="flex justify-between">
						<span class="text-muted-foreground">End:</span>
						<span class="font-mono">
							{trimRange.end !== null ? formatTime(trimRange.end) : '--:--'}
						</span>
					</div>
					{#if trimDuration !== null}
						<div class="mt-1 flex justify-between border-t border-border/50 pt-1">
							<span class="text-muted-foreground">Duration:</span>
							<span class="font-mono font-medium text-primary">
								{formatTime(trimDuration)}
							</span>
						</div>
					{/if}
				</div>
			{/if}
		</div>

		<!-- Create Clip Button -->
		{#if canCreateClip}
			<Button
				variant="secondary"
				size="sm"
				onclick={() => oncreateclip?.()}
				class="gap-2"
			>
				<Film class="size-4" />
				Create Clip from Selection
			</Button>
		{/if}

		<!-- Action Buttons -->
		<div class="flex gap-2 border-t border-border pt-3">
			{#if oncancel}
				<Button
					variant="ghost"
					size="sm"
					onclick={() => oncancel?.()}
					class="flex-1 gap-1"
				>
					<X class="size-4" />
					Cancel
				</Button>
			{/if}
			<Button
				variant="default"
				size="sm"
				onclick={handleReviewChanges}
				disabled={!hasChanges}
				class={oncancel ? "flex-1 gap-1" : "w-full gap-1"}
			>
				<Check class="size-4" />
				Review Changes
			</Button>
		</div>
	</div>
{/if}
