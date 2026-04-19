<script lang="ts">
	import { settings, type GameWindowInfo } from "$lib/stores/settings.svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import { invoke } from "@tauri-apps/api/core";
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card";
	import { InputGroup, InputGroupInput, InputGroupButton } from "$lib/components/ui/input-group";
	import { Label } from "$lib/components/ui/label";
	import { Switch } from "$lib/components/ui/switch";
	import { Separator } from "$lib/components/ui/separator";
	import { Gamepad2, Folder, Monitor, RefreshCw } from "@lucide/svelte";
	import { onMount } from "svelte";
	import { formatFileSize } from "$lib/utils/format";

	type DetectedWindow = {
		process_name: string;
		window_title: string;
		class_name: string;
		width: number;
		height: number;
		process_id: number;
	};

	let storageUsage = $state<{ totalBytes: number; recordingCount: number } | null>(null);
	let obsTestStatus = $state<"idle" | "testing" | "success" | "error">("idle");
	let obsTestMessage = $state("");
	let detectedWindows = $state<DetectedWindow[]>([]);
	let windowsLoading = $state(false);
	let previewDataUrl = $state<string | null>(null);
	let previewLoading = $state(false);

	async function refreshWindows(): Promise<void> {
		windowsLoading = true;
		try {
			detectedWindows = await invoke<DetectedWindow[]>("list_all_windows");
		} catch (error) {
			console.error("Failed to list windows:", error);
			detectedWindows = [];
		} finally {
			windowsLoading = false;
		}
	}

	async function selectWindow(w: DetectedWindow): Promise<void> {
		const info: GameWindowInfo = {
			title: w.window_title,
			className: w.class_name,
			processName: w.process_name,
			width: w.width,
			height: w.height,
		};
		await settings.set("gameWindow", info);

		// Capture a preview screenshot of the selected window and display it
		previewLoading = true;
		previewDataUrl = null;
		try {
			const b64 = await invoke<string>("capture_window_by_pid", { processId: w.process_id });
			previewDataUrl = `data:image/png;base64,${b64}`;
		} catch (error) {
			console.warn("Failed to capture window preview:", error);
			previewDataUrl = null;
		} finally {
			previewLoading = false;
		}
	}

	function clearWindow(): void {
		settings.set("gameWindow", null);
		previewDataUrl = null;
	}

	function isSelected(w: DetectedWindow): boolean {
		const g = settings.gameWindow;
		return (
			g !== null &&
			g.processName === w.process_name &&
			g.className === w.class_name &&
			g.title === w.window_title
		);
	}

	async function testObsConnection(): Promise<void> {
		obsTestStatus = "testing";
		obsTestMessage = "";
		try {
			await invoke("ensure_obs_ready");
			obsTestStatus = "success";
			obsTestMessage = "Connected successfully";
		} catch (error) {
			obsTestStatus = "error";
			obsTestMessage = String(error);
		}
	}

	const storagePercentage = $derived.by(() => {
		if (!storageUsage || settings.storageLimit === 0) return 0;
		const limitBytes = settings.storageLimit * 1024 * 1024 * 1024;
		return Math.min(100, (storageUsage.totalBytes / limitBytes) * 100);
	});

	async function loadStorageUsage(): Promise<void> {
		try {
			storageUsage = await invoke<{ totalBytes: number; recordingCount: number }>("get_storage_usage");
		} catch (error) {
			console.error("Failed to load storage usage:", error);
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

	onMount(async () => {
		await loadStorageUsage();
		await refreshWindows();
	});
</script>

<Card>
	<CardHeader>
		<div class="flex items-center gap-2">
			<Gamepad2 class="size-5" />
			<CardTitle>Recording</CardTitle>
		</div>
		<CardDescription>Configure recording behavior</CardDescription>
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

		<Separator />

		<!-- Storage Limit Section -->
		<div class="space-y-4">
			<div class="space-y-2">
				<Label for="storage-limit">Storage Limit (GB)</Label>
				<div class="flex items-center gap-4">
					<input
						id="storage-limit"
						type="number"
						min="0"
						step="1"
						class="flex h-9 w-24 rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
						value={settings.storageLimit}
						oninput={(e) => {
							const value = parseInt(e.currentTarget.value) || 0;
							settings.set("storageLimit", Math.max(0, value));
						}}
					/>
					<span class="text-sm text-muted-foreground">
						{settings.storageLimit === 0 ? "Unlimited" : `${settings.storageLimit} GB`}
					</span>
				</div>
				<p class="text-xs text-muted-foreground">
					Set to 0 for unlimited storage. When a limit is set, oldest recordings are automatically deleted.
				</p>
			</div>

			{#if storageUsage}
				<div class="space-y-2">
					<div class="flex items-center justify-between text-sm">
						<span class="text-muted-foreground">Current Usage</span>
						<span class="font-medium">
							{formatFileSize(storageUsage.totalBytes)}
							{#if settings.storageLimit > 0}
								/ {settings.storageLimit} GB
							{/if}
							<span class="text-muted-foreground ml-1">({storageUsage.recordingCount} recordings)</span>
						</span>
					</div>
					{#if settings.storageLimit > 0}
						<div class="h-2 w-full rounded-full bg-muted overflow-hidden">
							<div
								class="h-full rounded-full transition-all duration-300 {storagePercentage > 90 ? 'bg-destructive' : storagePercentage > 70 ? 'bg-yellow-500' : 'bg-primary'}"
								style="width: {storagePercentage}%"
							></div>
						</div>
					{/if}
				</div>
			{/if}
		</div>

		<Separator />

		<!-- Capture Window Section -->
		<div class="space-y-4">
			<div class="flex items-center gap-2">
				<Monitor class="size-4" />
				<Label>Capture Window</Label>
			</div>
			<p class="text-xs text-muted-foreground">
				Select which window OBS should capture. Leave unset to auto-detect Slippi Dolphin.
			</p>

			<div class="flex items-center gap-2">
				<button
					class="inline-flex items-center gap-1.5 rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring border border-input bg-background hover:bg-accent hover:text-accent-foreground h-8 px-3"
					onclick={refreshWindows}
					disabled={windowsLoading}
				>
					<RefreshCw class="size-3.5 {windowsLoading ? 'animate-spin' : ''}" />
					Refresh list
				</button>
				{#if settings.gameWindow}
					<button
						class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring h-8 px-3 hover:bg-accent hover:text-accent-foreground text-muted-foreground"
						onclick={clearWindow}
					>
						Clear selection
					</button>
				{/if}
			</div>

			{#if settings.gameWindow}
				<div class="rounded-md border border-input bg-muted/30 p-2 space-y-2">
					<div class="text-xs">
						<div class="font-medium">Selected:</div>
						<div class="text-muted-foreground">
							{settings.gameWindow.title || "(no title)"} — {settings.gameWindow.processName}
						</div>
					</div>
					{#if previewLoading}
						<div class="flex h-32 items-center justify-center text-xs text-muted-foreground">
							Capturing preview…
						</div>
					{:else if previewDataUrl}
						<img
							src={previewDataUrl}
							alt="Window preview"
							class="w-full rounded border border-input object-contain max-h-64 bg-background"
						/>
					{/if}
				</div>
			{/if}

			{#if detectedWindows.length === 0 && !windowsLoading}
				<p class="text-xs text-muted-foreground italic">No windows detected. Launch Slippi Dolphin and click Refresh.</p>
			{:else}
				<div class="max-h-60 overflow-y-auto space-y-1 rounded-md border border-input p-1">
					{#each detectedWindows as w, i (`${w.process_id}-${w.class_name}-${w.window_title}-${i}`)}
						<button
							class="w-full text-left rounded px-2 py-1.5 text-sm hover:bg-accent hover:text-accent-foreground transition-colors {isSelected(w) ? 'bg-accent' : ''}"
							onclick={() => selectWindow(w)}
						>
							<div class="font-medium truncate">{w.window_title || "(no title)"}</div>
							<div class="text-xs text-muted-foreground truncate">
								{w.process_name} · {w.width}×{w.height} · {w.class_name}
							</div>
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<Separator />

		<!-- OBS Integration Section -->
		<div class="space-y-4">
			<div class="flex items-center gap-2">
				<Monitor class="size-4" />
				<Label>OBS WebSocket</Label>
			</div>
			<p class="text-xs text-muted-foreground">
				Peppi connects to OBS via WebSocket to control recording. If OBS is already running, Peppi uses it. Otherwise, Peppi will launch OBS for you.
			</p>

			<div class="space-y-3">
				<div class="space-y-1">
					<Label for="obs-port">WebSocket Port</Label>
					<input
						id="obs-port"
						type="number"
						min="1"
						max="65535"
						class="flex h-9 w-24 rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
						value={settings.obsPort}
						oninput={(e) => {
							const value = parseInt(e.currentTarget.value) || 4455;
							settings.set("obsPort", Math.max(1, Math.min(65535, value)));
						}}
					/>
				</div>

				<div class="space-y-1">
					<Label for="obs-password">Password</Label>
					<input
						id="obs-password"
						type="password"
						placeholder="Leave empty if no password"
						class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
						value={settings.obsPassword}
						oninput={(e) => settings.set("obsPassword", e.currentTarget.value)}
					/>
				</div>

				<div class="flex items-center gap-2">
					<button
						class="inline-flex items-center justify-center rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 border border-input bg-background hover:bg-accent hover:text-accent-foreground h-8 px-3"
						onclick={testObsConnection}
						disabled={obsTestStatus === "testing"}
					>
						{obsTestStatus === "testing" ? "Testing..." : "Test Connection"}
					</button>
					{#if obsTestStatus === "success"}
						<span class="text-xs text-green-600 dark:text-green-400">{obsTestMessage}</span>
					{:else if obsTestStatus === "error"}
						<span class="text-xs text-destructive">{obsTestMessage}</span>
					{/if}
				</div>
			</div>
		</div>
	</CardContent>
</Card>
