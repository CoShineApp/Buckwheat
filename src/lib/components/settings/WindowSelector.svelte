<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card";
	import { Label } from "$lib/components/ui/label";
	import { Separator } from "$lib/components/ui/separator";
	import { Monitor, RefreshCw } from "@lucide/svelte";
	import { onMount } from "svelte";
	import {
		listGameWindows,
		getGameProcessName,
		setGameProcessName,
		captureWindowPreview,
		highlightGameWindow,
		type GameWindow,
	} from "$lib/commands";
	import { toast } from "svelte-sonner";

	let currentProcessName = $state<string | null>(null);
	let detectedWindows = $state<GameWindow[]>([]);
	let isDetecting = $state(false);
	let previewImage = $state<string | null>(null);
	let isCapturingPreview = $state(false);
	let highlightingPid = $state<number | null>(null);

	onMount(async () => {
		try {
			currentProcessName = await getGameProcessName();
		} catch (error) {
			console.error("Failed to get game process name:", error);
		}
	});

	async function previewWindow(window: GameWindow): Promise<void> {
		highlightingPid = window.process_id;
		try {
			await highlightGameWindow(window.process_id);
		} finally {
			highlightingPid = null;
		}
	}

	async function detectGameWindows(): Promise<void> {
		isDetecting = true;
		try {
			const windows = await listGameWindows();
			detectedWindows = windows;
			if (windows.length === 0) {
				toast.error("No game windows detected", {
					description: "Make sure Slippi Dolphin is running and try again.",
				});
			} else {
				toast.success(`Found ${windows.length} game window(s)`, {
					description: "Select the one you want to use for recording.",
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
			const identifier = `${window.window_title} (PID: ${window.process_id})`;
			await setGameProcessName(identifier);
			currentProcessName = identifier;
			toast.success("Game window set", {
				description: `Now using: ${window.window_title}`,
			});
			detectedWindows = [];
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
					description: "Make sure the window is visible.",
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
				description: "Will use auto-detection",
			});
		} catch (error) {
			console.error("Failed to clear game process:", error);
		}
	}
</script>

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
						<RefreshCw class={`size-4 mr-2 ${isCapturingPreview ? "animate-spin" : ""}`} />
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
				<RefreshCw class={`size-4 mr-2 ${isDetecting ? "animate-spin" : ""}`} />
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
						<div
							class="flex w-full items-center justify-between rounded-md border p-3 transition-colors {window.is_child ? 'bg-blue-50 dark:bg-blue-950/20 border-blue-300 dark:border-blue-700' : ''} {highlightingPid === window.process_id ? 'ring-2 ring-yellow-400' : ''}"
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
									<span>-</span>
									<span>{window.width}x{window.height}</span>
									<span>-</span>
									<span>Class: {window.class_name}</span>
									{#if window.is_cloaked}
										<span class="text-yellow-600">- Cloaked</span>
									{/if}
								</div>
							</div>
							<div class="flex items-center gap-2">
								<Button
									size="sm"
									variant="outline"
									onclick={() => previewWindow(window)}
									disabled={highlightingPid !== null}
								>
									{highlightingPid === window.process_id ? "..." : "Preview"}
								</Button>
								<Button size="sm" onclick={() => selectGameWindow(window)}>
									Select
								</Button>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}
	</CardContent>
</Card>
