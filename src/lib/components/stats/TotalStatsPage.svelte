<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { Loader2, Swords, Activity, Target, Zap, Shield, Skull, Sword, Filter, X, Calendar as CalendarIcon } from "@lucide/svelte";
	import { getCharacterName, getStageName } from "$lib/utils/characters";
	import CharacterIcon from "$lib/components/recordings/CharacterIcon.svelte";
	import StageIcon from "$lib/components/recordings/StageIcon.svelte";
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

	interface AvailableFilterOptions {
		connectCodes: string[];
		playerCharacters: number[];
		opponentCharacters: number[];
		stages: number[];
	}

	let loading = $state(true);
	let stats = $state<AggregatedStats | null>(null);
	let error = $state<string | null>(null);
	let filterOptions = $state<AvailableFilterOptions | null>(null);
	let filterOptionsLoading = $state(true);

	// Selected player (connect code)
	let selectedPlayer = $state<string | undefined>(undefined);

	// Filter state
	let opponentCharacterFilter = $state<string | undefined>(undefined);
	let playerCharacterFilter = $state<string | undefined>(undefined);
	let stageFilter = $state<string | undefined>(undefined);
	let startDateValue = $state<DateValue | undefined>(undefined);
	let endDateValue = $state<DateValue | undefined>(undefined);

	// Derived: available characters and stages from filter options
	let availablePlayerCharacters = $derived(
		filterOptions?.playerCharacters.map(id => ({
			id,
			name: getCharacterName(id)
		})).sort((a, b) => a.name.localeCompare(b.name)) ?? []
	);

	let availableOpponentCharacters = $derived(
		filterOptions?.opponentCharacters.map(id => ({
			id,
			name: getCharacterName(id)
		})).sort((a, b) => a.name.localeCompare(b.name)) ?? []
	);

	let availableStages = $derived(
		filterOptions?.stages.map(id => ({
			id,
			name: getStageName(id)
		})).sort((a, b) => a.name.localeCompare(b.name)) ?? []
	);

	let availablePlayers = $derived(filterOptions?.connectCodes ?? []);

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

	// Load filter options on mount
	$effect(() => {
		loadFilterOptions();
	});

	// Load stats when player changes
	$effect(() => {
		if (selectedPlayer) {
			loadStats();
		}
	});

	async function loadFilterOptions() {
		filterOptionsLoading = true;
		try {
			filterOptions = await invoke<AvailableFilterOptions>("get_available_filter_options");
			// Auto-select first player if available
			if (filterOptions.connectCodes.length > 0 && !selectedPlayer) {
				selectedPlayer = filterOptions.connectCodes[0];
			}
		} catch (e) {
			console.error("Failed to load filter options:", e);
		} finally {
			filterOptionsLoading = false;
		}
	}

	async function loadStats() {
		if (!selectedPlayer) return;
		
		loading = true;
		error = null;
		try {
			const filterToSend = hasActiveFilters ? currentFilter : null;
			
			stats = await invoke<AggregatedStats>("get_total_player_stats", {
				connectCode: selectedPlayer,
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

<div class="container mx-auto max-w-7xl p-6 space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<h1 class="text-3xl font-bold tracking-tight">Total Stats</h1>
		<div class="flex items-center gap-2">
			<span class="text-sm text-muted-foreground">Player:</span>
			{#if filterOptionsLoading}
				<Loader2 class="size-4 animate-spin" />
			{:else if availablePlayers.length > 0}
				<Select.Root type="single" bind:value={selectedPlayer}>
					<Select.Trigger class="w-40">
						{selectedPlayer ?? "Select player"}
					</Select.Trigger>
					<Select.Content>
						{#each availablePlayers as player}
							<Select.Item value={player}>{player}</Select.Item>
						{/each}
					</Select.Content>
				</Select.Root>
			{:else}
				<span class="text-muted-foreground text-sm">No players found</span>
			{/if}
		</div>
	</div>

	<!-- Filters (always visible) -->
	<Card.Root class="border-dashed border-2 bg-muted/30">
		<Card.Content class="pt-4 pb-4">
			<div class="flex flex-wrap items-end gap-4">
				<!-- Player Character Filter -->
				<div class="space-y-1.5">
					<span class="text-xs font-medium text-muted-foreground">Played As</span>
					<Select.Root type="single" bind:value={playerCharacterFilter}>
						<Select.Trigger class="w-40">
							{#if playerCharacterFilter}
								<div class="flex items-center gap-2">
									<CharacterIcon characterId={parseInt(playerCharacterFilter)} size="sm" />
									<span class="truncate">{getCharacterName(parseInt(playerCharacterFilter))}</span>
								</div>
							{:else}
								<span class="text-muted-foreground">Any character</span>
							{/if}
						</Select.Trigger>
						<Select.Content class="max-h-60">
							{#each availablePlayerCharacters as char}
								<Select.Item value={String(char.id)} class="flex items-center gap-2">
									<CharacterIcon characterId={char.id} size="sm" />
									{char.name}
								</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				<!-- Opponent Character Filter -->
				<div class="space-y-1.5">
					<span class="text-xs font-medium text-muted-foreground">Played Against</span>
					<Select.Root type="single" bind:value={opponentCharacterFilter}>
						<Select.Trigger class="w-40">
							{#if opponentCharacterFilter}
								<div class="flex items-center gap-2">
									<CharacterIcon characterId={parseInt(opponentCharacterFilter)} size="sm" />
									<span class="truncate">{getCharacterName(parseInt(opponentCharacterFilter))}</span>
								</div>
							{:else}
								<span class="text-muted-foreground">Any opponent</span>
							{/if}
						</Select.Trigger>
						<Select.Content class="max-h-60">
							{#each availableOpponentCharacters as char}
								<Select.Item value={String(char.id)} class="flex items-center gap-2">
									<CharacterIcon characterId={char.id} size="sm" />
									{char.name}
								</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				<!-- Stage Filter -->
				<div class="space-y-1.5">
					<span class="text-xs font-medium text-muted-foreground">Stage</span>
					<Select.Root type="single" bind:value={stageFilter}>
						<Select.Trigger class="w-40">
							{#if stageFilter}
								<div class="flex items-center gap-2">
									<StageIcon stageId={parseInt(stageFilter)} size="sm" />
									<span class="truncate">{getStageName(parseInt(stageFilter))}</span>
								</div>
							{:else}
								<span class="text-muted-foreground">Any stage</span>
							{/if}
						</Select.Trigger>
						<Select.Content>
							{#each availableStages as stage}
								<Select.Item value={String(stage.id)} class="flex items-center gap-2">
									<StageIcon stageId={stage.id} size="sm" />
									{stage.name}
								</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
				</div>

				<!-- Date Range -->
				<div class="space-y-1.5">
					<span class="text-xs font-medium text-muted-foreground">From</span>
					<Popover.Root>
						<Popover.Trigger>
							{#snippet child({ props })}
								<Button variant="outline" class="w-32 justify-start text-left font-normal h-9" {...props}>
									<CalendarIcon class="mr-2 size-3" />
									<span class="text-xs">{formatDateValue(startDateValue)}</span>
								</Button>
							{/snippet}
						</Popover.Trigger>
						<Popover.Content class="w-auto p-0">
							<Calendar type="single" bind:value={startDateValue} />
						</Popover.Content>
					</Popover.Root>
				</div>

				<div class="space-y-1.5">
					<span class="text-xs font-medium text-muted-foreground">To</span>
					<Popover.Root>
						<Popover.Trigger>
							{#snippet child({ props })}
								<Button variant="outline" class="w-32 justify-start text-left font-normal h-9" {...props}>
									<CalendarIcon class="mr-2 size-3" />
									<span class="text-xs">{formatDateValue(endDateValue)}</span>
								</Button>
							{/snippet}
						</Popover.Trigger>
						<Popover.Content class="w-auto p-0">
							<Calendar type="single" bind:value={endDateValue} />
						</Popover.Content>
					</Popover.Root>
				</div>

				<!-- Actions -->
				<div class="flex items-center gap-2 ml-auto">
					{#if hasActiveFilters}
						<Button variant="ghost" size="sm" onclick={clearAllFilters} class="gap-1 text-muted-foreground h-9">
							<X class="size-3" />
							Clear
						</Button>
					{/if}
					<Button onclick={applyFilters} size="sm" class="gap-1.5 h-9">
						<Filter class="size-3" />
						Apply
					</Button>
				</div>
			</div>
		</Card.Content>
	</Card.Root>

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
	{:else if !selectedPlayer}
		<div class="flex flex-col items-center justify-center py-32">
			<p class="text-muted-foreground">Select a player to view stats</p>
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
							<div class="flex items-center gap-3">
								<StageIcon stageId={stageStat.stageId} size="sm" />
								<div class="flex-1 space-y-1">
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
			<p class="text-muted-foreground">No stats found for {selectedPlayer}</p>
		</div>
	{/if}
</div>
