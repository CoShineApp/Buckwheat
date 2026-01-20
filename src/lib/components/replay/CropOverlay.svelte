<script lang="ts">
	/**
	 * CropOverlay - A draggable crop region overlay for video editing
	 * 
	 * Renders a resizable crop box on top of the video with:
	 * - 8 resize handles (corners + edges)
	 * - Semi-transparent mask outside the crop region
	 * - Outputs normalized coordinates (0-1 range) for any video resolution
	 */

	export type CropRegion = {
		x: number;      // Normalized 0-1 left offset
		y: number;      // Normalized 0-1 top offset
		width: number;  // Normalized 0-1 width
		height: number; // Normalized 0-1 height
	};

	let {
		enabled = false,
		region = { x: 0, y: 0, width: 1, height: 1 },
		onregionchange,
		minSize = 0.1, // Minimum 10% of video dimensions
	}: {
		enabled?: boolean;
		region?: CropRegion;
		onregionchange?: (region: CropRegion) => void;
		minSize?: number;
	} = $props();

	let containerRef: HTMLDivElement | null = null;
	let isDragging = $state(false);
	let dragType = $state<'move' | 'n' | 's' | 'e' | 'w' | 'nw' | 'ne' | 'sw' | 'se' | null>(null);
	let dragStart = $state({ x: 0, y: 0 });
	let regionStart = $state<CropRegion>({ x: 0, y: 0, width: 1, height: 1 });

	// Calculate pixel positions from normalized region
	const cropStyle = $derived.by(() => {
		return {
			left: `${region.x * 100}%`,
			top: `${region.y * 100}%`,
			width: `${region.width * 100}%`,
			height: `${region.height * 100}%`,
		};
	});

	function clamp(value: number, min: number, max: number): number {
		return Math.min(Math.max(value, min), max);
	}

	function getNormalizedPosition(e: MouseEvent): { x: number; y: number } {
		if (!containerRef) return { x: 0, y: 0 };
		const rect = containerRef.getBoundingClientRect();
		return {
			x: clamp((e.clientX - rect.left) / rect.width, 0, 1),
			y: clamp((e.clientY - rect.top) / rect.height, 0, 1),
		};
	}

	function handleMouseDown(e: MouseEvent, type: typeof dragType) {
		if (!enabled) return;
		e.preventDefault();
		e.stopPropagation();
		
		isDragging = true;
		dragType = type;
		dragStart = getNormalizedPosition(e);
		regionStart = { ...region };
		
		window.addEventListener('mousemove', handleMouseMove);
		window.addEventListener('mouseup', handleMouseUp);
	}

	function handleMouseMove(e: MouseEvent) {
		if (!isDragging || !dragType) return;
		
		const current = getNormalizedPosition(e);
		const deltaX = current.x - dragStart.x;
		const deltaY = current.y - dragStart.y;
		
		let newRegion = { ...regionStart };
		
		if (dragType === 'move') {
			// Move entire region
			newRegion.x = clamp(regionStart.x + deltaX, 0, 1 - regionStart.width);
			newRegion.y = clamp(regionStart.y + deltaY, 0, 1 - regionStart.height);
		} else {
			// Resize operations
			if (dragType.includes('w')) {
				const newX = clamp(regionStart.x + deltaX, 0, regionStart.x + regionStart.width - minSize);
				newRegion.width = regionStart.width - (newX - regionStart.x);
				newRegion.x = newX;
			}
			if (dragType.includes('e')) {
				newRegion.width = clamp(regionStart.width + deltaX, minSize, 1 - regionStart.x);
			}
			if (dragType.includes('n')) {
				const newY = clamp(regionStart.y + deltaY, 0, regionStart.y + regionStart.height - minSize);
				newRegion.height = regionStart.height - (newY - regionStart.y);
				newRegion.y = newY;
			}
			if (dragType.includes('s')) {
				newRegion.height = clamp(regionStart.height + deltaY, minSize, 1 - regionStart.y);
			}
		}
		
		region = newRegion;
		onregionchange?.(newRegion);
	}

	function handleMouseUp() {
		isDragging = false;
		dragType = null;
		window.removeEventListener('mousemove', handleMouseMove);
		window.removeEventListener('mouseup', handleMouseUp);
	}

	// Reset region to full size
	export function reset() {
		region = { x: 0, y: 0, width: 1, height: 1 };
		onregionchange?.(region);
	}

	// Get pixel-based crop values for a given video resolution
	export function getPixelCrop(videoWidth: number, videoHeight: number) {
		return {
			x: Math.round(region.x * videoWidth),
			y: Math.round(region.y * videoHeight),
			width: Math.round(region.width * videoWidth),
			height: Math.round(region.height * videoHeight),
		};
	}
</script>

{#if enabled}
	<div 
		class="absolute inset-0 z-10"
		bind:this={containerRef}
	>
		<!-- Dark overlay masks (4 rectangles around the crop area) -->
		<!-- Top mask -->
		<div 
			class="absolute left-0 right-0 top-0 bg-black/60 pointer-events-none"
			style="height: {region.y * 100}%"
		></div>
		<!-- Bottom mask -->
		<div 
			class="absolute bottom-0 left-0 right-0 bg-black/60 pointer-events-none"
			style="height: {(1 - region.y - region.height) * 100}%"
		></div>
		<!-- Left mask -->
		<div 
			class="absolute left-0 bg-black/60 pointer-events-none"
			style="top: {region.y * 100}%; height: {region.height * 100}%; width: {region.x * 100}%"
		></div>
		<!-- Right mask -->
		<div 
			class="absolute right-0 bg-black/60 pointer-events-none"
			style="top: {region.y * 100}%; height: {region.height * 100}%; width: {(1 - region.x - region.width) * 100}%"
		></div>

		<!-- Crop area -->
		<div
			class="absolute border-2 border-cyan-400 cursor-move"
			style="left: {cropStyle.left}; top: {cropStyle.top}; width: {cropStyle.width}; height: {cropStyle.height}"
			onmousedown={(e) => handleMouseDown(e, 'move')}
			role="application"
			aria-label="Crop region - drag to move"
		>
			<!-- Grid lines for visual guidance -->
			<div class="absolute inset-0 pointer-events-none">
				<div class="absolute left-1/3 top-0 bottom-0 w-px bg-cyan-400/40"></div>
				<div class="absolute left-2/3 top-0 bottom-0 w-px bg-cyan-400/40"></div>
				<div class="absolute top-1/3 left-0 right-0 h-px bg-cyan-400/40"></div>
				<div class="absolute top-2/3 left-0 right-0 h-px bg-cyan-400/40"></div>
			</div>

			<!-- Corner handles -->
			<div
				class="absolute -left-1.5 -top-1.5 h-3 w-3 cursor-nw-resize border-2 border-cyan-400 bg-background"
				onmousedown={(e) => handleMouseDown(e, 'nw')}
				role="slider"
				aria-label="Resize top-left corner"
			></div>
			<div
				class="absolute -right-1.5 -top-1.5 h-3 w-3 cursor-ne-resize border-2 border-cyan-400 bg-background"
				onmousedown={(e) => handleMouseDown(e, 'ne')}
				role="slider"
				aria-label="Resize top-right corner"
			></div>
			<div
				class="absolute -bottom-1.5 -left-1.5 h-3 w-3 cursor-sw-resize border-2 border-cyan-400 bg-background"
				onmousedown={(e) => handleMouseDown(e, 'sw')}
				role="slider"
				aria-label="Resize bottom-left corner"
			></div>
			<div
				class="absolute -bottom-1.5 -right-1.5 h-3 w-3 cursor-se-resize border-2 border-cyan-400 bg-background"
				onmousedown={(e) => handleMouseDown(e, 'se')}
				role="slider"
				aria-label="Resize bottom-right corner"
			></div>

			<!-- Edge handles -->
			<div
				class="absolute -top-1 left-1/2 h-2 w-6 -translate-x-1/2 cursor-n-resize border-2 border-cyan-400 bg-background"
				onmousedown={(e) => handleMouseDown(e, 'n')}
				role="slider"
				aria-label="Resize top edge"
			></div>
			<div
				class="absolute -bottom-1 left-1/2 h-2 w-6 -translate-x-1/2 cursor-s-resize border-2 border-cyan-400 bg-background"
				onmousedown={(e) => handleMouseDown(e, 's')}
				role="slider"
				aria-label="Resize bottom edge"
			></div>
			<div
				class="absolute -left-1 top-1/2 h-6 w-2 -translate-y-1/2 cursor-w-resize border-2 border-cyan-400 bg-background"
				onmousedown={(e) => handleMouseDown(e, 'w')}
				role="slider"
				aria-label="Resize left edge"
			></div>
			<div
				class="absolute -right-1 top-1/2 h-6 w-2 -translate-y-1/2 cursor-e-resize border-2 border-cyan-400 bg-background"
				onmousedown={(e) => handleMouseDown(e, 'e')}
				role="slider"
				aria-label="Resize right edge"
			></div>
		</div>
	</div>
{/if}
