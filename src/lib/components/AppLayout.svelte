<script lang="ts">
	import {
		SidebarInset,
		SidebarProvider,
		SidebarTrigger,
	} from "$lib/components/ui/sidebar";
	import { Circle, LogIn, User, Square, Loader2 } from "@lucide/svelte";
	import type { Snippet } from "svelte";
	import { navigation } from "$lib/stores/navigation.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { recording } from "$lib/stores/recording.svelte";
	import { auth } from "$lib/stores/auth.svelte";
	import AuthModal from "$lib/components/auth/AuthModal.svelte";
	import UpdateNotification from "$lib/components/UpdateNotification.svelte";
	import AppSidebar from "$lib/components/layout/AppSidebar.svelte";
	import { Button } from "$lib/components/ui/button";
	import { onMount, onDestroy } from "svelte";
	import { checkGameWindow, listGameWindows, getGameProcessName, setGameProcessName } from "$lib/commands";
	import { invoke } from "@tauri-apps/api/core";
	import { recordingsStore } from "$lib/stores/recordings.svelte";
	import { scoreWindow } from "$lib/utils/window-scoring";

	let sidebarOpen = $state(true);
	let { children }: { children?: Snippet } = $props();
	let pollingInterval: number | undefined;
	let showAuthModal = $state(false);

	// Initialize settings and start game window polling
	onMount(async () => {
		console.log("AppLayout initializing...");
		await settings.init();
		console.log("Settings initialized");

		// Auto-detect and set default window if none is configured
		const currentProcessName = await getGameProcessName();
		if (!currentProcessName) {
			try {
				const windows = await listGameWindows();
				if (windows.length > 0) {
					const scoredWindows = windows.map((w) => ({
						window: w,
						score: scoreWindow(w),
					}));

					scoredWindows.sort((a, b) => b.score - a.score);

					const bestMatch = scoredWindows[0];
					if (bestMatch.score > 0) {
						const identifier = `${bestMatch.window.window_title} (PID: ${bestMatch.window.process_id})`;
						await setGameProcessName(identifier);
						console.log("Auto-detected game window:", identifier);
					}
				}
			} catch (error) {
				console.error("Failed to auto-detect game window:", error);
			}
		}

		// Start watching for .slp files if enabled
		if (settings.watchForGames) {
			console.log("watchForGames is enabled, starting file watcher");
			try {
				let slippiPath = settings.slippiPath;
				if (!slippiPath) {
					console.log("No Slippi path configured, using default");
					slippiPath = await invoke<string>("get_default_slippi_path");
					console.log("Default Slippi path:", slippiPath);
				}

				console.log("Starting file watcher for path:", slippiPath);
				await invoke("start_watching", { path: slippiPath });
				console.log("File watcher started successfully");
			} catch (error) {
				console.error("Failed to start file watcher:", error);
			}
		} else {
			console.log("watchForGames is disabled, skipping file watcher");
		}

		// Check game window immediately
		const windowDetected = await checkGameWindow();
		recording.setGameWindow(windowDetected);

		console.log("Game window detected:", windowDetected);
		console.log("Polling interval:", pollingInterval);

		// Poll for game window every 2 seconds
		pollingInterval = window.setInterval(async () => {
			console.log("Polling for game window...");
			const detected = await checkGameWindow();
			recording.setGameWindow(detected);
		}, 2000);

		console.log("AppLayout initialization complete");
	});

	// Clean up polling interval on unmount
	onDestroy(() => {
		if (pollingInterval) {
			clearInterval(pollingInterval);
		}
	});

	// Reactive theme application
	$effect(() => {
		if (typeof window !== "undefined") {
			const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
			const shouldBeDark =
				settings.theme === "dark" ||
				(settings.theme === "system" && prefersDark);

			if (shouldBeDark) {
				document.documentElement.classList.add("dark");
			} else {
				document.documentElement.classList.remove("dark");
			}
		}
	});
</script>

<SidebarProvider bind:open={sidebarOpen}>
	<AppSidebar bind:sidebarOpen />
	<SidebarInset class="flex flex-col bg-background">
		<header class="flex h-16 shrink-0 items-center gap-2 border-b border-border/60 bg-sidebar px-4">
			<SidebarTrigger class="-ml-1" />
			<div class="h-4 w-px bg-sidebar-border"></div>
			<div class="flex flex-1 items-center justify-between gap-2">
				<h1 class="text-lg font-semibold text-sidebar-foreground">Peppi</h1>
				<div class="flex items-center gap-2">
					<Button
						size="sm"
						variant={recording.isRecording ? "destructive" : "default"}
						class="flex items-center gap-2"
						onclick={() =>
							recording.isRecording
								? recordingsStore.stopManualRecording()
								: recordingsStore.startManualRecording()
						}
						disabled={recording.isRecording ? recordingsStore.isManualStopping : recordingsStore.isManualStarting}
					>
						{#if recording.isRecording}
							{#if recordingsStore.isManualStopping}
								<Loader2 class="size-4 animate-spin" />
							{:else}
								<Square class="size-4" />
							{/if}
							Stop
						{:else}
							{#if recordingsStore.isManualStarting}
								<Loader2 class="size-4 animate-spin" />
							{:else}
								<Circle class="size-4 text-red-500" />
							{/if}
							Record
						{/if}
					</Button>
					{#if auth.isAuthenticated && auth.user}
						<Button variant="ghost" size="sm" onclick={() => navigation.navigateTo("profile")}>
							<User class="size-4 mr-2" />
							{auth.user.email}
						</Button>
					{:else}
						<Button variant="ghost" size="sm" onclick={() => showAuthModal = true}>
							<LogIn class="size-4 mr-2" />
							Log In
						</Button>
					{/if}
				</div>
			</div>
		</header>
		<div class="flex flex-1 flex-col gap-4 bg-background p-4 text-foreground">
			{@render children?.()}
		</div>
	</SidebarInset>
</SidebarProvider>

<AuthModal bind:open={showAuthModal} />
<UpdateNotification />
