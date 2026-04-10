<script lang="ts">
	import { settings } from "$lib/stores/settings.svelte";
	import { Button } from "$lib/components/ui/button";
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card";
	import { Label } from "$lib/components/ui/label";
	import { Palette } from "@lucide/svelte";
</script>

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
