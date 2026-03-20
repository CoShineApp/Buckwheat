<script lang="ts">
	import { settings } from "$lib/stores/settings.svelte";
	import { open } from "@tauri-apps/plugin-dialog";
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card";
	import { InputGroup, InputGroupInput, InputGroupButton } from "$lib/components/ui/input-group";
	import { Label } from "$lib/components/ui/label";
	import { Switch } from "$lib/components/ui/switch";
	import { Separator } from "$lib/components/ui/separator";
	import { Folder } from "@lucide/svelte";

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
</script>

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
