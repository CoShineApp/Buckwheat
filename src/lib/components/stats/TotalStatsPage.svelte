<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
	import { Loader2, Filter, X, Clock, Database, TrendingUp } from "@lucide/svelte";
	import { getCharacterName, getStageName } from "$lib/utils/characters";
	import CharacterIcon from "$lib/components/recordings/CharacterIcon.svelte";
	import StageIcon from "$lib/components/recordings/StageIcon.svelte";
	import { Button } from "$lib/components/ui/button";
	import * as Card from "$lib/components/ui/card";
	import * as Select from "$lib/components/ui/select";
	import * as Chart from "$lib/components/ui/chart";
	import { formatDecimal } from "$lib/utils/format";
	import { settings } from "$lib/stores/settings.svelte";
	import { scaleUtc } from "d3-scale";
	import { Area, AreaChart, ChartClipPath } from "layerchart";
	import { curveMonotoneX } from "d3-shape";
	import { cubicInOut } from "svelte/easing";

	// Time range options
	type TimeRange = "" | "today" | "week" | "month" | "3months" | "year";
	const timeRangeOptions: { value: TimeRange; label: string }[] = [
		{ value: "", label: "All Time" },
		{ value: "today", label: "Today" },
		{ value: "week", label: "Last Week" },
		{ value: "month", label: "Last Month" },
		{ value: "3months", label: "Last 3 Months" },
		{ value: "year", label: "Last Year" },
	];

	function getTimeRangeFilter(range: TimeRange): { startTime?: string; endTime?: string } {
		if (!range) return {};
		
		const now = new Date();
		const endTime = now.toISOString();
		let startDate: Date;
		
		switch (range) {
			case "today":
				startDate = new Date(now.getFullYear(), now.getMonth(), now.getDate());
				break;
			case "week":
				startDate = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000);
				break;
			case "month":
				startDate = new Date(now.getTime() - 30 * 24 * 60 * 60 * 1000);
				break;
			case "3months":
				startDate = new Date(now.getTime() - 90 * 24 * 60 * 60 * 1000);
				break;
			case "year":
				startDate = new Date(now.getTime() - 365 * 24 * 60 * 60 * 1000);
				break;
			default:
				return {};
		}
		
		return {
			startTime: startDate.toISOString(),
			endTime,
		};
	}

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
	let timeRangeFilter = $state<TimeRange>("");

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
	let currentFilter = $derived.by<StatsFilter>(() => {
		const timeFilter = getTimeRangeFilter(timeRangeFilter);
		return {
			opponentCharacterId: opponentCharacterFilter ? parseInt(opponentCharacterFilter) : undefined,
			playerCharacterId: playerCharacterFilter ? parseInt(playerCharacterFilter) : undefined,
			stageId: stageFilter ? parseInt(stageFilter) : undefined,
			startTime: timeFilter.startTime,
			endTime: timeFilter.endTime,
		};
	});

	// Check if any filters are active (empty string counts as no filter)
	let hasActiveFilters = $derived(
		(opponentCharacterFilter !== undefined && opponentCharacterFilter !== "") ||
		(playerCharacterFilter !== undefined && playerCharacterFilter !== "") ||
		(stageFilter !== undefined && stageFilter !== "") ||
		timeRangeFilter !== ""
	);

	// Historical sync state
	let isSyncing = $state(false);
	let syncProgress = $state({ current: 0, total: 0, skipped: 0 });
	let syncError = $state<string | null>(null);

	// Chart state
	type ChartMetric = "lcancel" | "winrate" | "apm" | "openings" | "damage" | "neutral" | "rolls";
	let selectedChartMetric = $state<ChartMetric>("lcancel");
	let chartData = $state<Array<{ date: Date; value: number }>>([]);
	let chartLoading = $state(false);

	// Chart metric configuration - all use the app's primary green color
	const chartColor = "var(--primary)";
	const chartMetricConfig: Record<ChartMetric, { label: string; color: string; format: (v: number) => string }> = {
		lcancel: { label: "L-Cancel %", color: chartColor, format: (v) => `${v.toFixed(1)}%` },
		winrate: { label: "Win Rate", color: chartColor, format: (v) => `${v.toFixed(0)}%` },
		apm: { label: "APM", color: chartColor, format: (v) => v.toFixed(0) },
		openings: { label: "Openings/Kill", color: chartColor, format: (v) => v.toFixed(2) },
		damage: { label: "Damage/Opening", color: chartColor, format: (v) => `${v.toFixed(1)}%` },
		neutral: { label: "Neutral Win %", color: chartColor, format: (v) => `${v.toFixed(1)}%` },
		rolls: { label: "Rolls/Game", color: chartColor, format: (v) => v.toFixed(1) },
	};

	const currentChartConfig = $derived({
		value: { label: chartMetricConfig[selectedChartMetric].label, color: chartMetricConfig[selectedChartMetric].color }
	} satisfies Chart.ChartConfig);

	// Load chart data when metric or slippi code changes
	$effect(() => {
		if (slippiCode && stats) {
			loadChartData();
		}
	});

	// Store raw time series data so we can switch metrics without refetching
	let rawTimeSeriesData = $state<Array<{
		date: string;
		l_cancel_percent: number | null;
		win: boolean;
		inputs_per_minute: number | null;
		openings_per_kill: number | null;
		damage_per_opening: number | null;
		neutral_win_ratio: number | null;
		roll_count: number | null;
	}>>([]);

	// Helper to get date string (YYYY-MM-DD) for grouping by day
	function getDateKey(dateStr: string): string {
		const d = new Date(dateStr);
		return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
	}

	// Helper to extract metric value from a data point
	function getMetricValue(d: typeof rawTimeSeriesData[0], metric: ChartMetric): number {
		switch (metric) {
			case "lcancel":
				return d.l_cancel_percent ?? 0;
			case "winrate":
				return d.win ? 100 : 0;
			case "apm":
				return d.inputs_per_minute ?? 0;
			case "openings":
				return d.openings_per_kill ?? 0;
			case "damage":
				return d.damage_per_opening ?? 0;
			case "neutral":
				return (d.neutral_win_ratio ?? 0) * 100;
			case "rolls":
				return d.roll_count ?? 0;
		}
	}

	// Aggregate data by day and compute average for the selected metric
	function aggregateByDay(data: typeof rawTimeSeriesData, metric: ChartMetric): Array<{ date: Date; value: number }> {
		const dayMap = new Map<string, { sum: number; count: number }>();
		
		for (const d of data) {
			const dayKey = getDateKey(d.date);
			const value = getMetricValue(d, metric);
			
			const existing = dayMap.get(dayKey);
			if (existing) {
				existing.sum += value;
				existing.count += 1;
			} else {
				dayMap.set(dayKey, { sum: value, count: 1 });
			}
		}
		
		// Convert to array and sort by date
		return Array.from(dayMap.entries())
			.map(([dayKey, { sum, count }]) => ({
				date: new Date(dayKey),
				value: sum / count
			}))
			.sort((a, b) => a.date.getTime() - b.date.getTime());
	}

	// Derived chart data that updates when metric changes
	const chartDataAggregated = $derived(aggregateByDay(rawTimeSeriesData, selectedChartMetric));

	async function loadChartData() {
		chartLoading = true;
		try {
			// Fetch time-series data from backend
			rawTimeSeriesData = await invoke<typeof rawTimeSeriesData>("get_player_stats_timeseries", {
				connectCode: slippiCode,
				filter: hasActiveFilters ? currentFilter : null
			});

			// chartData is now computed via chartDataAggregated derived
			chartData = chartDataAggregated;
		} catch (e) {
			console.error("Failed to load chart data:", e);
			rawTimeSeriesData = [];
			chartData = [];
		} finally {
			chartLoading = false;
		}
	}

	// Update chartData when metric changes (uses cached raw data)
	$effect(() => {
		if (rawTimeSeriesData.length > 0) {
			chartData = aggregateByDay(rawTimeSeriesData, selectedChartMetric);
		}
	});

	function selectChartMetric(metric: ChartMetric) {
		selectedChartMetric = metric;
	}

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
		timeRangeFilter = "";
	}

	function applyFilters() {
		loadStats();
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

				<!-- Time Range -->
				<div class="space-y-1.5">
					<span class="text-xs font-medium text-muted-foreground">Time Range</span>
					<Select.Root type="single" bind:value={timeRangeFilter}>
						<Select.Trigger class="w-36">
							<div class="flex items-center gap-2">
								<Clock class="size-3 text-muted-foreground" />
								<span>{timeRangeOptions.find(o => o.value === timeRangeFilter)?.label ?? "All Time"}</span>
							</div>
						</Select.Trigger>
						<Select.Content>
							{#each timeRangeOptions as option}
								<Select.Item value={option.value}>{option.label}</Select.Item>
							{/each}
						</Select.Content>
					</Select.Root>
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
		<!-- Performance Chart -->
		<Card.Root>
			<Card.Header class="flex items-center gap-2 space-y-0 border-b py-5 sm:flex-row">
				<div class="grid flex-1 gap-1 text-center sm:text-start">
					<Card.Title class="flex items-center gap-2">
						<TrendingUp class="size-5" />
						{chartMetricConfig[selectedChartMetric].label} Over Time
					</Card.Title>
					<Card.Description>
						Click on a stat card below to change what's displayed • {chartData.length} days ({rawTimeSeriesData.length} games)
					</Card.Description>
				</div>
			</Card.Header>
			<Card.Content class="pt-4">
				{#if chartLoading}
					<div class="flex items-center justify-center h-[250px]">
						<Loader2 class="size-8 animate-spin text-muted-foreground" />
					</div>
				{:else if chartData.length === 0}
					<div class="flex items-center justify-center h-[250px] text-muted-foreground">
						No data available for chart
					</div>
				{:else}
					<Chart.Container config={currentChartConfig} class="aspect-auto h-[250px] w-full">
						<AreaChart
							data={chartData}
							x="date"
							xScale={scaleUtc()}
							series={[
								{
									key: "value",
									label: chartMetricConfig[selectedChartMetric].label,
									color: chartMetricConfig[selectedChartMetric].color,
								},
							]}
							props={{
								area: {
									curve: curveMonotoneX,
									"fill-opacity": 0.4,
									line: { class: "stroke-2" },
									motion: "tween",
								},
								xAxis: {
									format: (v: Date) => v.toLocaleDateString("en-US", { month: "short", day: "numeric" }),
								},
								yAxis: { 
									format: (v: number) => chartMetricConfig[selectedChartMetric].format(v)
								},
							}}
						>
							{#snippet marks({ series, getAreaProps })}
								<defs>
									<linearGradient id="fillValue" x1="0" y1="0" x2="0" y2="1">
										<stop offset="5%" stop-color={chartMetricConfig[selectedChartMetric].color} stop-opacity={0.8} />
										<stop offset="95%" stop-color={chartMetricConfig[selectedChartMetric].color} stop-opacity={0.1} />
									</linearGradient>
								</defs>
								<ChartClipPath
									initialWidth={0}
									motion={{
										width: { type: "tween", duration: 800, easing: cubicInOut },
									}}
								>
									{#each series as s, i (s.key)}
										<Area {...getAreaProps(s, i)} fill="url(#fillValue)" />
									{/each}
								</ChartClipPath>
							{/snippet}
							{#snippet tooltip()}
								<Chart.Tooltip
									labelFormatter={(v: Date) => v.toLocaleDateString("en-US", { month: "long", day: "numeric", year: "numeric" })}
								/>
							{/snippet}
						</AreaChart>
					</Chart.Container>
				{/if}
			</Card.Content>
		</Card.Root>

		<!-- Overview Cards (clickable to change chart) -->
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
			<!-- Total Games / Win Rate -->
			<button onclick={() => selectChartMetric("winrate")} class="text-left">
				<Card.Root class={`transition-all cursor-pointer hover:border-primary/50 ${selectedChartMetric === "winrate" ? "ring-2 ring-primary border-primary" : ""}`}>
					<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
						<Card.Title class="text-sm font-medium">Total Games</Card.Title>
					</Card.Header>
					<Card.Content>
						<div class="text-2xl font-bold">{stats.totalGames}</div>
						<p class="text-xs text-muted-foreground">
							{stats.totalWins} wins ({getWinRate(stats.totalWins, stats.totalGames)})
						</p>
					</Card.Content>
				</Card.Root>
			</button>
			
			<!-- L-Cancel -->
			<button onclick={() => selectChartMetric("lcancel")} class="text-left">
				<Card.Root class={`transition-all cursor-pointer hover:border-primary/50 ${selectedChartMetric === "lcancel" ? "ring-2 ring-primary border-primary" : ""}`}>
					<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
						<Card.Title class="text-sm font-medium">Avg L-Cancel</Card.Title>
					</Card.Header>
					<Card.Content>
						<div class="text-2xl font-bold">{formatDecimal(stats.avgLCancelPercent)}%</div>
						<p class="text-xs text-muted-foreground">Success rate</p>
					</Card.Content>
				</Card.Root>
			</button>
			
			<!-- Neutral Wins -->
			<button onclick={() => selectChartMetric("neutral")} class="text-left">
				<Card.Root class={`transition-all cursor-pointer hover:border-primary/50 ${selectedChartMetric === "neutral" ? "ring-2 ring-primary border-primary" : ""}`}>
					<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
						<Card.Title class="text-sm font-medium">Neutral Wins</Card.Title>
					</Card.Header>
					<Card.Content>
						<div class="text-2xl font-bold">{formatDecimal(stats.avgNeutralWins)}%</div>
						<p class="text-xs text-muted-foreground">Win rate</p>
					</Card.Content>
				</Card.Root>
			</button>

			<!-- APM -->
			<button onclick={() => selectChartMetric("apm")} class="text-left">
				<Card.Root class={`transition-all cursor-pointer hover:border-primary/50 ${selectedChartMetric === "apm" ? "ring-2 ring-primary border-primary" : ""}`}>
					<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
						<Card.Title class="text-sm font-medium">Inputs / Min</Card.Title>
					</Card.Header>
					<Card.Content>
						<div class="text-2xl font-bold">{formatDecimal(stats.avgInputsPerMinute, 0)}</div>
						<p class="text-xs text-muted-foreground">APM</p>
					</Card.Content>
				</Card.Root>
			</button>

			<!-- Openings / Kill -->
			<button onclick={() => selectChartMetric("openings")} class="text-left">
				<Card.Root class={`transition-all cursor-pointer hover:border-primary/50 ${selectedChartMetric === "openings" ? "ring-2 ring-primary border-primary" : ""}`}>
					<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
						<Card.Title class="text-sm font-medium">Openings / Kill</Card.Title>
					</Card.Header>
					<Card.Content>
						<div class="text-2xl font-bold">{formatDecimal(stats.avgOpeningsPerKill)}</div>
						<p class="text-xs text-muted-foreground">Lower is better</p>
					</Card.Content>
				</Card.Root>
			</button>

			<!-- Damage / Opening -->
			<button onclick={() => selectChartMetric("damage")} class="text-left">
				<Card.Root class={`transition-all cursor-pointer hover:border-primary/50 ${selectedChartMetric === "damage" ? "ring-2 ring-primary border-primary" : ""}`}>
					<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
						<Card.Title class="text-sm font-medium">Damage / Opening</Card.Title>
					</Card.Header>
					<Card.Content>
						<div class="text-2xl font-bold">{formatDecimal(stats.avgDamagePerOpening)}%</div>
						<p class="text-xs text-muted-foreground">Punish game</p>
					</Card.Content>
				</Card.Root>
			</button>
			
			<!-- Rolls / Game -->
			<button onclick={() => selectChartMetric("rolls")} class="text-left">
				<Card.Root class={`transition-all cursor-pointer hover:border-primary/50 ${selectedChartMetric === "rolls" ? "ring-2 ring-primary border-primary" : ""}`}>
					<Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
						<Card.Title class="text-sm font-medium">Avg Rolls/Game</Card.Title>
					</Card.Header>
					<Card.Content>
						<div class="text-2xl font-bold">{formatDecimal(stats.avgRollsPerGame)}</div>
						<p class="text-xs text-muted-foreground">Defensive habits</p>
					</Card.Content>
				</Card.Root>
			</button>
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
