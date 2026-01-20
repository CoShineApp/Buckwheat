<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { Loader2, Swords, Activity, Target, Zap, Shield, Skull, Sword, Filter, X, Calendar as CalendarIcon } from "@lucide/svelte";
	import { getCharacterName, getStageName, CHARACTER_NAMES, STAGE_NAMES } from "$lib/utils/characters";
	import { CharacterId, StageId } from "$lib/types/recording";
	import CharacterIcon from "$lib/components/recordings/CharacterIcon.svelte";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import * as Select from "$lib/components/ui/select";
	import * as Popover from "$lib/components/ui/popover";
	import { Calendar } from "$lib/components/ui/calendar";
	import { formatDecimal } from "$lib/utils/format";
	import { CalendarDate, type DateValue } from "@internationalized/date";

	interface AggregatedStats {
		totalGames: number;
		totalWins: number;
		avgLCancelPercent: number;
		avgRollsPerGame: number;
		avgOpeningsPerKill: number;
		avgDamagePerOpening: number;
		avgNeutralWins: number;
		avgInputsPerMinute: number;
		characterStats: Array<{
			characterId: number;
			games: number;
			wins: number;
		}>;
		stageStats: Array<{
			stageId: number;
			games: number;
			wins: number;
		}>;
	}

	interface StatsFilter {
		opponentCharacterId?: number;
		playerCharacterId?: number;
		stageId?: number;
		startTime?: string;
		endTime?: string;
	}

	let loading = $state(true);
	let stats = $state<AggregatedStats | null>(null);
	let error = $state<string | null>(null);
	let connectCode = $state("HATS#982");

	// Filter state
	let opponentCharacterFilter = $state<string | undefined>(undefined);
	let playerCharacterFilter = $state<string | undefined>(undefined);
	let stageFilter = $state<string | undefined>(undefined);
	let startDateValue = $state<DateValue | undefined>(undefined);
	let endDateValue = $state<DateValue | undefined>(undefined);
	let showFilters = $state(false);

	// Get all characters and stages for filter dropdowns
	const allCharacters = Object.entries(CHARACTER_NAMES).map(([id, name]) => ({
		id: parseInt(id),
		name
	})).sort((a, b) => a.name.localeCompare(b.name));

	const allStages = Object.entries(STAGE_NAMES).map(([id, name]) => ({
		id: parseInt(id),
		name
	})).sort((a, b) => a.name.localeCompare(b.name));

	// Build filter object from state
	let currentFilter = $derived<StatsFilter>({
		opponentCharacterId: opponentCharacterFilter ? parseInt(opponentCharacterFilter) : undefined,
		playerCharacterId: playerCharacterFilter ? parseInt(playerCharacterFilter) : undefined,
		stageId: stageFilter ? parseInt(stageFilter) : undefined,
		startTime: startDateValue ? `${startDateValue.year}-${String(startDateValue.month).padStart(2, '0')}-${String(startDateValue.day).padStart(2, '0')}T00:00:00` : undefined,
		endTime: endDateValue ? `${endDateValue.year}-${String(endDateValue.month).padStart(2, '0')}-${String(endDateValue.day).padStart(2, '0')}T23:59:59` : undefined,
	});

	// Check if any filters are active
	let hasActiveFilters = $derived(
		opponentCharacterFilter !== undefined ||
		playerCharacterFilter !== undefined ||
		stageFilter !== undefined ||
		startDateValue !== undefined ||
		endDateValue !== undefined
	);

	// Derived sorted stats to avoid mutating state in template
	let sortedCharacterStats = $derived(
		stats?.characterStats ? [...stats.characterStats].sort((a, b) => b.games - a.games) : []
	);

	let sortedStageStats = $derived(
		stats?.stageStats ? [...stats.stageStats].sort((a, b) => b.games - a.games) : []
	);

	$effect(() => {
		loadStats();
	});

	async function loadStats() {
		loading = true;
		error = null;
		try {
			// Only pass non-empty filter
			const filterToSend = hasActiveFilters ? currentFilter : null;
			
			stats = await invoke<AggregatedStats>("get_total_player_stats", {
				connectCode: connectCode,
				filter: filterToSend
			});
		} catch (e) {
			console.error("Failed to load total stats:", e);
			error = e instanceof Error ? e.message : "Failed to load stats";
		} finally {
			loading = false;
		}
	}

	function getWinRate(wins: number, games: number): string {
		if (games === 0) return "0%";
		return `${Math.round((wins / games) * 100)}%`;
	}

	function clearAllFilters() {
		opponentCharacterFilter = undefined;
		playerCharacterFilter = undefined;
		stageFilter = undefined;
		startDateValue = undefined;
		endDateValue = undefined;
	}

	function applyFilters() {
		loadStats();
	}

	function formatDateValue(dv: DateValue | undefined): string {
		if (!dv) return "Pick a date";
		return `${dv.month}/${dv.day}/${dv.year}`;
	}
</script>

<div class="container mx-auto max-w-7xl p-6 space-y-8">
	<div class="flex items-center justify-between">
		<h1 class="text-3xl font-bold tracking-tight">Total Stats</h1>
		<div class="flex items-center gap-4">
			<Button 
				variant={showFilters ? "default" : "outline"} 
				size="sm"
				onclick={() => showFilters = !showFilters}
				class="gap-2"
			>
				<Filter class="size-4" />
				Filters
				{#if hasActiveFilters}
					<span class="ml-1 bg-primary-foreground text-primary rounded-full px-1.5 py-0.5 text-xs font-bold">
						!
					</span>
				{/if}
			</Button>
			<div class="flex items-center gap-2">
				<span class="text-sm text-muted-foreground">Player:</span>
				<code class="bg-muted px-2 py-1 rounded text-sm font-mono">{connectCode}</code>
			</div>
		</div>
	</div>

	<!-- Filter Panel -->
	{#if showFilters}
		<Card.Root class="border-dashed border-2 bg-muted/30">
			<Card.Header class="pb-3">
				<div class="flex items-center justify-between">
					<Card.Title class="text-lg">Filter Stats</Card.Title>
					{#if hasActiveFilters}
						<Button variant="ghost" size="sm" onclick={clearAllFilters} class="gap-1 text-muted-foreground">
							<X class="size-3" />
							Clear all
						</Button>
					{/if}
				</div>
			</Card.Header>
			<Card.Content>
				<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-5">
					<!-- Player Character Filter -->
					<div class="space-y-2">
						<label class="text-sm font-medium text-muted-foreground">Played As</label>
						<Select.Root type="single" bind:value={playerCharacterFilter}>
							<Select.Trigger class="w-full">
								{#if playerCharacterFilter}
									<div class="flex items-center gap-2">
										<CharacterIcon characterId={parseInt(playerCharacterFilter)} size="sm" />
										{getCharacterName(parseInt(playerCharacterFilter))}
									</div>
								{:else}
									<span class="text-muted-foreground">Any character</span>
								{/if}
							</Select.Trigger>
							<Select.Content class="max-h-60">
								{#each allCharacters as char}
									<Select.Item value={String(char.id)} class="flex items-center gap-2">
										<CharacterIcon characterId={char.id} size="sm" />
										{char.name}
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>

					<!-- Opponent Character Filter -->
					<div class="space-y-2">
						<label class="text-sm font-medium text-muted-foreground">Played Against</label>
						<Select.Root type="single" bind:value={opponentCharacterFilter}>
							<Select.Trigger class="w-full">
								{#if opponentCharacterFilter}
									<div class="flex items-center gap-2">
										<CharacterIcon characterId={parseInt(opponentCharacterFilter)} size="sm" />
										{getCharacterName(parseInt(opponentCharacterFilter))}
									</div>
								{:else}
									<span class="text-muted-foreground">Any opponent</span>
								{/if}
							</Select.Trigger>
							<Select.Content class="max-h-60">
								{#each allCharacters as char}
									<Select.Item value={String(char.id)} class="flex items-center gap-2">
										<CharacterIcon characterId={char.id} size="sm" />
										{char.name}
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>

					<!-- Stage Filter -->
					<div class="space-y-2">
						<label class="text-sm font-medium text-muted-foreground">Stage</label>
						<Select.Root type="single" bind:value={stageFilter}>
							<Select.Trigger class="w-full">
								{#if stageFilter}
									{getStageName(parseInt(stageFilter))}
								{:else}
									<span class="text-muted-foreground">Any stage</span>
								{/if}
							</Select.Trigger>
							<Select.Content>
								{#each allStages as stage}
									<Select.Item value={String(stage.id)}>
										{stage.name}
									</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>

					<!-- Start Date -->
					<div class="space-y-2">
						<label class="text-sm font-medium text-muted-foreground">From Date</label>
						<Popover.Root>
							<Popover.Trigger>
								{#snippet child({ props })}
									<Button variant="outline" class="w-full justify-start text-left font-normal" {...props}>
										<CalendarIcon class="mr-2 size-4" />
										{formatDateValue(startDateValue)}
									</Button>
								{/snippet}
							</Popover.Trigger>
							<Popover.Content class="w-auto p-0">
								<Calendar type="single" bind:value={startDateValue} />
							</Popover.Content>
						</Popover.Root>
					</div>

					<!-- End Date -->
					<div class="space-y-2">
						<label class="text-sm font-medium text-muted-foreground">To Date</label>
						<Popover.Root>
							<Popover.Trigger>
								{#snippet child({ props })}
									<Button variant="outline" class="w-full justify-start text-left font-normal" {...props}>
										<CalendarIcon class="mr-2 size-4" />
										{formatDateValue(endDateValue)}
									</Button>
								{/snippet}
							</Popover.Trigger>
							<Popover.Content class="w-auto p-0">
								<Calendar type="single" bind:value={endDateValue} />
							</Popover.Content>
						</Popover.Root>
					</div>
				</div>

				<div class="mt-4 flex justify-end">
					<Button onclick={applyFilters} class="gap-2">
						<Filter class="size-4" />
						Apply Filters
					</Button>
				</div>
			</Card.Content>
		</Card.Root>
	{/if}

	{#if loading}
		<div class="flex flex-col items-center justify-center py-32 space-y-4">
			<Loader2 class="size-10 animate-spin text-primary" />
			<p class="text-muted-foreground text-lg">Crunching the numbers...</p>
		</div>
	{:else if error}
		<div class="flex flex-col items-center justify-center py-32 space-y-4">
			<p class="text-destructive font-semibold text-xl">Failed to load stats</p>
			<p class="text-muted-foreground max-w-md text-center">{error}</p>
			<Button variant="outline" onclick={loadStats}>Try Again</Button>
		</div>
	{:else if stats}
		<!-- Overview Cards -->
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
			<Card.Root>
				<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
					<Card.Title class="text-sm font-medium">Total Games</Card.Title>
					<Swords class="size-4 text-muted-foreground" />
				</Card.Header>
				<Card.Content>
					<div class="text-2xl font-bold">{stats.totalGames}</div>
					<p class="text-xs text-muted-foreground">
						{stats.totalWins} wins ({getWinRate(stats.totalWins, stats.totalGames)})
					</p>
				</Card.Content>
			</Card.Root>
			
			<Card.Root>
				<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
					<Card.Title class="text-sm font-medium">Avg L-Cancel</Card.Title>
					<Activity class="size-4 text-muted-foreground" />
				</Card.Header>
				<Card.Content>
					<div class="text-2xl font-bold">{formatDecimal(stats.avgLCancelPercent)}%</div>
					<p class="text-xs text-muted-foreground">Success rate</p>
				</Card.Content>
			</Card.Root>
			
			<Card.Root>
				<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
					<Card.Title class="text-sm font-medium">Neutral Wins</Card.Title>
					<Target class="size-4 text-muted-foreground" />
				</Card.Header>
				<Card.Content>
					<div class="text-2xl font-bold">{formatDecimal(stats.avgNeutralWins)}%</div>
					<p class="text-xs text-muted-foreground">Win rate</p>
				</Card.Content>
			</Card.Root>

			<Card.Root>
				<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
					<Card.Title class="text-sm font-medium">Inputs / Min</Card.Title>
					<Zap class="size-4 text-muted-foreground" />
				</Card.Header>
				<Card.Content>
					<div class="text-2xl font-bold">{formatDecimal(stats.avgInputsPerMinute, 0)}</div>
					<p class="text-xs text-muted-foreground">APM</p>
				</Card.Content>
			</Card.Root>

			<Card.Root>
				<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
					<Card.Title class="text-sm font-medium">Openings / Kill</Card.Title>
					<Skull class="size-4 text-muted-foreground" />
				</Card.Header>
				<Card.Content>
					<div class="text-2xl font-bold">{formatDecimal(stats.avgOpeningsPerKill)}</div>
					<p class="text-xs text-muted-foreground">Lower is better</p>
				</Card.Content>
			</Card.Root>

			<Card.Root>
				<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
					<Card.Title class="text-sm font-medium">Damage / Opening</Card.Title>
					<Sword class="size-4 text-muted-foreground" />
				</Card.Header>
				<Card.Content>
					<div class="text-2xl font-bold">{formatDecimal(stats.avgDamagePerOpening)}%</div>
					<p class="text-xs text-muted-foreground">Punish game</p>
				</Card.Content>
			</Card.Root>
			
			<Card.Root>
				<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
					<Card.Title class="text-sm font-medium">Avg Rolls/Game</Card.Title>
					<Shield class="size-4 text-muted-foreground" />
				</Card.Header>
				<Card.Content>
					<div class="text-2xl font-bold">{formatDecimal(stats.avgRollsPerGame)}</div>
					<p class="text-xs text-muted-foreground">Defensive habits</p>
				</Card.Content>
			</Card.Root>
		</div>

		<div class="grid gap-4 md:grid-cols-2">
			<!-- Character Stats (Opponents Faced) -->
			<Card.Root class="col-span-1">
				<Card.Header>
					<Card.Title>vs. Character Performance</Card.Title>
					<p class="text-sm text-muted-foreground">Win rate against each opponent</p>
				</Card.Header>
				<Card.Content>
					<div class="space-y-4">
						{#each sortedCharacterStats as charStat}
							<div class="flex items-center gap-4">
								<CharacterIcon characterId={charStat.characterId} size="md" />
								<div class="flex-1 space-y-1">
									<p class="text-sm font-medium leading-none">{getCharacterName(charStat.characterId)}</p>
									<div class="flex items-center gap-2 text-xs text-muted-foreground">
										<span>{charStat.games} games</span>
										<span>•</span>
										<span>{charStat.wins} wins</span>
									</div>
								</div>
								<div class="text-right">
									<div class="text-sm font-bold">{getWinRate(charStat.wins, charStat.games)}</div>
									<div class="text-xs text-muted-foreground">Win Rate</div>
								</div>
							</div>
						{/each}
					</div>
				</Card.Content>
			</Card.Root>

			<!-- Stage Stats -->
			<Card.Root class="col-span-1">
				<Card.Header>
					<Card.Title>Stage Performance</Card.Title>
					<p class="text-sm text-muted-foreground">Win rate on each stage</p>
				</Card.Header>
				<Card.Content>
					<div class="space-y-4">
						{#each sortedStageStats as stageStat}
							<div class="flex items-center justify-between">
								<div class="space-y-1">
									<p class="text-sm font-medium leading-none">{getStageName(stageStat.stageId)}</p>
									<p class="text-xs text-muted-foreground">{stageStat.games} games</p>
								</div>
								<div class="text-right">
									<div class="text-sm font-bold">{getWinRate(stageStat.wins, stageStat.games)}</div>
									<div class="text-xs text-muted-foreground">Win Rate</div>
								</div>
							</div>
						{/each}
					</div>
				</Card.Content>
			</Card.Root>
		</div>
	{:else}
		<div class="flex flex-col items-center justify-center py-32">
			<p class="text-muted-foreground">No stats found for {connectCode}</p>
		</div>
	{/if}
</div>
