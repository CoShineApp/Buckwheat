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
	import { Folder, Gamepad2, Keyboard, Palette, FolderOpen, Database, Monitor, RefreshCw } from "@lucide/svelte";
	import { onMount } from "svelte";
	import { listGameWindows, getGameProcessName, setGameProcessName, captureWindowPreview, type GameWindow } from "$lib/commands.svelte";
	import { toast } from "svelte-sonner";

	let settingsPath = $state<string>("");
	let currentProcessName = $state<string | null>(null);
	let detectedWindows = $state<GameWindow[]>([]);
	let isDetecting = $state(false);
	let previewImage = $state<string | null>(null);
	let isCapturingPreview = $state(false);

	onMount(async () => {
		try {
			settingsPath = await invoke<string>("get_settings_path");
			currentProcessName = await getGameProcessName();
		} catch (error) {
			console.error("Failed to get settings path:", error);
		}
	});

	async function detectGameWindows(): Promise<void> {
		isDetecting = true;
		try {
			const windows = await listGameWindows();
			detectedWindows = windows;
			if (windows.length === 0) {
				toast.error("No game windows detected", {
					description: "Make sure Slippi Dolphin is running and try again."
				});
			} else {
				toast.success(`Found ${windows.length} game window(s)`, {
					description: "Select the one you want to use for recording."
				});
			}
		} catch (error) {
			console.error("Failed to detect game windows:", error);
			toast.error("Failed to detect game windows");
		} finally {
			isDetecting = false;
		}
	}

	async function selectGameWindow(window: GameWindow): Promise<void> {
		try {
			// Store window title pattern to uniquely identify this window
			// We'll use the full title to be more specific
			const identifier = `${window.window_title} (PID: ${window.process_id})`;
			
			await setGameProcessName(identifier);
			currentProcessName = identifier;
			toast.success("Game window set", {
				description: `Now using: ${window.window_title}`
			});
			detectedWindows = []; // Clear the list
			
			// Automatically capture preview after selection
			await capturePreview();
		} catch (error) {
			console.error("Failed to set game window:", error);
			toast.error("Failed to save selection");
		}
	}

	async function capturePreview(): Promise<void> {
		isCapturingPreview = true;
		try {
			const data = await captureWindowPreview();
			previewImage = data;
			if (!data) {
				toast.error("Failed to capture preview", {
					description: "Make sure the window is visible."
				});
			}
		} catch (error) {
			console.error("Failed to capture preview:", error);
			toast.error("Failed to capture preview");
		} finally {
			isCapturingPreview = false;
		}
	}

	async function clearGameProcess(): Promise<void> {
		try {
			await setGameProcessName("");
			currentProcessName = null;
			toast.info("Game process cleared", {
				description: "Will use auto-detection"
			});
		} catch (error) {
			console.error("Failed to clear game process:", error);
		}
	}

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

		<!-- Game Window Detection -->
		<Card>
			<CardHeader>
				<div class="flex items-center gap-2">
					<Monitor class="size-5" />
					<CardTitle>Game Window Detection</CardTitle>
				</div>
				<CardDescription>Configure which game window to record</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="space-y-2">
					<Label>Current Game Process</Label>
					<div class="flex items-center gap-2">
						<div class="flex-1 rounded-md border bg-muted px-3 py-2 text-sm">
							{currentProcessName || "Auto-detecting..."}
						</div>
						{#if currentProcessName}
							<Button variant="outline" size="sm" onclick={clearGameProcess}>
								Clear
							</Button>
						{/if}
					</div>
					<p class="text-xs text-muted-foreground">
						{currentProcessName 
							? "Using this specific process for detection and recording" 
							: "Will attempt to auto-detect Slippi Dolphin"}
					</p>
				</div>

				{#if currentProcessName}
					<Separator />
					
					<div class="space-y-2">
						<div class="flex items-center justify-between">
							<Label>Window Preview</Label>
							<Button 
								variant="ghost" 
								size="sm" 
								onclick={capturePreview}
								disabled={isCapturingPreview}
							>
								<RefreshCw class={`size-4 mr-2 ${isCapturingPreview ? 'animate-spin' : ''}`} />
								{isCapturingPreview ? "Capturing..." : "Refresh"}
							</Button>
						</div>
						{#if previewImage}
							<div class="flex items-center justify-center rounded-md border bg-muted p-2">
								<img
									src={`data:image/png;base64,${previewImage}`}
									alt="Game window preview"
									class="max-h-48 w-full rounded-md object-contain"
								/>
							</div>
							<p class="text-xs text-muted-foreground">
								Preview of the selected game window
							</p>
						{:else if isCapturingPreview}
							<div class="flex items-center justify-center rounded-md border bg-muted p-8">
								<p class="text-sm text-muted-foreground">Capturing preview...</p>
							</div>
						{:else}
							<div class="flex items-center justify-center rounded-md border bg-muted p-8">
								<p class="text-sm text-muted-foreground">Click refresh to capture preview</p>
							</div>
						{/if}
					</div>
				{/if}

				<Separator />

				<div class="space-y-2">
					<Label>Detect Game Windows</Label>
					<Button 
						onclick={detectGameWindows} 
						disabled={isDetecting}
						class="w-full"
					>
						<RefreshCw class={`size-4 mr-2 ${isDetecting ? 'animate-spin' : ''}`} />
						{isDetecting ? "Detecting..." : "Scan for Game Windows"}
					</Button>
					<p class="text-xs text-muted-foreground">
						Make sure Slippi Dolphin is running, then click to scan
					</p>
				</div>

				{#if detectedWindows.length > 0}
					<Separator />
					<div class="space-y-2">
						<Label>Detected Windows ({detectedWindows.length})</Label>
						<div class="space-y-2">
							{#each detectedWindows as window}
								<button
									class="flex w-full items-center justify-between rounded-md border p-3 text-left hover:bg-accent transition-colors {window.is_child ? 'bg-blue-50 dark:bg-blue-950/20 border-blue-300 dark:border-blue-700' : ''}"
									onclick={() => selectGameWindow(window)}
								>
									<div class="flex-1 space-y-1">
										<div class="flex items-center gap-2">
											<p class="text-sm font-medium">{window.window_title}</p>
											{#if window.is_child}
												<span class="rounded bg-blue-500 px-1.5 py-0.5 text-xs font-medium text-white">CHILD</span>
											{/if}
											{#if window.has_owner}
												<span class="rounded bg-purple-500 px-1.5 py-0.5 text-xs font-medium text-white">OWNED</span>
											{/if}
										</div>
										<div class="flex flex-wrap gap-2 text-xs text-muted-foreground">
											<span>PID: {window.process_id}</span>
											<span>•</span>
											<span>{window.width}×{window.height}</span>
											<span>•</span>
											<span>Class: {window.class_name}</span>
											{#if window.is_cloaked}
												<span class="text-yellow-600">• Cloaked</span>
											{/if}
										</div>
									</div>
									<Button size="sm" variant="ghost">
										Select
									</Button>
								</button>
							{/each}
						</div>
					</div>
				{/if}
			</CardContent>
		</Card>

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

