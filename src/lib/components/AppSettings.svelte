<script lang="ts">
	import { settings } from "$lib/stores/settings.svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { onMount } from "svelte";
	import GeneralSettings from "$lib/components/settings/GeneralSettings.svelte";
	import SlippiSettings from "$lib/components/settings/SlippiSettings.svelte";
	import RecordingSettings from "$lib/components/settings/RecordingSettings.svelte";
	import { Button } from "$lib/components/ui/button";
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card";
	import { InputGroup, InputGroupInput, InputGroupButton } from "$lib/components/ui/input-group";
	import { Label } from "$lib/components/ui/label";
	import { Keyboard, FolderOpen, Database } from "@lucide/svelte";
	import HotkeySelector from "$lib/components/hotkey/HotkeySelector.svelte";

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
		<GeneralSettings />

		<RecordingSettings />

		<SlippiSettings />

		<!-- Clips Settings -->
		<Card>
			<CardHeader>
				<div class="flex items-center gap-2">
					<Keyboard class="size-5" />
					<CardTitle>Clips</CardTitle>
				</div>
				<CardDescription>Configure clip creation settings</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="space-y-2">
					<Label for="clip-hotkey">Create Clip Hotkey</Label>
					<HotkeySelector
						bind:value={settings.createClipHotkey}
						placeholder="Press a key combination..."
						onchange={(value) => settings.set("createClipHotkey", value)}
					/>
					<p class="text-xs text-muted-foreground">
						Press this hotkey during a recording to mark a clip
					</p>
				</div>

				<div class="space-y-2">
					<Label for="clip-duration">
						Clip Duration: {settings.clipDuration} seconds
					</Label>
					<input
						type="range"
						id="clip-duration"
						min="5"
						max="60"
						step="5"
						bind:value={settings.clipDuration}
						onchange={() => settings.set("clipDuration", settings.clipDuration)}
						class="w-full h-2 bg-secondary rounded-lg appearance-none cursor-pointer"
					/>
					<p class="text-xs text-muted-foreground">
						Capture the last {settings.clipDuration} seconds when creating a clip (5-60 seconds)
					</p>
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
