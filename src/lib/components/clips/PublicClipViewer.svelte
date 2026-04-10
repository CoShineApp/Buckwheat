<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { cloudStorage, type Clip } from '$lib/stores/cloud-storage.svelte';
	import { Film, Share2, Copy } from '@lucide/svelte';
	import { toast } from 'svelte-sonner';
	import { formatFileSize } from '$lib/utils/format';

	let shareCode = $state('');
	let clip = $state<Clip | null>(null);
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	async function handleSearch() {
		if (!shareCode.trim()) {
			toast.error('Please enter a share code');
			return;
		}

		isLoading = true;
		error = null;
		clip = null;

		try {
			const result = await cloudStorage.getClipByCode(shareCode.toUpperCase());
			clip = result;
		} catch (err) {
			console.error('Error loading clip:', err);
			error = err instanceof Error ? err.message : 'Failed to load clip';
			toast.error('Clip not found');
		} finally {
			isLoading = false;
		}
	}

	function copyShareCode() {
		if (clip) {
			navigator.clipboard.writeText(clip.share_code);
			toast.success('Share code copied!');
		}
	}

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
	}

</script>

<Card>
	<CardHeader>
		<div class="flex items-center gap-2">
			<Film class="size-5" />
			<CardTitle>View Public Clip</CardTitle>
		</div>
		<CardDescription>Enter a share code to view a public clip</CardDescription>
	</CardHeader>

	<CardContent class="space-y-4">
		<form onsubmit={(e) => { e.preventDefault(); handleSearch(); }} class="flex gap-2">
			<div class="flex-1">
				<Input
					type="text"
					placeholder="Enter 8-character code (e.g., ABC12345)"
					bind:value={shareCode}
					disabled={isLoading}
					class="uppercase"
					maxlength={8}
				/>
			</div>
			<Button type="submit" disabled={isLoading || !shareCode.trim()}>
				{isLoading ? 'Loading...' : 'View Clip'}
			</Button>
		</form>

		{#if error}
			<div class="p-4 rounded-lg bg-red-500/10 text-red-500 text-sm">
				{error}
			</div>
		{/if}

		{#if clip}
			<div class="space-y-4">
				<div class="rounded-lg border bg-card p-4 space-y-3">
					<div class="flex items-center justify-between">
						<h3 class="font-semibold">{clip.filename}</h3>
						<Button variant="ghost" size="sm" onclick={copyShareCode}>
							<Copy class="size-4 mr-2" />
							Copy Code
						</Button>
					</div>

					<div class="grid grid-cols-2 gap-2 text-sm">
						<div>
							<p class="text-muted-foreground">Share Code</p>
							<p class="font-mono font-medium">{clip.share_code}</p>
						</div>
						<div>
							<p class="text-muted-foreground">File Size</p>
							<p class="font-medium">{formatFileSize(clip.file_size)}</p>
						</div>
						<div>
							<p class="text-muted-foreground">Uploaded</p>
							<p class="font-medium">{formatDate(clip.uploaded_at)}</p>
						</div>
						{#if clip.duration_seconds}
							<div>
								<p class="text-muted-foreground">Duration</p>
								<p class="font-medium">{clip.duration_seconds}s</p>
							</div>
						{/if}
					</div>

					{#if clip.b2_file_name}
						<div class="pt-3 border-t">
							<video 
								controls 
								class="w-full rounded-lg bg-black"
								src={clip.b2_file_name}
							>
								<track kind="captions" />
								Your browser does not support the video tag.
							</video>
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</CardContent>
</Card>

