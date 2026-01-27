<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { Loader2, Swords, Activity, Target, Zap, Shield, Skull, Sword, Filter, X, Calendar as CalendarIcon, RefreshCw, Database } from "@lucide/svelte";
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
	import { settings } from "$lib/stores/settings.svelte";

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

	// User's Slippi code (loaded from/saved to settings)
	let slippiCode = $state<string>(settings.slippiCode ?? "");
	let slippiCodeInput = $state<string>(settings.slippiCode ?? "");
	let isEditingCode = $state(!settings.slippiCode);

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

	// Build filter object from state
	let currentFilter = $derived<StatsFilter>({
		opponentCharacterId: opponentCharacterFilter ? parseInt(opponentCharacterFilter) : undefined,
		playerCharacterId: playerCharacterFilter ? parseInt(playerCharacterFilter) : undefined,
		stageId: stageFilter ? parseInt(stageFilter) : undefined,
		startTime: startDateValue ? `${startDateValue.year}-${String(startDateValue.month).padStart(2, '0')}-${String(startDateValue.day).padStart(2, '0')}T00:00:00` : undefined,
		endTime: endDateValue ? `${endDateValue.year}-${String(endDateValue.month).padStart(2, '0')}-${String(endDateValue.day).padStart(2, '0')}T23:59:59` : undefined,
	});

	// Check if any filters are active (empty string counts as no filter)
	let hasActiveFilters = $derived(
		(opponentCharacterFilter !== undefined && opponentCharacterFilter !== "") ||
		(playerCharacterFilter !== undefined && playerCharacterFilter !== "") ||
		(stageFilter !== undefined && stageFilter !== "") ||
		startDateValue !== undefined ||
		endDateValue !== undefined
	);

	// Historical sync state
	let isSyncing = $state(false);
	let syncProgress = $state({ current: 0, total: 0, skipped: 0 });
	let syncError = $state<string | null>(null);

	// Derived sorted stats to avoid mutating state in template
	let sortedCharacterStats = $derived(
		stats?.characterStats ? [...stats.characterStats].sort((a, b) => b.games - a.games) : []
	);

	let sortedStageStats = $derived(
		stats?.stageStats ? [...stats.stageStats].sort((a, b) => b.games - a.games) : []
	);

	// Load filter options when slippi code changes
	$effect(() => {
		loadFilterOptions(slippiCode || undefined);
	});

	// Load stats when slippi code changes
	$effect(() => {
		if (slippiCode) {
			loadStats();
		}
	});

	async function loadFilterOptions(connectCode?: string) {
		filterOptionsLoading = true;
		try {
			filterOptions = await invoke<AvailableFilterOptions>("get_available_filter_options", {
				connectCode: connectCode || null
			});
		} catch (e) {
			console.error("Failed to load filter options:", e);
		} finally {
			filterOptionsLoading = false;
		}
	}

	async function loadStats() {
		if (!slippiCode) return;
		
		loading = true;
		error = null;
		try {
			const filterToSend = hasActiveFilters ? currentFilter : null;
			
			stats = await invoke<AggregatedStats>("get_total_player_stats", {
				connectCode: slippiCode,
				filter: filterToSend
			});
		} catch (e) {
			console.error("Failed to load total stats:", e);
			error = e instanceof Error ? e.message : "Failed to load stats";
		} finally {
			loading = false;
		}
	}

	async function saveSlippiCode() {
		const code = slippiCodeInput.trim().toUpperCase();
		if (code) {
			slippiCode = code;
			await settings.set("slippiCode", code);
			isEditingCode = false;
		}
	}

	function editSlippiCode() {
		slippiCodeInput = slippiCode;
		isEditingCode = true;
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

	async function syncHistoricalData() {
		const slippiPath = settings.slippiPath;
		if (!slippiPath) {
			syncError = "Slippi directory not configured in settings";
			return;
		}

		isSyncing = true;
		syncError = null;
		syncProgress = { current: 0, total: 0, skipped: 0 };

		try {
			// Get list of all .slp files
			const slpFiles: string[] = await invoke("list_slp_files", { directory: slippiPath });
			syncProgress.total = slpFiles.length;

			if (slpFiles.length === 0) {
				syncError = "No .slp files found in directory";
				isSyncing = false;
				return;
			}

			// Import slippi-js parsing function
			const { parseAndSaveSlippiStats } = await import("$lib/services/slippi-stats");

			// Process each file
			for (let i = 0; i < slpFiles.length; i++) {
				const slpPath = slpFiles[i];
				syncProgress.current = i + 1;

				// Check if already synced
				const alreadySynced: boolean = await invoke("check_slp_synced", { slpPath });
				if (alreadySynced) {
					syncProgress.skipped++;
					continue;
				}

				// Parse and save stats (uses slp_path as recording_id for historical games)
				try {
					// Generate a unique ID using crypto hash of the full path
					const encoder = new TextEncoder();
					const data = encoder.encode(slpPath);
					const hashBuffer = await crypto.subtle.digest('SHA-256', data);
					const hashArray = Array.from(new Uint8Array(hashBuffer));
					const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
					const recordingId = `historical-${hashHex.slice(0, 32)}`;
					
					await parseAndSaveSlippiStats(slpPath, recordingId);
				} catch (e) {
					console.warn(`Failed to parse ${slpPath}:`, e);
					// Continue with other files
				}
			}

			// Refresh filter options and stats after sync
			await loadFilterOptions(slippiCode || undefined);
			if (slippiCode) {
				await loadStats();
			}
		} catch (e) {
			console.error("Historical sync failed:", e);
			syncError = e instanceof Error ? e.message : "Sync failed";
		} finally {
			isSyncing = false;
		}
	}
</script>

<div class="container mx-auto max-w-7xl p-6 space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<h1 class="text-3xl font-bold tracking-tight">Total Stats</h1>
		<div class="flex items-center gap-4">
			<!-- Sync Historical Data Button -->
			<Button
				variant="outline"
				size="sm"
				onclick={syncHistoricalData}
				disabled={isSyncing}
				class="gap-2"
			>
				{#if isSyncing}
					<Loader2 class="size-4 animate-spin" />
					<span>Syncing {syncProgress.current}/{syncProgress.total}...</span>
				{:else}
					<Database class="size-4" />
					<span>Sync Historical Data</span>
				{/if}
			</Button>

			<!-- Slippi Code Input -->
			<div class="flex items-center gap-2">
				{#if isEditingCode}
					<input
						type="text"
						bind:value={slippiCodeInput}
						placeholder="HATS#982"
						class="w-32 rounded-md border border-input bg-background px-3 py-1.5 text-sm uppercase placeholder:normal-case"
						onkeydown={(e) => e.key === 'Enter' && saveSlippiCode()}
					/>
					<Button size="sm" onclick={saveSlippiCode}>Save</Button>
				{:else}
					<span class="font-medium text-lg">{slippiCode}</span>
					<Button variant="ghost" size="sm" onclick={editSlippiCode}>Edit</Button>
				{/if}
			</div>
		</div>
	</div>

	<!-- Sync Status/Error -->
	{#if syncError}
		<div class="rounded-lg border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive">
			{syncError}
		</div>
	{/if}
	{#if isSyncing && syncProgress.skipped > 0}
		<div class="text-sm text-muted-foreground">
			Skipped {syncProgress.skipped} already synced files
		</div>
	{/if}

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
							<Select.Item value="" class="text-muted-foreground">Any character</Select.Item>
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
							<Select.Item value="" class="text-muted-foreground">Any opponent</Select.Item>
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
							<Select.Item value="" class="text-muted-foreground">Any stage</Select.Item>
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
	{:else if !slippiCode}
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
			<p class="text-muted-foreground">No stats found for {slippiCode}</p>
		</div>
	{/if}
</div>
