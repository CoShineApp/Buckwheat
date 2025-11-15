<script lang="ts">
	import {
		Sidebar,
		SidebarContent,
		SidebarFooter,
		SidebarGroup,
		SidebarGroupContent,
		SidebarGroupLabel,
		SidebarHeader,
		SidebarInset,
		SidebarMenu,
		SidebarMenuButton,
		SidebarMenuItem,
		SidebarProvider,
		SidebarTrigger
	} from "$lib/components/ui/sidebar";
	import { Home, Settings, Moon, Sun, Circle, Cloud, LogIn, User, Scissors, Square, Loader2 } from "@lucide/svelte";
	import type { Snippet } from "svelte";
	import { navigation } from "$lib/stores/navigation.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { recording } from "$lib/stores/recording.svelte";
	import { auth } from "$lib/stores/auth.svelte";
	import AuthModal from "$lib/components/auth/AuthModal.svelte";
	import { Button } from "$lib/components/ui/button";
	import { onMount, onDestroy } from "svelte";
	import { checkGameWindow, listGameWindows, getGameProcessName, setGameProcessName } from "$lib/commands.svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { recordingsStore } from "$lib/stores/recordings.svelte";

	let sidebarOpen = $state(true);
	let { children }: { children?: Snippet } = $props();
	let pollingInterval: number | undefined;
	let showAuthModal = $state(false);
	// Initialize settings and start game window polling
	onMount(async () => {
		console.log("🚀 AppLayout initializing...");
		await settings.init();
		console.log("✅ Settings initialized");
		
		// Auto-detect and set default window if none is configured
		const currentProcessName = await getGameProcessName();
		if (!currentProcessName) {
			try {
				const windows = await listGameWindows();
				if (windows.length > 0) {
					// Score windows to find the best match (Dolphin/Slippi/Melee)
					const scoredWindows = windows.map(w => ({
						window: w,
						score: scoreWindow(w)
					}));
					
					// Sort by score descending
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
			console.log("🎮 watchForGames is enabled, starting file watcher");
			try {
				// Get Slippi path (use configured or default)
				let slippiPath = settings.slippiPath;
				if (!slippiPath) {
					console.log("📁 No Slippi path configured, using default");
					slippiPath = await invoke<string>("get_default_slippi_path");
					console.log("📁 Default Slippi path:", slippiPath);
				}
				
				console.log("🚀 Starting file watcher for path:", slippiPath);
				await invoke("start_watching", { path: slippiPath });
				console.log("✅ File watcher started successfully");
			} catch (error) {
				console.error("❌ Failed to start file watcher:", error);
			}
		} else {
			console.log("⏭️  watchForGames is disabled, skipping file watcher");
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
		
		console.log("✅ AppLayout initialization complete");
	});
	
	// Score a window based on how likely it is to be the game window
	function scoreWindow(window: any): number {
		let score = 0;
		const title = window.window_title.toLowerCase();
		const className = window.class_name.toLowerCase();
		
		// Prioritize by class name (most reliable)
		if (className.includes('d3dproxy')) {
			score += 5000;
		}
		if (className.includes('wxwindownr')) {
			score += 1000;
		}
		
		// Check title for game-related keywords
		if (title.includes('slippi')) {
			score += 500;
		}
		if (title.includes('melee')) {
			score += 500;
		}
		if (title.includes('dolphin')) {
			score += 500;
		}
		if (title.includes('faster melee')) {
			score += 500;
		}
		
		// Prefer larger windows (actual game window vs small utility windows)
		if (window.width >= 640 && window.height >= 480) {
			score += 100;
		}
		
		// Penalize child windows or owned windows (they're often not the main game)
		if (window.is_child) {
			score -= 50;
		}
		if (window.has_owner) {
			score -= 50;
		}
		
		return score;
	}

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

	function toggleTheme(): void {
		const newTheme = settings.theme === "dark" ? "light" : "dark";
		settings.set("theme", newTheme);
	}

	let isDarkMode = $derived.by(() => {
		if (settings.theme === "system") {
			return typeof window !== "undefined" && 
				window.matchMedia("(prefers-color-scheme: dark)").matches;
		}
		return settings.theme === "dark";
	});

	// Status indicator config
	const statusConfig = $derived.by(() => {
		const configs: Record<string, {
			bg: string;
			text: string;
			circle: string;
			label: string;
			pulse: boolean;
		}> = {
			recording: {
				bg: "bg-green-500/10",
				text: "text-green-600 dark:text-green-400",
				circle: "fill-green-500 text-green-500",
				label: "Recording in Progress",
				pulse: true
			},
			ready: {
				bg: "bg-yellow-500/10",
				text: "text-yellow-600 dark:text-yellow-400",
				circle: "fill-yellow-500 text-yellow-500",
				label: "Game Window Found",
				pulse: false
			},
			waiting: {
				bg: "bg-yellow-500/10",
				text: "text-yellow-600 dark:text-yellow-400",
				circle: "fill-yellow-500 text-yellow-500",
				label: "Waiting for Game",
				pulse: false
			},
			"no-window": {
				bg: "bg-red-500/10",
				text: "text-red-600 dark:text-red-400",
				circle: "fill-red-500 text-red-500",
				label: "No Game Window",
				pulse: false
			}
		};
		return configs[recording.status] || configs["no-window"];
	});

</script>

<SidebarProvider bind:open={sidebarOpen}>
	<Sidebar collapsible="icon">
		<SidebarHeader>
			<SidebarMenu>
				<SidebarMenuItem>
					<SidebarMenuButton size="lg">
						<div class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
							<Home class="size-4" />
						</div>
						<div class="grid flex-1 text-left text-sm leading-tight">
							<span class="truncate font-semibold">Peppi</span>
							<span class="truncate text-xs">Slippi Recorder</span>
						</div>
					</SidebarMenuButton>
				</SidebarMenuItem>
			</SidebarMenu>
		</SidebarHeader>
		<SidebarContent>
			<!-- Status Indicator -->
			<SidebarGroup>
				<div class="px-2 pb-2">
					<div class="flex items-center gap-2 rounded-md {statusConfig.bg} {statusConfig.text} px-1 py-1 {statusConfig.pulse ? 'animate-pulse' : ''}">
						<Circle class="size-1 {statusConfig.circle}" />
						{#if sidebarOpen}
							<span class="text-xs font-medium">{statusConfig.label}</span>
						{/if}
					</div>
				</div>
			</SidebarGroup>
			<SidebarGroup>
				<SidebarGroupLabel>Navigation</SidebarGroupLabel>
				<SidebarGroupContent>
					<SidebarMenu>
						<SidebarMenuItem>
							<SidebarMenuButton 
								tooltipContent="Home" 
								onclick={() => navigation.navigateTo("home")}
								isActive={navigation.currentPage === "home"}
							>
								<Home />
								<span>Home</span>
							</SidebarMenuButton>
						</SidebarMenuItem>
						<SidebarMenuItem>
							<SidebarMenuButton 
								tooltipContent="Clips" 
								onclick={() => navigation.navigateTo("clips")}
								isActive={navigation.currentPage === "clips"}
							>
								<Scissors />
								<span>Clips</span>
							</SidebarMenuButton>
						</SidebarMenuItem>
						<SidebarMenuItem>
							<SidebarMenuButton 
								tooltipContent="Settings" 
								onclick={() => navigation.navigateTo("settings")}
								isActive={navigation.currentPage === "settings"}
							>
								<Settings />
								<span>Settings</span>
							</SidebarMenuButton>
						</SidebarMenuItem>
						{#if auth.isAuthenticated}
							<SidebarMenuItem>
								<SidebarMenuButton 
									tooltipContent="Cloud Storage" 
									onclick={() => navigation.navigateTo("cloud")}
									isActive={navigation.currentPage === "cloud"}
								>
									<Cloud />
									<span>Cloud Storage</span>
								</SidebarMenuButton>
							</SidebarMenuItem>
						{/if}
					</SidebarMenu>
				</SidebarGroupContent>
			</SidebarGroup>
		</SidebarContent>
		<SidebarFooter>
			<SidebarMenu>
				<SidebarMenuItem>
					<SidebarMenuButton tooltipContent={isDarkMode ? "Switch to light mode" : "Switch to dark mode"} onclick={toggleTheme}>
						{#if isDarkMode}
							<Sun />
							<span>Light Mode</span>
						{:else}
							<Moon />
							<span>Dark Mode</span>
						{/if}
					</SidebarMenuButton>
				</SidebarMenuItem>
			</SidebarMenu>
		</SidebarFooter>
	</Sidebar>
	<SidebarInset class="flex flex-col bg-background">
		<header class="flex h-16 shrink-0 items-center gap-2 border-b bg-sidebar px-4">
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
