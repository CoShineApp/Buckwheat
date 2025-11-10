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
	import { Home, Settings, Moon, Sun, Circle } from "@lucide/svelte";
	import type { Snippet } from "svelte";
	import { navigation } from "$lib/stores/navigation.svelte";
	import { settings } from "$lib/stores/settings.svelte";
	import { recording } from "$lib/stores/recording.svelte";
	import { onMount } from "svelte";

	let sidebarOpen = $state(true);
	let { children }: { children?: Snippet } = $props();

	// Initialize settings on mount
	onMount(async () => {
		await settings.init();
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
						{#if recording.isRecording}
							<div class="flex items-center gap-1.5">
								<Circle class="size-2 animate-pulse fill-red-500 text-red-500" />
								<span class="text-xs font-medium text-red-500">LIVE</span>
							</div>
						{/if}
					</SidebarMenuButton>
				</SidebarMenuItem>
			</SidebarMenu>
		</SidebarHeader>
		<SidebarContent>
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
	<SidebarInset class="bg-background">
		<header class="flex h-16 shrink-0 items-center gap-2 border-b bg-sidebar px-4">
			<SidebarTrigger class="-ml-1" />
			<div class="h-4 w-px bg-sidebar-border"></div>
			<div class="flex flex-1 items-center gap-2">
				<h1 class="text-lg font-semibold text-sidebar-foreground">Peppi</h1>
			</div>
		</header>
		<div class="flex flex-1 flex-col gap-4 bg-background p-4 text-foreground">
			{@render children?.()}
		</div>
	</SidebarInset>
</SidebarProvider>

