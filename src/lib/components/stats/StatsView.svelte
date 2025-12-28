<script lang="ts">
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { statsStore } from '$lib/stores/stats.svelte';
	import { recordingsStore } from '$lib/stores/recordings.svelte';
	import { navigation } from '$lib/stores/navigation.svelte';
	import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '$lib/components/ui/table';
	import { BarChart3, TrendingUp, Target, Zap, Medal, Clock, Activity } from '@lucide/svelte';
	import { getCharacterName, getStageName } from '$lib/utils/characters';
	import { handleTauriError } from '$lib/utils/errors';
	import CharacterIcon from '$lib/components/recordings/CharacterIcon.svelte';
	import type { PlayerGameStats } from '$lib/types/stats';

	// State for all stats
	let allPlayerStats = $state<PlayerGameStats[]>([]);
	let isLoadingStats = $state(false);
	let selectedRecordingStats = $state<PlayerGameStats[] | null>(null);
	let showDetailsModal = $state(false);

	onMount(async () => {
		// Load recordings to get game data
		await recordingsStore.refresh();
		
		// Load all player stats
		await loadAllStats();
	});

	async function loadAllStats() {
		isLoadingStats = true;
		try {
			allPlayerStats = await invoke<PlayerGameStats[]>('get_player_stats', {
				playerTag: null,
				characterId: null,
				limit: 100
			});
		} catch (err) {
			handleTauriError(err, 'Failed to load player stats');
		} finally {
			isLoadingStats = false;
		}
	}

	// Get recordings that have associated slippi metadata
	const recordingsWithStats = $derived(
		recordingsStore.recordings.filter(r => r.slp_path && r.slippi_metadata)
	);

	// Calculate aggregate stats from all player stats
	const aggregateStats = $derived.by(() => {
		if (allPlayerStats.length === 0) {
			return {
				totalGames: 0,
				avgLCancelRate: '0.0',
				avgTechRate: '0.0',
				avgAPM: '0.0',
				totalWins: 0,
				totalLosses: 0
			};
		}

		const totalGames = allPlayerStats.length;
		
		// L-cancel rate
		const totalLCancels = allPlayerStats.reduce((sum, s) => sum + s.l_cancel_hit + s.l_cancel_missed, 0);
		const totalLCancelHits = allPlayerStats.reduce((sum, s) => sum + s.l_cancel_hit, 0);
		const avgLCancelRate = totalLCancels > 0 ? (totalLCancelHits / totalLCancels) * 100 : 0;

		// Tech rate
		const totalTechs = allPlayerStats.reduce((sum, s) => sum + s.successful_techs + s.missed_techs, 0);
		const totalSuccessfulTechs = allPlayerStats.reduce((sum, s) => sum + s.successful_techs, 0);
		const avgTechRate = totalTechs > 0 ? (totalSuccessfulTechs / totalTechs) * 100 : 0;

		// APM
		const avgAPM = allPlayerStats.reduce((sum, s) => sum + s.apm, 0) / totalGames;

		// Win/loss (simple heuristic: kills > deaths = win)
		const totalWins = allPlayerStats.filter(s => s.kills > s.deaths).length;
		const totalLosses = allPlayerStats.filter(s => s.deaths > s.kills).length;

		return {
			totalGames,
			avgLCancelRate: avgLCancelRate.toFixed(1),
			avgTechRate: avgTechRate.toFixed(1),
			avgAPM: avgAPM.toFixed(1),
			totalWins,
			totalLosses
		};
	});

	function formatDate(dateStr: string): string {
		try {
			const date = new Date(dateStr);
			return date.toLocaleDateString('en-US', { 
				month: 'short', 
				day: 'numeric',
				year: 'numeric',
				hour: 'numeric',
				minute: '2-digit'
			});
		} catch {
			return dateStr;
		}
	}

	function formatDuration(frames: number): string {
		const seconds = Math.floor(frames / 60);
		const minutes = Math.floor(seconds / 60);
		const secs = seconds % 60;
		return `${minutes}:${secs.toString().padStart(2, '0')}`;
	}

	function getPlayerCharacter(recording: typeof recordingsWithStats[0]): number | null {
		if (!recording.slippi_metadata) return null;
		return recording.slippi_metadata.players[0]?.character_id ?? null;
	}

	function getOpponentCharacter(recording: typeof recordingsWithStats[0]): number | null {
		if (!recording.slippi_metadata) return null;
		return recording.slippi_metadata.players[1]?.character_id ?? null;
	}

	async function viewGameStats(recordingId: string) {
		// Load stats for this specific recording
		try {
			selectedRecordingStats = await invoke<PlayerGameStats[]>('get_recording_stats', {
				recordingId
			});
			if (selectedRecordingStats && selectedRecordingStats.length > 0) {
				showDetailsModal = true;
			} else {
				// No stats calculated yet, just view the replay
				navigation.navigateToReplay(recordingId);
			}
		} catch (err) {
			handleTauriError(err, 'Failed to load game stats');
			// Fallback to replay view
			navigation.navigateToReplay(recordingId);
		}
	}

	function closeDetailsModal() {
		showDetailsModal = false;
		selectedRecordingStats = null;
	}
</script>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold tracking-tight">Game Statistics</h1>
			<p class="text-muted-foreground">
				Track your performance across all recorded games
			</p>
		</div>
		{#if isLoadingStats}
			<Button disabled>
				<Activity class="h-4 w-4 mr-2 animate-spin" />
				Loading Stats...
			</Button>
		{:else}
			<Button onclick={loadAllStats}>
				<Activity class="h-4 w-4 mr-2" />
				Refresh Stats
			</Button>
		{/if}
	</div>

	<!-- Stats Overview Cards -->
	<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Total Games</CardTitle>
				<BarChart3 class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{aggregateStats.totalGames}</div>
				<p class="text-xs text-muted-foreground">
					{aggregateStats.totalWins}W - {aggregateStats.totalLosses}L
				</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Avg L-Cancel</CardTitle>
				<Target class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{aggregateStats.avgLCancelRate}%</div>
				<p class="text-xs text-muted-foreground">
					Success rate
				</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Avg Tech Rate</CardTitle>
				<TrendingUp class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{aggregateStats.avgTechRate}%</div>
				<p class="text-xs text-muted-foreground">
					Success rate
				</p>
			</CardContent>
		</Card>

		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Avg APM</CardTitle>
				<Zap class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{aggregateStats.avgAPM}</div>
				<p class="text-xs text-muted-foreground">
					Actions per minute
				</p>
			</CardContent>
		</Card>
	</div>

	<!-- Games List -->
	<Card>
		<CardHeader>
			<CardTitle>Recent Games</CardTitle>
			<CardDescription>
				Click on a game to view detailed stats and replay
			</CardDescription>
		</CardHeader>
		<CardContent>
			{#if recordingsWithStats.length === 0}
				<div class="flex flex-col items-center justify-center py-12 text-center">
					<Medal class="h-12 w-12 text-muted-foreground/50 mb-4" />
					<h3 class="text-lg font-semibold mb-2">No games recorded yet</h3>
					<p class="text-sm text-muted-foreground mb-4">
						Start recording games to track your stats and improvement over time
					</p>
					<Button onclick={() => navigation.navigateTo('home')}>
						Go to Home
					</Button>
				</div>
			{:else}
				<div class="rounded-md border">
					<Table>
						<TableHeader>
							<TableRow>
								<TableHead class="w-[180px]">Date</TableHead>
								<TableHead>Characters</TableHead>
								<TableHead>Stage</TableHead>
								<TableHead class="w-[100px]">Duration</TableHead>
								<TableHead class="text-right w-[100px]">Actions</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{#each recordingsWithStats as recording}
								<TableRow class="cursor-pointer hover:bg-muted/50">
									<TableCell class="font-medium">
										{formatDate(recording.start_time)}
									</TableCell>
									<TableCell>
										<div class="flex items-center gap-2">
											{#if getPlayerCharacter(recording) !== null}
												{@const playerChar = getPlayerCharacter(recording)}
												<CharacterIcon characterId={playerChar!} size="sm" />
												<span class="text-sm">
													{getCharacterName(playerChar!)}
												</span>
											{:else}
												<span class="text-sm text-muted-foreground">Unknown</span>
											{/if}
											{#if getOpponentCharacter(recording) !== null}
												{@const opponentChar = getOpponentCharacter(recording)}
												<span class="text-muted-foreground">vs</span>
												<CharacterIcon characterId={opponentChar!} size="sm" />
												<span class="text-sm">
													{getCharacterName(opponentChar!)}
												</span>
											{/if}
										</div>
									</TableCell>
									<TableCell class="text-sm text-muted-foreground">
										{recording.slippi_metadata?.stage ? `Stage ${recording.slippi_metadata.stage}` : 'Unknown'}
									</TableCell>
									<TableCell>
										{#if recording.slippi_metadata?.game_duration}
											<div class="flex items-center gap-1 text-sm">
												<Clock class="h-3 w-3" />
												{formatDuration(recording.slippi_metadata.game_duration)}
											</div>
										{:else}
											<span class="text-sm text-muted-foreground">--</span>
										{/if}
									</TableCell>
									<TableCell class="text-right">
										<Button 
											variant="ghost" 
											size="sm"
											onclick={() => viewGameStats(recording.id)}
										>
											View
										</Button>
									</TableCell>
								</TableRow>
							{/each}
						</TableBody>
					</Table>
				</div>
			{/if}
		</CardContent>
	</Card>

	<!-- Recent Stats Breakdown -->
	{#if allPlayerStats.length > 0}
		<Card>
			<CardHeader>
				<CardTitle>All Games Performance</CardTitle>
				<CardDescription>
					Individual game statistics sorted by date
				</CardDescription>
			</CardHeader>
			<CardContent>
				<div class="rounded-md border">
					<Table>
						<TableHeader>
							<TableRow>
								<TableHead>Player</TableHead>
								<TableHead>Character</TableHead>
								<TableHead class="text-center">K/D</TableHead>
								<TableHead class="text-center">L-Cancel</TableHead>
								<TableHead class="text-center">Tech Rate</TableHead>
								<TableHead class="text-center">APM</TableHead>
								<TableHead class="w-[150px]">Date</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{#each allPlayerStats.slice(0, 20) as stat}
								<TableRow>
									<TableCell class="font-medium">{stat.player_tag}</TableCell>
									<TableCell>
										<div class="flex items-center gap-2">
											<CharacterIcon characterId={stat.character_id} size="xs" />
											<span class="text-sm">{getCharacterName(stat.character_id)}</span>
										</div>
									</TableCell>
									<TableCell class="text-center">
										<span class={stat.kills > stat.deaths ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}>
											{stat.kills}/{stat.deaths}
										</span>
									</TableCell>
									<TableCell class="text-center">
										{stat.l_cancel_hit + stat.l_cancel_missed > 0 
											? ((stat.l_cancel_hit / (stat.l_cancel_hit + stat.l_cancel_missed)) * 100).toFixed(0)
											: '--'}%
									</TableCell>
									<TableCell class="text-center">
										{stat.successful_techs + stat.missed_techs > 0
											? ((stat.successful_techs / (stat.successful_techs + stat.missed_techs)) * 100).toFixed(0)
											: '--'}%
									</TableCell>
									<TableCell class="text-center">{stat.apm.toFixed(0)}</TableCell>
									<TableCell class="text-sm text-muted-foreground">
										{formatDate(stat.game_date)}
									</TableCell>
								</TableRow>
							{/each}
						</TableBody>
					</Table>
				</div>
			</CardContent>
		</Card>
	{/if}
</div>

<!-- Game Stats Details Modal -->
<Dialog open={showDetailsModal} onOpenChange={(open) => !open && closeDetailsModal()}>
	<DialogContent class="max-w-4xl max-h-[80vh] overflow-y-auto">
		<DialogHeader>
			<DialogTitle>Game Statistics</DialogTitle>
			<DialogDescription>
				Detailed performance breakdown for this match
			</DialogDescription>
		</DialogHeader>

		{#if selectedRecordingStats && selectedRecordingStats.length > 0}
			<div class="space-y-6">
				<!-- Per-player stats cards -->
				{#each selectedRecordingStats as playerStats}
					<Card>
						<CardHeader>
							<div class="flex items-center justify-between">
								<div class="flex items-center gap-3">
									<CharacterIcon characterId={playerStats.character_id} size="md" />
									<div>
										<CardTitle>{playerStats.player_tag}</CardTitle>
										<CardDescription>
											Port {playerStats.player_port} • {getCharacterName(playerStats.character_id)}
										</CardDescription>
									</div>
								</div>
								<div class="text-right">
									<div class="text-2xl font-bold">{playerStats.kills} - {playerStats.deaths}</div>
									<p class="text-xs text-muted-foreground">Kills - Deaths</p>
								</div>
							</div>
						</CardHeader>
						<CardContent>
							<div class="grid gap-4 md:grid-cols-3">
								<!-- Technical Stats -->
								<div class="space-y-3">
									<h4 class="font-semibold text-sm border-b pb-2">Technical</h4>
									<div class="space-y-2 text-sm">
										<div class="flex justify-between">
											<span class="text-muted-foreground">L-Cancel Rate</span>
											<span class="font-medium">
												{playerStats.l_cancel_hit + playerStats.l_cancel_missed > 0 
													? ((playerStats.l_cancel_hit / (playerStats.l_cancel_hit + playerStats.l_cancel_missed)) * 100).toFixed(1)
													: '0.0'}%
											</span>
										</div>
										<div class="flex justify-between text-xs text-muted-foreground">
											<span>└ Hit/Missed</span>
											<span>{playerStats.l_cancel_hit} / {playerStats.l_cancel_missed}</span>
										</div>
										<div class="flex justify-between">
											<span class="text-muted-foreground">Tech Rate</span>
											<span class="font-medium">
												{playerStats.successful_techs + playerStats.missed_techs > 0
													? ((playerStats.successful_techs / (playerStats.successful_techs + playerStats.missed_techs)) * 100).toFixed(1)
													: '0.0'}%
											</span>
										</div>
										<div class="flex justify-between text-xs text-muted-foreground">
											<span>└ Success/Missed</span>
											<span>{playerStats.successful_techs} / {playerStats.missed_techs}</span>
										</div>
										<div class="flex justify-between">
											<span class="text-muted-foreground">APM</span>
											<span class="font-medium">{playerStats.apm.toFixed(1)}</span>
										</div>
									</div>
								</div>

								<!-- Neutral & Punish -->
								<div class="space-y-3">
									<h4 class="font-semibold text-sm border-b pb-2">Neutral & Punish</h4>
									<div class="space-y-2 text-sm">
										<div class="flex justify-between">
											<span class="text-muted-foreground">Neutral Wins</span>
											<span class="font-medium">{playerStats.neutral_wins}</span>
										</div>
										<div class="flex justify-between">
											<span class="text-muted-foreground">Openings</span>
											<span class="font-medium">{playerStats.openings}</span>
										</div>
										<div class="flex justify-between">
											<span class="text-muted-foreground">Damage/Opening</span>
											<span class="font-medium">
												{playerStats.damage_per_opening?.toFixed(1) ?? '--'}%
											</span>
										</div>
										<div class="flex justify-between">
											<span class="text-muted-foreground">Openings/Kill</span>
											<span class="font-medium">
												{playerStats.openings_per_kill?.toFixed(1) ?? '--'}
											</span>
										</div>
										<div class="flex justify-between">
											<span class="text-muted-foreground">Avg Kill %</span>
											<span class="font-medium">
												{playerStats.avg_kill_percent?.toFixed(1) ?? '--'}%
											</span>
										</div>
									</div>
								</div>

								<!-- Movement & Input -->
								<div class="space-y-3">
									<h4 class="font-semibold text-sm border-b pb-2">Movement</h4>
									<div class="space-y-2 text-sm">
										<div class="flex justify-between">
											<span class="text-muted-foreground">Wavedashes</span>
											<span class="font-medium">{playerStats.wavedash_count}</span>
										</div>
										<div class="flex justify-between">
											<span class="text-muted-foreground">Dashdances</span>
											<span class="font-medium">{playerStats.dashdance_count}</span>
										</div>
										<div class="flex justify-between">
											<span class="text-muted-foreground">Grab Attempts</span>
											<span class="font-medium">{playerStats.grab_attempts}</span>
										</div>
										<div class="flex justify-between">
											<span class="text-muted-foreground">Damage Dealt</span>
											<span class="font-medium">{playerStats.total_damage_dealt.toFixed(1)}%</span>
										</div>
										<div class="flex justify-between">
											<span class="text-muted-foreground">Damage Taken</span>
											<span class="font-medium">{playerStats.total_damage_taken.toFixed(1)}%</span>
										</div>
									</div>
								</div>
							</div>
						</CardContent>
					</Card>
				{/each}

				<!-- Action buttons -->
				<div class="flex gap-2 justify-end">
					<Button variant="outline" onclick={closeDetailsModal}>
						Close
					</Button>
					<Button onclick={() => {
						if (selectedRecordingStats && selectedRecordingStats.length > 0) {
							navigation.navigateToReplay(selectedRecordingStats[0].recording_id);
						}
					}}>
						View Replay
					</Button>
				</div>
			</div>
		{/if}
	</DialogContent>
</Dialog>