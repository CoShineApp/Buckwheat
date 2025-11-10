<script lang="ts">
	import { settings } from "$lib/stores/settings.svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import { invoke } from "@tauri-apps/api/core";
	import { Button } from "$lib/components/ui/button";
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card";
	import { InputGroup, InputGroupInput, InputGroupButton } from "$lib/components/ui/input-group";
	import { Label } from "$lib/components/ui/label";
	import { Switch } from "$lib/components/ui/switch";
	import { Separator } from "$lib/components/ui/separator";
	import HotkeySelector from "$lib/components/hotkey/HotkeySelector.svelte";
	import { Folder, Gamepad2, Keyboard, Palette, FolderOpen, Database } from "@lucide/svelte";
	import { onMount } from "svelte";

	let settingsPath = $state<string>("");

	onMount(async () => {
		try {
			settingsPath = await invoke<string>("get_settings_path");
		} catch (error) {
			console.error("Failed to get settings path:", error);
		}
	});

	async function handleReset(): Promise<void> {
		if (confirm("Are you sure you want to reset all settings to default?")) {
			await settings.reset();
		}
	}

	async function selectRecordingPath(): Promise<void> {
		const selected = await open({
			directory: true,
			multiple: false,
			title: "Select Recording Output Folder",
		});
		
		if (selected && typeof selected === "string") {
			await settings.set("recordingPath", selected);
		}
	}

	async function selectSlippiPath(): Promise<void> {
		const selected = await open({
			directory: true,
			multiple: false,
			title: "Select Slippi Folder",
		});
		
		if (selected && typeof selected === "string") {
			await settings.set("slippiPath", selected);
		}
	}

	async function openSettingsFolder(): Promise<void> {
		try {
			await invoke("open_settings_folder");
		} catch (error) {
			console.error("Failed to open settings folder:", error);
		}
	}
</script>

<div class="container mx-auto max-w-4xl space-y-6 p-6">
	<div class="space-y-2">
		<h1 class="text-3xl font-bold">Settings</h1>
		<p class="text-muted-foreground">Configure your recording preferences and application settings</p>
	</div>

	{#if settings.isLoading}
		<div class="flex items-center justify-center py-12">
			<p class="text-muted-foreground">Loading settings...</p>
		</div>
	{:else}
		<!-- Appearance Settings -->
		<Card>
			<CardHeader>
				<div class="flex items-center gap-2">
					<Palette class="size-5" />
					<CardTitle>Appearance</CardTitle>
				</div>
				<CardDescription>Customize how the app looks</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="flex items-center justify-between">
					<div class="space-y-0.5">
						<Label>Theme</Label>
						<p class="text-sm text-muted-foreground">Currently: {settings.theme}</p>
					</div>
					<div class="flex gap-2">
						<Button 
							variant={settings.theme === "light" ? "default" : "outline"} 
							size="sm"
							onclick={() => settings.set("theme", "light")}
						>
							Light
						</Button>
						<Button 
							variant={settings.theme === "dark" ? "default" : "outline"} 
							size="sm"
							onclick={() => settings.set("theme", "dark")}
						>
							Dark
						</Button>
						<Button 
							variant={settings.theme === "system" ? "default" : "outline"} 
							size="sm"
							onclick={() => settings.set("theme", "system")}
						>
							System
						</Button>
					</div>
				</div>
			</CardContent>
		</Card>

		<!-- Recording Settings -->
		<Card>
			<CardHeader>
				<div class="flex items-center gap-2">
					<Gamepad2 class="size-5" />
					<CardTitle>Recording</CardTitle>
				</div>
				<CardDescription>Configure recording behavior and quality</CardDescription>
			</CardHeader>
			<CardContent class="space-y-6">
				<div class="space-y-2">
					<Label for="recording-path">Recording Output Path</Label>
					<InputGroup>
						<InputGroupInput
							id="recording-path"
							type="text"
							placeholder="/path/to/recordings"
							value={settings.recordingPath}
							oninput={(e) => settings.set("recordingPath", e.currentTarget.value)}
						/>
						<InputGroupButton onclick={selectRecordingPath}>
							<Folder class="size-4" />
						</InputGroupButton>
					</InputGroup>
					<p class="text-xs text-muted-foreground">Where recorded videos will be saved</p>
				</div>

				<Separator />

				<div class="space-y-2">
					<Label>Recording Quality</Label>
					<div class="flex gap-2">
						<Button 
							variant={settings.recordingQuality === "low" ? "default" : "outline"} 
							size="sm"
							onclick={() => settings.set("recordingQuality", "low")}
						>
							Low
						</Button>
						<Button 
							variant={settings.recordingQuality === "medium" ? "default" : "outline"} 
							size="sm"
							onclick={() => settings.set("recordingQuality", "medium")}
						>
							Medium
						</Button>
						<Button 
							variant={settings.recordingQuality === "high" ? "default" : "outline"} 
							size="sm"
							onclick={() => settings.set("recordingQuality", "high")}
						>
							High
						</Button>
						<Button 
							variant={settings.recordingQuality === "ultra" ? "default" : "outline"} 
							size="sm"
							onclick={() => settings.set("recordingQuality", "ultra")}
						>
							Ultra
						</Button>
					</div>
				</div>

				<Separator />

				<div class="flex items-center justify-between">
					<div class="space-y-0.5">
						<Label for="auto-start">Auto-start Recording</Label>
						<p class="text-sm text-muted-foreground">Automatically start recording when a game is detected</p>
					</div>
					<Switch
						id="auto-start"
						checked={settings.autoStartRecording}
						onCheckedChange={(checked) => settings.set("autoStartRecording", checked)}
					/>
				</div>
			</CardContent>
		</Card>

		<!-- Slippi Settings -->
		<Card>
			<CardHeader>
				<div class="flex items-center gap-2">
					<Folder class="size-5" />
					<CardTitle>Slippi</CardTitle>
				</div>
				<CardDescription>Configure Slippi integration</CardDescription>
			</CardHeader>
			<CardContent class="space-y-6">
				<div class="space-y-2">
					<Label for="slippi-path">Slippi Directory</Label>
					<InputGroup>
						<InputGroupInput
							id="slippi-path"
							type="text"
							placeholder="/path/to/slippi"
							value={settings.slippiPath}
							oninput={(e) => settings.set("slippiPath", e.currentTarget.value)}
						/>
						<InputGroupButton onclick={selectSlippiPath}>
							<Folder class="size-4" />
						</InputGroupButton>
					</InputGroup>
					<p class="text-xs text-muted-foreground">Location of your Slippi replays folder</p>
				</div>

				<Separator />

				<div class="flex items-center justify-between">
					<div class="space-y-0.5">
						<Label for="watch-games">Watch for Games</Label>
						<p class="text-sm text-muted-foreground">Monitor Slippi folder for new games</p>
					</div>
					<Switch
						id="watch-games"
						checked={settings.watchForGames}
						onCheckedChange={(checked) => settings.set("watchForGames", checked)}
					/>
				</div>
			</CardContent>
		</Card>

		<!-- Hotkeys Settings -->
		<Card>
			<CardHeader>
				<div class="flex items-center gap-2">
					<Keyboard class="size-5" />
					<CardTitle>Hotkeys</CardTitle>
				</div>
				<CardDescription>Configure keyboard shortcuts</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="space-y-2">
					<Label for="start-hotkey">Start Recording</Label>
					<HotkeySelector
						bind:value={settings.startRecordingHotkey}
						placeholder="Press a key combination..."
						onchange={(value) => settings.set("startRecordingHotkey", value)}
					/>
					<p class="text-xs text-muted-foreground">Click and press a key combination to set hotkey</p>
				</div>

				<div class="space-y-2">
					<Label for="stop-hotkey">Stop Recording</Label>
					<HotkeySelector
						bind:value={settings.stopRecordingHotkey}
						placeholder="Press a key combination..."
						onchange={(value) => settings.set("stopRecordingHotkey", value)}
					/>
					<p class="text-xs text-muted-foreground">Click and press a key combination to set hotkey</p>
				</div>
			</CardContent>
		</Card>

		<!-- Settings Storage -->
		<Card>
			<CardHeader>
				<div class="flex items-center gap-2">
					<Database class="size-5" />
					<CardTitle>Settings Storage</CardTitle>
				</div>
				<CardDescription>Manage where your settings are stored</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="space-y-2">
					<Label>Settings File Location</Label>
					<InputGroup>
						<InputGroupInput
							type="text"
							readonly
							value={settingsPath}
							placeholder="Loading..."
						/>
						<InputGroupButton onclick={openSettingsFolder}>
							<FolderOpen class="size-4" />
						</InputGroupButton>
					</InputGroup>
					<p class="text-xs text-muted-foreground">Click the folder icon to open the settings directory</p>
				</div>
			</CardContent>
		</Card>

		<!-- Reset Section -->
		<Card>
			<CardHeader>
				<CardTitle>Danger Zone</CardTitle>
				<CardDescription>Reset all settings to their default values</CardDescription>
			</CardHeader>
			<CardContent>
				<Button variant="destructive" onclick={handleReset}>
					Reset All Settings
				</Button>
			</CardContent>
		</Card>
	{/if}
</div>

