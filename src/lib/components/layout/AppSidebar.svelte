<script lang="ts">
	import {
		Sidebar,
		SidebarContent,
		SidebarFooter,
		SidebarGroup,
		SidebarGroupContent,
		SidebarGroupLabel,
		SidebarHeader,
		SidebarMenu,
		SidebarMenuButton,
		SidebarMenuItem,
	} from "$lib/components/ui/sidebar";
	import { Home, Settings, Moon, Sun, Circle, Cloud, Scissors, BarChart, HelpCircle, Copy, Check } from "@lucide/svelte";

	let copiedPath = $state<string | null>(null);

	const dolphinIniMac = "~/Library/Application Support/Slippi Launcher/playback/User/Config/Dolphin.ini";
	const dolphinIniWin = "%AppData%\\Slippi Launcher\\playback\\User\\Config\\Dolphin.ini";

	async function copyPath(path: string): Promise<void> {
		await navigator.clipboard.writeText(path);
		copiedPath = path;
		setTimeout(() => {
			copiedPath = null;
		}, 2000);
	}
	import {
		Dialog,
		DialogContent,
		DialogHeader,
		DialogTitle,
		DialogDescription,
		DialogTrigger,
	} from "$lib/components/ui/dialog";
	import { navigation } from "$lib/stores/navigation.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { recording } from "$lib/stores/recording.svelte";
	import { auth } from "$lib/stores/auth.svelte";

	let { sidebarOpen = $bindable(true) }: { sidebarOpen?: boolean } = $props();

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
				pulse: true,
			},
			ready: {
				bg: "bg-yellow-500/10",
				text: "text-yellow-600 dark:text-yellow-400",
				circle: "fill-yellow-500 text-yellow-500",
				label: "Game Window Found",
				pulse: false,
			},
			waiting: {
				bg: "bg-yellow-500/10",
				text: "text-yellow-600 dark:text-yellow-400",
				circle: "fill-yellow-500 text-yellow-500",
				label: "Waiting for Game",
				pulse: false,
			},
			"no-window": {
				bg: "bg-red-500/10",
				text: "text-red-600 dark:text-red-400",
				circle: "fill-red-500 text-red-500",
				label: "No Game Window",
				pulse: false,
			},
		};
		return configs[recording.status] || configs["no-window"];
	});
</script>

<Sidebar collapsible="icon">
	<SidebarHeader>
		<SidebarMenu>
			<SidebarMenuItem>
				<SidebarMenuButton size="lg">
					<div class="flex aspect-square size-8 items-center justify-center rounded-lg bg-gradient-to-br from-emerald-500 to-green-600 text-white shadow-md shadow-emerald-500/20">
						<Home class="size-4" />
					</div>
					<div class="grid flex-1 text-left text-sm leading-tight">
						<span class="truncate font-semibold">Peppi</span>
						<span class="truncate text-xs text-muted-foreground">Slippi Recorder</span>
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
							tooltipContent="Total Stats"
							onclick={() => navigation.navigateTo("total_stats")}
							isActive={navigation.currentPage === "total_stats"}
						>
							<BarChart />
							<span>Total Stats</span>
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
				</SidebarMenu>
			</SidebarGroupContent>
		</SidebarGroup>
	</SidebarContent>
	<SidebarFooter>
		<SidebarMenu>
			<SidebarMenuItem>
				<Dialog>
					<DialogTrigger>
						<SidebarMenuButton tooltipContent="Setup Guide">
							<HelpCircle />
							<span>Setup Guide</span>
						</SidebarMenuButton>
					</DialogTrigger>
					<DialogContent class="max-w-2xl max-h-[85vh] overflow-y-auto">
						<DialogHeader>
							<DialogTitle>OBS Recording Setup Guide</DialogTitle>
							<DialogDescription>
								How to configure OBS to record Slippi Melee gameplay
							</DialogDescription>
						</DialogHeader>
						<div class="space-y-6 text-sm">
							<!-- Step 1 -->
							<div class="space-y-2">
								<h3 class="font-semibold text-base">1. Install & Open OBS</h3>
								<p class="text-muted-foreground">
									If you don't have OBS installed, Peppi can install it for you (Settings &gt; OBS Integration &gt; Automatic mode). Otherwise, <a href="https://obsproject.com" target="_blank" class="underline text-primary">download OBS Studio</a> and open it.
								</p>
							</div>

							<!-- Step 2 -->
							<div class="space-y-2">
								<h3 class="font-semibold text-base">2. Enable WebSocket Server</h3>
								<p class="text-muted-foreground">
									This lets Peppi control OBS to start/stop recordings automatically.
								</p>
								<ol class="list-decimal list-inside space-y-1 text-muted-foreground ml-2">
									<li>In OBS, go to <span class="font-medium text-foreground">Tools &gt; WebSocket Server Settings</span></li>
									<li>Check <span class="font-medium text-foreground">"Enable WebSocket server"</span></li>
									<li>Note the port (default: 4455) and password</li>
									<li>Click OK</li>
								</ol>
							</div>

							<!-- Step 3 -->
							<div class="space-y-2">
								<h3 class="font-semibold text-base">3. Add a Window Capture Source</h3>
								<p class="text-muted-foreground">
									This captures just the Dolphin game window — no desktop, no dock.
								</p>
								<ol class="list-decimal list-inside space-y-1 text-muted-foreground ml-2">
									<li>In the <span class="font-medium text-foreground">Sources</span> panel, click <span class="font-medium text-foreground">+</span></li>
									<li>Select <span class="font-medium text-foreground">"Window Capture"</span> (macOS) or <span class="font-medium text-foreground">"Game Capture"</span> (Windows)</li>
									<li>Click OK on the name dialog</li>
									<li>Select your <span class="font-medium text-foreground">Dolphin/Slippi window</span> from the dropdown</li>
									<li>Click OK</li>
								</ol>
								<p class="text-xs text-muted-foreground italic">
									Tip: If Dolphin is fullscreen, press Esc to window it first so OBS can see it in the list, then go back to fullscreen.
								</p>
							</div>

							<!-- Step 4 -->
							<div class="space-y-2">
								<h3 class="font-semibold text-base">4. Optimal Dolphin Settings (Optional)</h3>
								<p class="text-muted-foreground">
									For the best recording quality with native Melee aspect ratio (73:60):
								</p>
								<ol class="list-decimal list-inside space-y-1 text-muted-foreground ml-2">
									<li>Open Dolphin <span class="font-medium text-foreground">Graphics Settings</span></li>
									<li>Set Aspect Ratio to <span class="font-medium text-foreground">"Force 73:60 (Melee)"</span></li>
									<li>Uncheck <span class="font-medium text-foreground">"Render to Main Window"</span> — this gives OBS a clean render window without Dolphin's toolbar</li>
									<li>Check <span class="font-medium text-foreground">"Hide Mouse Cursor"</span></li>
								</ol>
								<div class="rounded-md bg-muted p-3 mt-2 space-y-3">
									<p class="text-xs font-medium">Render Window Size (Dolphin.ini)</p>
									<p class="text-xs text-muted-foreground">
										For pixel-perfect output, edit <code class="bg-background px-1 rounded">RenderWindowWidth</code> and <code class="bg-background px-1 rounded">RenderWindowHeight</code> in your Dolphin.ini.
									</p>

									<div class="space-y-1.5">
										<p class="text-xs font-medium">macOS</p>
										<button
											class="flex w-full items-center gap-2 rounded border bg-background px-2 py-1.5 text-left text-xs font-mono text-muted-foreground hover:bg-accent transition-colors"
											onclick={() => copyPath(dolphinIniMac)}
										>
											<span class="flex-1 truncate">{dolphinIniMac}</span>
											{#if copiedPath === dolphinIniMac}
												<Check class="size-3 shrink-0 text-green-500" />
											{:else}
												<Copy class="size-3 shrink-0" />
											{/if}
										</button>
									</div>

									<div class="space-y-1.5">
										<p class="text-xs font-medium">Windows</p>
										<button
											class="flex w-full items-center gap-2 rounded border bg-background px-2 py-1.5 text-left text-xs font-mono text-muted-foreground hover:bg-accent transition-colors"
											onclick={() => copyPath(dolphinIniWin)}
										>
											<span class="flex-1 truncate">{dolphinIniWin}</span>
											{#if copiedPath === dolphinIniWin}
												<Check class="size-3 shrink-0 text-green-500" />
											{:else}
												<Copy class="size-3 shrink-0" />
											{/if}
										</button>
									</div>

									<ul class="text-xs text-muted-foreground space-y-0.5 ml-2">
										<li><span class="font-medium text-foreground">1080p display:</span> 1176 x 968</li>
										<li><span class="font-medium text-foreground">1440p+ display:</span> 1312 x 1080</li>
									</ul>
									<p class="text-xs text-muted-foreground">
										Then set OBS Video settings (Settings &gt; Video) base + output resolution to match.
									</p>
								</div>
							</div>

							<!-- Step 5 -->
							<div class="space-y-2">
								<h3 class="font-semibold text-base">5. Connect Peppi to OBS</h3>
								<ol class="list-decimal list-inside space-y-1 text-muted-foreground ml-2">
									<li>In Peppi, go to <span class="font-medium text-foreground">Settings &gt; OBS Integration</span></li>
									<li>Select <span class="font-medium text-foreground">"Connect to my OBS"</span></li>
									<li>Enter the WebSocket port and password</li>
									<li>Click <span class="font-medium text-foreground">"Test Connection"</span></li>
								</ol>
								<p class="text-muted-foreground mt-1">
									Once connected, Peppi will automatically start/stop OBS recordings when it detects new Slippi replay files.
								</p>
							</div>

							<div class="rounded-md border p-3">
								<p class="text-xs text-muted-foreground">
									Based on the guide by <a href="https://realdingoes.medium.com/recording-slippi-replays-with-native-aspect-ratio-8d95dcf6f70b" target="_blank" class="underline text-primary">realdingoes</a>. For replay recording (not live play), unchecking "Render to Main Window" gives the cleanest capture.
								</p>
							</div>
						</div>
					</DialogContent>
				</Dialog>
			</SidebarMenuItem>
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
