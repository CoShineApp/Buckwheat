<script lang="ts">
	import AppLayout from "$lib/components/AppLayout.svelte";
	import AppSettings from "$lib/components/AppSettings.svelte";
	import { navigation } from "$lib/stores/navigation.svelte";
</script>

<AppLayout>
	{#if navigation.currentPage === "home"}
		<div class="flex min-h-screen items-center justify-center bg-background text-foreground">
			<h1 class="text-4xl font-bold">Home Page</h1>
		</div>
	{:else if navigation.currentPage === "settings"}
		<AppSettings />
	{/if}
</AppLayout>
