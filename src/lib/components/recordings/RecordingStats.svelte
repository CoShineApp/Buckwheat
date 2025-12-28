<script lang="ts">
	import { Card, CardContent, CardHeader, CardTitle } from "$lib/components/ui/card";
	import { recordingsStore } from "$lib/stores/recordings.svelte";
	import { getCharacterName } from "$lib/utils/characters";
	import { formatFileSize } from "$lib/utils/format";
	import CharacterIcon from "./CharacterIcon.svelte";
	import { Video, HardDrive, Trophy, Clock } from "@lucide/svelte";

	const totalRecordings = $derived(recordingsStore.recordings.length);
	const totalStorage = $derived(recordingsStore.totalStorage);
	const mostPlayedChar = $derived(recordingsStore.mostPlayedCharacter);
	
	// Calculate total recording time
	const totalDuration = $derived(
		recordingsStore.recordings.reduce((total, rec) => total + (rec.duration || 0), 0)
	);

	// Format total duration
	const formatTotalDuration = (seconds: number) => {
		const hours = Math.floor(seconds / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		if (hours > 0) return `${hours}h ${minutes}m`;
		return `${minutes}m`;
	};

	// Get recent activity (recordings in last 24 hours)
	const recentActivity = $derived(() => {
		const oneDayAgo = new Date(Date.now() - 24 * 60 * 60 * 1000);
		return recordingsStore.recordings.filter((rec) => {
			const recDate = new Date(rec.start_time);
			return recDate > oneDayAgo;
		}).length;
	});
</script>

<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
	<!-- Total Recordings -->
	<Card>
		<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
			<CardTitle class="text-sm font-medium">Total Recordings</CardTitle>
			<Video class="size-4 text-muted-foreground" />
		</CardHeader>
		<CardContent>
			<div class="text-2xl font-bold">{totalRecordings}</div>
			<p class="text-xs text-muted-foreground">
				{formatTotalDuration(totalDuration)} of content
			</p>
		</CardContent>
	</Card>

	<!-- Storage Used -->
	<Card>
		<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
			<CardTitle class="text-sm font-medium">Storage Used</CardTitle>
			<HardDrive class="size-4 text-muted-foreground" />
		</CardHeader>
		<CardContent>
			<div class="text-2xl font-bold">{formatFileSize(totalStorage)}</div>
			<p class="text-xs text-muted-foreground">
				{totalRecordings > 0 ? formatFileSize(totalStorage / totalRecordings) : "0 B"} avg per recording
			</p>
		</CardContent>
	</Card>

	<!-- Most Played Character -->
	<Card>
		<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
			<CardTitle class="text-sm font-medium">Most Played</CardTitle>
			<Trophy class="size-4 text-muted-foreground" />
		</CardHeader>
		<CardContent>
			{#if mostPlayedChar >= 0}
				<div class="flex items-center gap-2">
					<CharacterIcon characterId={mostPlayedChar} size="sm" />
					<div class="text-lg font-bold">{getCharacterName(mostPlayedChar)}</div>
				</div>
			{:else}
				<div class="text-2xl font-bold text-muted-foreground">—</div>
				<p class="text-xs text-muted-foreground">No recordings yet</p>
			{/if}
		</CardContent>
	</Card>

	<!-- Recent Activity -->
	<Card>
		<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
			<CardTitle class="text-sm font-medium">Recent Activity</CardTitle>
			<Clock class="size-4 text-muted-foreground" />
		</CardHeader>
		<CardContent>
			<div class="text-2xl font-bold">{recentActivity()}</div>
			<p class="text-xs text-muted-foreground">
				{recentActivity() === 1 ? "recording" : "recordings"} in last 24h
			</p>
		</CardContent>
	</Card>
</div>

