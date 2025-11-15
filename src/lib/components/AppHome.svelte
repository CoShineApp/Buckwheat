<script lang="ts">
import RecordingStats from "$lib/components/recordings/RecordingStats.svelte";
import RecordingsTable from "$lib/components/recordings/RecordingsTable.svelte";
import BatchActions from "$lib/components/recordings/BatchActions.svelte";
import StorageUsageBar from "$lib/components/cloud/StorageUsageBar.svelte";
import { onMount } from "svelte";
import { recordingsStore } from "$lib/stores/recordings.svelte";

onMount(() => {
	const dispose = recordingsStore.bootstrap();
	return () => dispose();
});
</script>

<div class="flex h-full flex-col gap-6 p-6">
	<!-- Cloud Storage Usage -->
	<StorageUsageBar />

	<!-- Stats Dashboard -->
	<RecordingStats />

	<!-- Recordings Table -->
	<RecordingsTable />

	<!-- Batch Actions Toolbar (floats at bottom when items selected) -->
	<BatchActions />
</div>

