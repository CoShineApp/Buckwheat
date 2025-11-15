<script lang="ts">
	import AppLayout from "$lib/components/AppLayout.svelte";
	import AppHome from "$lib/components/AppHome.svelte";
	import AppSettings from "$lib/components/AppSettings.svelte";
	import ReplayViewer from "$lib/components/replay/ReplayViewer.svelte";
	import CloudVideos from "$lib/components/cloud/CloudVideos.svelte";
	import CloudUpload from "$lib/components/cloud/CloudUpload.svelte";
	import UserProfile from "$lib/components/auth/UserProfile.svelte";
	import ClipsView from "$lib/components/clips/ClipsView.svelte";
	import { Toaster } from "$lib/components/ui/sonner";
	import { navigation } from "$lib/stores/navigation.svelte";
</script>

<Toaster />

<AppLayout>
	{#if navigation.currentPage === "home"}
		<AppHome />
	{:else if navigation.currentPage === "settings"}
		<AppSettings />
	{:else if navigation.currentPage === "replay" && navigation.replayId}
		<ReplayViewer recordingId={navigation.replayId} isClip={navigation.isClipReplay} />
	{:else if navigation.currentPage === "cloud"}
		<div class="space-y-4">
			<h2 class="text-2xl font-bold">Cloud Storage</h2>
			<CloudUpload />
			<CloudVideos />
		</div>
	{:else if navigation.currentPage === "profile"}
		<div class="space-y-4">
			<h2 class="text-2xl font-bold">Profile</h2>
			<UserProfile />
		</div>
	{:else if navigation.currentPage === "clips"}
		<ClipsView />
	{/if}
</AppLayout>
