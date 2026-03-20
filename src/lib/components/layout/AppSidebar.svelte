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
	import { Home, Settings, Moon, Sun, Circle, Cloud, Scissors, BarChart } from "@lucide/svelte";
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
