<script lang="ts">
	import AppLayout from "$lib/components/AppLayout.svelte";
	import AppHome from "$lib/components/AppHome.svelte";
	import AppSettings from "$lib/components/AppSettings.svelte";
	import { Toaster } from "$lib/components/ui/sonner";
	import { navigation } from "$lib/stores/navigation.svelte";
</script>

<Toaster />

<AppLayout>
	{#if navigation.currentPage === "home"}
		<AppHome />
	{:else if navigation.currentPage === "settings"}
		<AppSettings />
	{/if}
</AppLayout>
