<script lang="ts">
	import type { SlippiMetadata } from '$lib/types/recording';
	import { getStageName } from '$lib/utils/characters';
	import CharacterIcon from '../recordings/CharacterIcon.svelte';
	import StageIcon from '../recordings/StageIcon.svelte';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Separator } from '$lib/components/ui/separator';
	import { Crown, ChevronDown, ChevronUp } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';

	/**
	 * Player stats from the database (uses camelCase from Rust/Serde)
	 */
	interface PlayerStats {
		playerIndex: number;
		connectCode: string | null;
		displayName: string | null;
		characterId: number;
		port: number;
		lCancelSuccessCount: number;
		lCancelFailCount: number;
		// Detailed L-cancel breakdown
		lCancelNairShieldSuccess: number;
		lCancelNairShieldFail: number;
		lCancelNairWhiffSuccess: number;
		lCancelNairWhiffFail: number;
		lCancelNairHitSuccess: number;
		lCancelNairHitFail: number;
		lCancelFairShieldSuccess: number;
		lCancelFairShieldFail: number;
		lCancelFairWhiffSuccess: number;
		lCancelFairWhiffFail: number;
		lCancelFairHitSuccess: number;
		lCancelFairHitFail: number;
		lCancelBairShieldSuccess: number;
		lCancelBairShieldFail: number;
		lCancelBairWhiffSuccess: number;
		lCancelBairWhiffFail: number;
		lCancelBairHitSuccess: number;
		lCancelBairHitFail: number;
		lCancelUairShieldSuccess: number;
		lCancelUairShieldFail: number;
		lCancelUairWhiffSuccess: number;
		lCancelUairWhiffFail: number;
		lCancelUairHitSuccess: number;
		lCancelUairHitFail: number;
		lCancelDairShieldSuccess: number;
		lCancelDairShieldFail: number;
		lCancelDairWhiffSuccess: number;
		lCancelDairWhiffFail: number;
		lCancelDairHitSuccess: number;
		lCancelDairHitFail: number;
		shieldGrabCount: number;
	}

	let { metadata, playerStats = [] }: { metadata: SlippiMetadata | null; playerStats?: PlayerStats[] } = $props();
	
	// Toggle state for L-cancel breakdown
	let showLCancelBreakdown = $state(false);
	
	// Helper to calculate L-cancel percentage
	function getLCancelPercent(success: number, fail: number): string {
		const total = success + fail;
		if (total === 0) return '--';
		return `${Math.round((success / total) * 100)}%`;
	}
	
	// Helper to get aerial totals for a player
	function getAerialStats(player: PlayerStats, aerial: 'nair' | 'fair' | 'bair' | 'uair' | 'dair') {
		const key = aerial.charAt(0).toUpperCase() + aerial.slice(1);
		const shieldSuccess = player[`lCancel${key}ShieldSuccess` as keyof PlayerStats] as number;
		const shieldFail = player[`lCancel${key}ShieldFail` as keyof PlayerStats] as number;
		const whiffSuccess = player[`lCancel${key}WhiffSuccess` as keyof PlayerStats] as number;
		const whiffFail = player[`lCancel${key}WhiffFail` as keyof PlayerStats] as number;
		const hitSuccess = player[`lCancel${key}HitSuccess` as keyof PlayerStats] as number;
		const hitFail = player[`lCancel${key}HitFail` as keyof PlayerStats] as number;
		
		const totalSuccess = shieldSuccess + whiffSuccess + hitSuccess;
		const totalFail = shieldFail + whiffFail + hitFail;
		
		return {
			total: totalSuccess + totalFail,
			percent: getLCancelPercent(totalSuccess, totalFail),
			shield: { success: shieldSuccess, fail: shieldFail, percent: getLCancelPercent(shieldSuccess, shieldFail) },
			whiff: { success: whiffSuccess, fail: whiffFail, percent: getLCancelPercent(whiffSuccess, whiffFail) },
			hit: { success: hitSuccess, fail: hitFail, percent: getLCancelPercent(hitSuccess, hitFail) },
		};
	}
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

			<!-- L-Cancel Stats Section -->
			{#if playerStats && playerStats.length > 0}
				<Separator />
				
				<div class="space-y-3">
					<div class="flex items-center justify-between">
						<span class="text-sm font-medium">L-Cancel Stats</span>
						<Button 
							variant="ghost" 
							size="sm" 
							class="h-6 px-2 text-xs"
							onclick={() => showLCancelBreakdown = !showLCancelBreakdown}
						>
							{showLCancelBreakdown ? 'Hide' : 'Show'} Details
							{#if showLCancelBreakdown}
								<ChevronUp class="size-3 ml-1" />
							{:else}
								<ChevronDown class="size-3 ml-1" />
							{/if}
						</Button>
					</div>
					
					<!-- Overall L-Cancel for each player -->
					{#each playerStats as player}
						{@const totalPercent = getLCancelPercent(player.lCancelSuccessCount, player.lCancelFailCount)}
						{@const matchingMetaPlayer = metadata?.players.find(p => p.port === player.port)}
						<div class="rounded-lg border p-3 space-y-2">
							<div class="flex items-center justify-between">
								<div class="flex items-center gap-2">
									<CharacterIcon characterId={player.characterId} size="sm" />
									<span class="text-sm font-medium">{matchingMetaPlayer?.player_tag ?? player.connectCode ?? `P${player.port + 1}`}</span>
								</div>
								<div class="text-right">
									<div class="text-lg font-bold text-primary">{totalPercent}</div>
									<div class="text-xs text-muted-foreground">
										{player.lCancelSuccessCount}/{player.lCancelSuccessCount + player.lCancelFailCount}
									</div>
								</div>
							</div>
							
							<!-- Detailed breakdown (collapsible) -->
							{#if showLCancelBreakdown}
								<div class="pt-2 space-y-2">
									<div class="grid grid-cols-5 gap-1 text-center text-xs">
										{#each ['nair', 'fair', 'bair', 'uair', 'dair'] as aerial}
											{@const stats = getAerialStats(player, aerial as 'nair' | 'fair' | 'bair' | 'uair' | 'dair')}
											<div class="space-y-1">
												<div class="font-medium uppercase text-muted-foreground">{aerial}</div>
												<div class="rounded bg-muted/50 p-1">
													<div class="font-bold {stats.total > 0 ? 'text-foreground' : 'text-muted-foreground'}">{stats.percent}</div>
												</div>
											</div>
										{/each}
									</div>
									
									<!-- Target type breakdown -->
									<div class="text-xs space-y-1 pt-1">
										<div class="grid grid-cols-3 gap-2">
											<div class="text-center">
												<div class="text-muted-foreground mb-1">On Hit</div>
												<div class="grid grid-cols-5 gap-1">
													{#each ['nair', 'fair', 'bair', 'uair', 'dair'] as aerial}
														{@const stats = getAerialStats(player, aerial as 'nair' | 'fair' | 'bair' | 'uair' | 'dair')}
														<div class="text-center font-medium">{stats.hit.percent}</div>
													{/each}
												</div>
											</div>
											<div class="text-center">
												<div class="text-muted-foreground mb-1">On Shield</div>
												<div class="grid grid-cols-5 gap-1">
													{#each ['nair', 'fair', 'bair', 'uair', 'dair'] as aerial}
														{@const stats = getAerialStats(player, aerial as 'nair' | 'fair' | 'bair' | 'uair' | 'dair')}
														<div class="text-center font-medium">{stats.shield.percent}</div>
													{/each}
												</div>
											</div>
											<div class="text-center">
												<div class="text-muted-foreground mb-1">Whiffed</div>
												<div class="grid grid-cols-5 gap-1">
													{#each ['nair', 'fair', 'bair', 'uair', 'dair'] as aerial}
														{@const stats = getAerialStats(player, aerial as 'nair' | 'fair' | 'bair' | 'uair' | 'dair')}
														<div class="text-center font-medium">{stats.whiff.percent}</div>
													{/each}
												</div>
											</div>
										</div>
									</div>
								</div>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		{:else}
			<div class="text-center text-sm text-muted-foreground">No match data available</div>
		{/if}
	</CardContent>
</Card>

