<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import Plyr from 'plyr';
	import 'plyr/dist/plyr.css';

	let {
		videoPath,
		oncurrenttimeupdate,
		ondurationchange,
	}: {
		videoPath: string;
		oncurrenttimeupdate?: (time: number) => void;
		ondurationchange?: (duration: number) => void;
	} = $props();

	// Convert Windows path to proper Tauri asset URL
	const videoSrc = $derived(convertFileSrc(videoPath));
	const DEFAULT_ASPECT_RATIO = 4 / 3;

	type PlayerDimensions = {
		width: number;
		height: number;
	};

	let videoElement: HTMLVideoElement;
	let containerElement: HTMLDivElement | null = null;
	let resizeObserver: ResizeObserver | null = null;
	let windowResizeHandler: (() => void) | null = null;
	let plyrInstance: Plyr | null = null;
	let currentTime = $state(0);
	let duration = $state(0);
	let aspectRatio = $state(DEFAULT_ASPECT_RATIO);
	let playerDimensions = $state<PlayerDimensions>({ width: 640, height: 480 });

	function updatePlayerDimensions() {
		if (!containerElement || aspectRatio <= 0) {
			return;
		}

		const rect = containerElement.getBoundingClientRect();
		const availableWidth = rect.width;
		const availableHeight = rect.height;

		if (!availableWidth || !availableHeight) {
			return;
		}

		let width = availableWidth;
		let height = width / aspectRatio;

		if (height > availableHeight) {
			height = availableHeight;
			width = height * aspectRatio;
		}

		playerDimensions = {
			width: Math.round(width),
			height: Math.round(height),
		};
	}

	onMount(() => {
		updatePlayerDimensions();

		if (containerElement && typeof ResizeObserver !== 'undefined') {
			resizeObserver = new ResizeObserver(() => updatePlayerDimensions());
			resizeObserver.observe(containerElement);
		} else if (typeof window !== 'undefined') {
			windowResizeHandler = () => updatePlayerDimensions();
			window.addEventListener('resize', windowResizeHandler);
		}

		if (videoElement) {
			console.log('🎬 Initializing video player with source:', videoSrc);

			// Initialize Plyr with aspect ratio preservation
			plyrInstance = new Plyr(videoElement, {
				controls: [
					'play-large',
					'play',
					'progress',
					'current-time',
					'duration',
					'mute',
					'volume',
					'settings',
					'fullscreen',
				],
				settings: ['speed', 'quality'],
				speed: { selected: 1, options: [0.25, 0.5, 0.75, 1, 1.25, 1.5, 2] },
				ratio: null, // Let video maintain its natural aspect ratio
				fullscreen: { enabled: true, fallback: true, iosNative: true, container: null },
			});

			// Listen for time updates
			plyrInstance.on('timeupdate', () => {
				if (plyrInstance) {
					currentTime = plyrInstance.currentTime;
					oncurrenttimeupdate?.(currentTime);
				}
			});

			// Listen for duration change
			plyrInstance.on('loadedmetadata', () => {
				if (plyrInstance) {
					const videoWidth = videoElement.videoWidth;
					const videoHeight = videoElement.videoHeight;

					if (videoWidth && videoHeight) {
						aspectRatio = videoWidth / videoHeight;
						updatePlayerDimensions();
					}

					duration = plyrInstance.duration;
					ondurationchange?.(duration);
					console.log('📊 Video duration:', duration);
				}
			});

			// Listen for errors
			plyrInstance.on('error', (error) => {
				console.error('❌ Video player error:', error);
			});
		}
	});

	onDestroy(() => {
		resizeObserver?.disconnect();

		if (windowResizeHandler && typeof window !== 'undefined') {
			window.removeEventListener('resize', windowResizeHandler);
		}

		plyrInstance?.destroy();
	});

	// Expose seek function for external control
	export function seekTo(time: number) {
		if (plyrInstance) {
			plyrInstance.currentTime = time;
		}
	}
</script>

<style>
	.player-container {
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.player-wrapper {
		display: flex;
		align-items: center;
		justify-content: center;
		max-width: 100%;
		max-height: 100%;
	}

	.player-wrapper :global(.plyr) {
		width: 100% !important;
		height: 100% !important;
	}

	.player-wrapper :global(.plyr__video-wrapper) {
		position: relative !important;
		width: 100% !important;
		height: 100% !important;
		padding-bottom: 0 !important;
		background-color: #000;
	}

	.player-wrapper :global(.plyr__video-wrapper video) {
		position: absolute !important;
		top: 0 !important;
		left: 0 !important;
		width: 100% !important;
		height: 100% !important;
		object-fit: contain !important;
		background-color: #000;
	}
</style>

<div class="player-container" bind:this={containerElement}>
	<div
		class="player-wrapper"
		style={`width: ${playerDimensions.width}px; height: ${playerDimensions.height}px;`}
	>
		<video bind:this={videoElement} playsinline>
			<source src={videoSrc} type="video/mp4" />
			Your browser does not support the video tag.
		</video>
	</div>
</div>
