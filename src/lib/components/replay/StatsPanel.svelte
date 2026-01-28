<script lang="ts">
	import type { SlippiMetadata } from '$lib/types/recording';
	import { getStageName } from '$lib/utils/characters';
	import CharacterIcon from '../recordings/CharacterIcon.svelte';
	import StageIcon from '../recordings/StageIcon.svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Separator } from '$lib/components/ui/separator';
	import { Crown } from '@lucide/svelte';

	let { metadata }: { metadata: SlippiMetadata | null } = $props();
</script>

<Card class="h-full">
	<CardHeader>
		<CardTitle>Match Stats</CardTitle>
	</CardHeader>
	<CardContent class="space-y-4">
		{#if metadata}
			<!-- Players -->
			<div class="space-y-3">
				{#each metadata.players as player}
					{@const isWinner = player.kill_count === 4}
					<div
						class="flex items-center gap-3 rounded-lg border p-3 {isWinner
							? 'border-green-500 bg-green-500/10'
							: 'border-border'}"
					>
						<CharacterIcon
							characterId={player.character_id}
							colorIndex={player.character_color}
							size="md"
						/>
						<div class="flex-1">
							<div class="font-semibold {isWinner ? 'text-green-600 dark:text-green-400' : ''}">
								{player.player_tag}
							</div>
							<div class="text-xs text-muted-foreground">Port {player.port}</div>
						</div>
						{#if isWinner}
							<Crown class="size-4 text-yellow-500 fill-yellow-500/30" />
						{/if}
					</div>
				{/each}
			</div>

			<Separator />

			<!-- Game Info -->
			<div class="space-y-2 text-sm">
				<div class="flex items-center justify-between">
					<span class="text-muted-foreground">Stage</span>
					<div class="flex items-center gap-2">
						<StageIcon stageId={metadata.stage} size="sm" />
						<span class="font-medium">{getStageName(metadata.stage)}</span>
					</div>
				</div>
				<div class="flex justify-between">
					<span class="text-muted-foreground">Duration</span>
					<span class="font-medium"
						>{Math.floor(metadata.game_duration / 60 / 60)}:{String(
							Math.floor((metadata.game_duration / 60) % 60)
						).padStart(2, '0')}</span
					>
				</div>
				<div class="flex justify-between">
					<span class="text-muted-foreground">Total Frames</span>
					<span class="font-medium">{metadata.total_frames}</span>
				</div>
				{#if metadata.played_on}
					<div class="flex justify-between">
						<span class="text-muted-foreground">Played On</span>
						<span class="font-medium capitalize">{metadata.played_on}</span>
					</div>
				{/if}
				<div class="flex justify-between">
					<span class="text-muted-foreground">Region</span>
					<span class="font-medium">{metadata.is_pal ? 'PAL' : 'NTSC'}</span>
				</div>
			</div>
		{:else}
			<div class="text-center text-sm text-muted-foreground">No match data available</div>
		{/if}
	</CardContent>
</Card>

