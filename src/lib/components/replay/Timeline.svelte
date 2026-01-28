<script lang="ts">
	import type { GameEvent } from '$lib/types/recording';
	import TimelineEvent from './TimelineEvent.svelte';

	export type TrimRange = {
		start: number | null;
		end: number | null;
	};

	let {
		events = [],
		duration,
		currentTime = 0,
		onseek,
		editMode = false,
		trimRange = { start: null, end: null },
		ontrimchange,
	}: {
		events: GameEvent[];
		duration: number;
		currentTime?: number;
		onseek?: (time: number) => void;
		editMode?: boolean;
		trimRange?: TrimRange;
		ontrimchange?: (range: TrimRange) => void;
	} = $props();

	let timelineRef: HTMLDivElement | null = null;
	let isDraggingTrim = $state<'start' | 'end' | null>(null);

	// Calculate progress percentage
	const progress = $derived((currentTime / duration) * 100);

	// Calculate trim positions as percentages
	const trimStartPercent = $derived(
		trimRange.start !== null ? (trimRange.start / duration) * 100 : null
	);
	const trimEndPercent = $derived(
		trimRange.end !== null ? (trimRange.end / duration) * 100 : null
	);

	// Handle timeline click to seek
	function handleTimelineClick(e: MouseEvent) {
		if (isDraggingTrim) return;
		const timeline = e.currentTarget as HTMLDivElement;
		const rect = timeline.getBoundingClientRect();
		const clickX = e.clientX - rect.left;
		const percentage = clickX / rect.width;
		const seekTime = percentage * duration;
		onseek?.(seekTime);
	}

	function getTimeFromMouseEvent(e: MouseEvent): number {
		if (!timelineRef) return 0;
		const rect = timelineRef.getBoundingClientRect();
		const percentage = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
		return percentage * duration;
	}

	function handleTrimHandleMouseDown(e: MouseEvent, handle: 'start' | 'end') {
		if (!editMode) return;
		e.preventDefault();
		e.stopPropagation();
		isDraggingTrim = handle;
		
		window.addEventListener('mousemove', handleTrimMouseMove);
		window.addEventListener('mouseup', handleTrimMouseUp);
	}

	function handleTrimMouseMove(e: MouseEvent) {
		if (!isDraggingTrim) return;
		
		const time = getTimeFromMouseEvent(e);
		const newRange = { ...trimRange };
		
		if (isDraggingTrim === 'start') {
			// Ensure start doesn't go past end
			const maxStart = trimRange.end !== null ? trimRange.end - 0.1 : duration;
			newRange.start = Math.max(0, Math.min(time, maxStart));
		} else {
			// Ensure end doesn't go before start
			const minEnd = trimRange.start !== null ? trimRange.start + 0.1 : 0;
			newRange.end = Math.min(duration, Math.max(time, minEnd));
		}
		
		ontrimchange?.(newRange);
	}

	function handleTrimMouseUp() {
		isDraggingTrim = null;
		window.removeEventListener('mousemove', handleTrimMouseMove);
		window.removeEventListener('mouseup', handleTrimMouseUp);
	}
</script>

<div class="w-full space-y-2">
	<div class="text-xs text-muted-foreground">
		{Math.floor(currentTime / 60)}:{String(Math.floor(currentTime % 60)).padStart(2, '0')} / {Math.floor(
			duration / 60
		)}:{String(Math.floor(duration % 60)).padStart(2, '0')}
	</div>

	<!-- Timeline bar -->
	<div
		bind:this={timelineRef}
		class="relative h-8 w-full cursor-pointer rounded-md bg-muted"
		onclick={handleTimelineClick}
		role="progressbar"
		aria-valuenow={currentTime}
		aria-valuemin={0}
		aria-valuemax={duration}
	>
		<!-- Trim region highlight (shown when in edit mode with trim points set) -->
		{#if editMode && trimStartPercent !== null && trimEndPercent !== null}
			<div 
				class="absolute inset-y-0 bg-cyan-500/30 pointer-events-none"
				style="left: {trimStartPercent}%; width: {trimEndPercent - trimStartPercent}%"
			></div>
		{/if}

		<!-- Progress indicator -->
		<div class="absolute inset-0 rounded-md bg-primary/20" style="width: {progress}%"></div>

		<!-- Current time indicator -->
		<div
			class="absolute top-0 h-full w-0.5 bg-primary"
			style="left: {progress}%"
		></div>

		<!-- Event markers -->
		{#each events as event (event.frame)}
			<TimelineEvent {event} {duration} onclick={onseek} />
		{/each}

		<!-- Trim handles (shown when in edit mode) -->
		{#if editMode}
			<!-- Start handle -->
			{#if trimStartPercent !== null}
				<div
					class="absolute top-0 h-full w-1 cursor-ew-resize bg-cyan-400 hover:bg-cyan-300 transition-colors z-10"
					style="left: {trimStartPercent}%"
					onmousedown={(e) => handleTrimHandleMouseDown(e, 'start')}
					role="slider"
					aria-label="Trim start handle"
					aria-valuenow={trimRange.start ?? 0}
				>
					<!-- Handle grip -->
					<div class="absolute -left-1.5 top-1/2 -translate-y-1/2 h-5 w-4 rounded bg-cyan-400 border border-cyan-300 flex items-center justify-center">
						<div class="w-0.5 h-3 bg-cyan-700/50 rounded"></div>
					</div>
				</div>
			{/if}
			
			<!-- End handle -->
			{#if trimEndPercent !== null}
				<div
					class="absolute top-0 h-full w-1 cursor-ew-resize bg-cyan-400 hover:bg-cyan-300 transition-colors z-10"
					style="left: {trimEndPercent}%"
					onmousedown={(e) => handleTrimHandleMouseDown(e, 'end')}
					role="slider"
					aria-label="Trim end handle"
					aria-valuenow={trimRange.end ?? duration}
				>
					<!-- Handle grip -->
					<div class="absolute -left-1.5 top-1/2 -translate-y-1/2 h-5 w-4 rounded bg-cyan-400 border border-cyan-300 flex items-center justify-center">
						<div class="w-0.5 h-3 bg-cyan-700/50 rounded"></div>
					</div>
				</div>
			{/if}
		{/if}
	</div>

	<!-- Legend -->
	<div class="flex items-center gap-4 text-xs text-muted-foreground">
		<div class="flex items-center gap-1.5">
			<div class="h-2 w-2 rounded-full bg-red-500"></div>
			<span>Death</span>
		</div>
		{#if editMode && (trimStartPercent !== null || trimEndPercent !== null)}
			<div class="flex items-center gap-1.5">
				<div class="h-2 w-2 rounded bg-cyan-400"></div>
				<span>Trim Range</span>
			</div>
		{/if}
	</div>
</div>

