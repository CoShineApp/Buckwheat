<script lang="ts">
	import { CharacterId } from "$lib/types/recording";
	import { getCharacterName, getCharacterImage } from "$lib/utils/characters";
	import { Tooltip, TooltipContent, TooltipTrigger } from "$lib/components/ui/tooltip";
	import { Crown } from "@lucide/svelte";

	interface Props {
		characterId: CharacterId | number;
		size?: "sm" | "md" | "lg" | "xl";
		colorIndex?: number;
		isWinner?: boolean;
	}

	let { characterId, size = "md", colorIndex = 0, isWinner = false }: Props = $props();

	const characterName = $derived(getCharacterName(characterId));
	const imageUrl = $derived(getCharacterImage(characterId));

	// Size mappings
	const sizeClasses = {
		sm: "size-8",
		md: "size-12",
		lg: "size-16",
		xl: "size-20",
	};

	const sizeClass = $derived(sizeClasses[size]);

	// Calculate crop position based on color index (each character has 4 colors horizontally)
	const getCropStyle = (colorIdx: number) => {
		// SSBWiki palette images show all 4 color variants horizontally
		// Each variant is 25% of the width
		const xOffset = (colorIdx % 4) * 25;
		return {
			objectPosition: `-${xOffset}% 0`,
			width: '400%', // Show 1/4 of the image
		};
	};

	let imageLoaded = $state(false);
	let imageError = $state(false);
</script>

<Tooltip>
	<TooltipTrigger>
		<!-- Outer wrapper for crown positioning (no overflow-hidden so crown can extend outside) -->
		<div class="relative">
			<div class={`overflow-hidden rounded-md border border-border bg-muted ${sizeClass}`}>
				{#if !imageError && imageUrl}
					<img
						src={imageUrl}
						alt={characterName}
						class="h-full object-cover object-left"
						style="object-position: {getCropStyle(colorIndex).objectPosition}; width: {getCropStyle(colorIndex).width};"
						onload={() => {
							imageLoaded = true;
						}}
						onerror={() => {
							imageError = true;
						}}
						class:opacity-0={!imageLoaded}
						class:opacity-100={imageLoaded}
					/>
				{:else}
					<div class="flex h-full items-center justify-center bg-gradient-to-br from-primary/20 to-primary/5 text-xs font-semibold text-primary">
						{characterName.slice(0, 2).toUpperCase()}
					</div>
				{/if}
			</div>
			<!-- Winner crown overlay - positioned outside overflow-hidden container -->
			{#if isWinner}
				<div class="absolute -top-1 -right-1 rounded-full bg-yellow-500 p-0.5 shadow-md border border-yellow-600">
					<Crown class="size-3 text-white fill-white" />
				</div>
			{/if}
		</div>
	</TooltipTrigger>
	<TooltipContent>
		<p>{characterName}{isWinner ? ' (Winner)' : ''}</p>
	</TooltipContent>
</Tooltip>

