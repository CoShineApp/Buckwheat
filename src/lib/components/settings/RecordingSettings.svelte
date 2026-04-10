<script lang="ts">
	import { settings } from "$lib/stores/settings.svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import { invoke } from "@tauri-apps/api/core";
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card";
	import { InputGroup, InputGroupInput, InputGroupButton } from "$lib/components/ui/input-group";
	import { Label } from "$lib/components/ui/label";
	import { Switch } from "$lib/components/ui/switch";
	import { Separator } from "$lib/components/ui/separator";
	import { Gamepad2, Folder, Monitor } from "@lucide/svelte";
	import { onMount } from "svelte";
	import { formatFileSize } from "$lib/utils/format";

	let storageUsage = $state<{ totalBytes: number; recordingCount: number } | null>(null);
	let obsTestStatus = $state<"idle" | "testing" | "success" | "error">("idle");
	let obsTestMessage = $state("");

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

		<!-- OBS Integration Section -->
		<div class="space-y-4">
			<div class="flex items-center gap-2">
				<Monitor class="size-4" />
				<Label>OBS Integration</Label>
			</div>

			<div class="space-y-3">
				<label class="flex items-center gap-3 cursor-pointer">
					<input
						type="radio"
						name="obs-mode"
						value="managed"
						checked={settings.obsMode === "managed"}
						onchange={() => settings.set("obsMode", "managed")}
						class="accent-primary"
					/>
					<div>
						<span class="text-sm font-medium">Automatic</span>
						<span class="text-xs text-muted-foreground ml-1">(recommended)</span>
						<p class="text-xs text-muted-foreground">Peppi manages OBS for you</p>
					</div>
				</label>

				<label class="flex items-center gap-3 cursor-pointer">
					<input
						type="radio"
						name="obs-mode"
						value="connect"
						checked={settings.obsMode === "connect"}
						onchange={() => settings.set("obsMode", "connect")}
						class="accent-primary"
					/>
					<div>
						<span class="text-sm font-medium">Connect to my OBS</span>
						<p class="text-xs text-muted-foreground">For streamers who already use OBS</p>
					</div>
				</label>
			</div>

			{#if settings.obsMode === "connect"}
				<div class="space-y-3 pl-6 border-l-2 border-muted">
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
			{/if}
		</div>
	</CardContent>
</Card>
