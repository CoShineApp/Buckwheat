<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { clipsStore, type ClipSession } from '$lib/stores/clips.svelte';
	import { cloudStorage } from '$lib/stores/cloud-storage.svelte';
	import { auth } from '$lib/stores/auth.svelte';
	import { navigation } from '$lib/stores/navigation.svelte';
	import { Play, Share2, Trash2, RefreshCw, Scissors, Copy, ExternalLink } from '@lucide/svelte';
	import { toast } from 'svelte-sonner';
	import { onMount } from 'svelte';
	import { invoke, convertFileSrc } from '@tauri-apps/api/core';

	let isDeleting = $state<string | null>(null);
	let isUploading = $state<string | null>(null);
	let shareDialog = $state<{ clip: ClipSession; shareCode: string; url: string } | null>(null);

	onMount(() => {
		clipsStore.refresh();
	});

	function handlePlay(clip: ClipSession) {
	navigation.navigateToReplay(clip.id, { isClip: true });
	}

	async function handleShare(clip: ClipSession) {
		isUploading = clip.id;
		try {
			// Get device ID
			const deviceId = await invoke<string>('get_device_id');
			
			// Upload clip to cloud
			toast.info('Uploading clip...');
			const result = await cloudStorage.createPublicClip(
				clip.video_path,
				deviceId,
				{
					slippi_metadata: clip.slippi_metadata,
					duration: clip.duration,
				}
			);
			
			// Show share dialog
			shareDialog = {
				clip,
				shareCode: result.share_code,
				url: `https://clips.peppi.app/${result.share_code}`,
			};
			
			toast.success('Clip uploaded successfully!');
		} catch (error) {
			console.error('Upload error:', error);
			toast.error(error instanceof Error ? error.message : 'Failed to upload clip');
		} finally {
			isUploading = null;
		}
	}

	function closeShareDialog() {
		shareDialog = null;
	}

	async function copyShareLink() {
		if (shareDialog) {
			try {
				await navigator.clipboard.writeText(shareDialog.url);
				toast.success('Link copied to clipboard!');
			} catch (error) {
				toast.error('Failed to copy link');
			}
		}
	}

	function openShareLink() {
		if (shareDialog) {
			window.open(shareDialog.url, '_blank');
		}
	}

	async function handleDelete(clip: ClipSession) {
		if (!confirm(`Delete clip "${clip.filename}"?`)) {
			return;
		}

		isDeleting = clip.id;
		try {
			await clipsStore.deleteClip(clip.id, clip.video_path);
			toast.success('Clip deleted');
		} catch (error) {
			console.error('Delete error:', error);
			toast.error('Failed to delete clip');
		} finally {
			isDeleting = null;
		}
	}

	async function handleRefresh() {
		try {
			await clipsStore.refresh();
			toast.success('Clips refreshed');
		} catch (error) {
			console.error('Refresh error:', error);
			toast.error('Failed to refresh clips');
		}
	}

	function formatDuration(seconds: number | null): string {
		if (!seconds) return 'Unknown';
		const mins = Math.floor(seconds / 60);
		const secs = seconds % 60;
		return `${mins}:${secs.toString().padStart(2, '0')}`;
	}

	function formatFileSize(bytes: number | null): string {
		if (!bytes) return 'Unknown';
		const mb = bytes / 1024 / 1024;
		if (mb < 1) {
			return `${(bytes / 1024).toFixed(1)} KB`;
		}
		return `${mb.toFixed(1)} MB`;
	}

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { 
			hour: '2-digit', 
			minute: '2-digit' 
		});
	}
</script>

<div class="flex h-full flex-col gap-4 p-4">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-2">
			<Scissors class="size-5" />
			<div>
				<h2 class="text-xl font-bold">Clips</h2>
				<p class="text-sm text-muted-foreground">
					{clipsStore.clips.length} clip{clipsStore.clips.length !== 1 ? 's' : ''}
				</p>
			</div>
		</div>
		<Button variant="outline" size="sm" onclick={handleRefresh} disabled={clipsStore.loading}>
			<RefreshCw class={`size-3 mr-1.5 ${clipsStore.loading ? 'animate-spin' : ''}`} />
			Refresh
		</Button>
	</div>

	<!-- Clips Grid -->
	{#if clipsStore.loading}
		<div class="flex items-center justify-center py-12">
			<RefreshCw class="size-8 animate-spin text-muted-foreground" />
		</div>
	{:else if clipsStore.clips.length === 0}
		<!-- Empty State -->
		<Card>
			<CardContent class="flex flex-col items-center justify-center py-12">
				<Scissors class="size-16 text-muted-foreground mb-4" />
				<CardTitle class="mb-2">No Clips Yet</CardTitle>
				<CardDescription class="text-center max-w-md">
					Press the Create Clip hotkey during a recording to capture the last few seconds.
					Clips will appear here after the recording ends.
				</CardDescription>
			</CardContent>
		</Card>
	{:else}
		<!-- Grid of Clips -->
		<div class="grid gap-3 md:grid-cols-2 lg:grid-cols-4 xl:grid-cols-5">
			{#each clipsStore.clips as clip (clip.id)}
				<Card class="overflow-hidden hover:shadow-lg transition-shadow">
					<CardContent class="p-2 space-y-1.5">
						<!-- Thumbnail -->
						<div class="flex justify-center">
							{#if clip.thumbnail_path}
								<button
									type="button"
									onclick={() => handlePlay(clip)}
									class="relative w-24 h-24 bg-black overflow-hidden rounded-md cursor-pointer group"
								>
									<img
										src={convertFileSrc(clip.thumbnail_path)}
										alt={clip.filename}
										class="w-full h-full object-cover"
									/>
									<!-- Play overlay -->
									<div class="absolute inset-0 flex items-center justify-center bg-black/30 opacity-0 group-hover:opacity-100 transition-opacity rounded-md">
										<Play class="size-5 text-white drop-shadow-lg fill-white" />
									</div>
								</button>
							{:else}
								<button
									type="button"
									onclick={() => handlePlay(clip)}
									class="relative w-24 h-24 bg-muted flex items-center justify-center cursor-pointer hover:bg-muted/80 transition-colors rounded-md"
								>
									<Scissors class="size-5 text-muted-foreground" />
								</button>
							{/if}
						</div>
						
						<!-- Filename -->
						<div class="space-y-1">
							<p class="text-xs font-medium truncate" title={clip.filename}>
								{clip.filename}
							</p>
							<p class="text-[10px] text-muted-foreground">
								{formatDate(clip.start_time)}
							</p>
						</div>

						<!-- Metadata -->
						<div class="flex justify-between text-[10px] text-muted-foreground">
							<span>{formatDuration(clip.duration)}</span>
							<span>{formatFileSize(clip.file_size)}</span>
						</div>

						<!-- Actions -->
						<div class="flex gap-1 pt-1">
							<Button 
								variant="default" 
								size="sm" 
								class="flex-1 h-7 text-xs"
								onclick={() => handlePlay(clip)}
							>
								<Play class="size-3 mr-1" />
								Play
							</Button>
							<Button 
								variant="outline" 
								size="sm"
								class="h-7 w-7 p-0"
								onclick={() => handleShare(clip)}
								disabled={isUploading === clip.id}
								title="Share"
							>
								<Share2 class="size-3" />
							</Button>
							<Button 
								variant="outline" 
								size="sm"
								class="h-7 w-7 p-0"
								onclick={() => handleDelete(clip)}
								disabled={isDeleting === clip.id}
								title="Delete"
							>
								<Trash2 class="size-3" />
							</Button>
						</div>
					</CardContent>
				</Card>
			{/each}
		</div>
	{/if}
</div>

<!-- Share Dialog -->
{#if shareDialog}
	<div 
		class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
		onclick={closeShareDialog}
	>
		<Card 
			class="w-full max-w-md m-4"
			onclick={(e) => e.stopPropagation()}
		>
			<CardHeader>
				<CardTitle>Clip Shared Successfully!</CardTitle>
				<CardDescription>
					Your clip has been uploaded and is now publicly accessible
				</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="space-y-2">
					<label class="text-sm font-medium">Share Code</label>
					<div class="flex gap-2">
						<input
							type="text"
							readonly
							value={shareDialog.shareCode}
							class="flex-1 px-3 py-2 bg-secondary rounded-md font-mono text-sm"
						/>
						<Button size="sm" onclick={copyShareLink}>
							<Copy class="size-4" />
						</Button>
					</div>
				</div>

				<div class="space-y-2">
					<label class="text-sm font-medium">Public URL</label>
					<div class="flex gap-2">
						<input
							type="text"
							readonly
							value={shareDialog.url}
							class="flex-1 px-3 py-2 bg-secondary rounded-md text-sm"
						/>
						<Button size="sm" onclick={copyShareLink}>
							<Copy class="size-4" />
						</Button>
					</div>
				</div>

				<div class="flex gap-2 pt-2">
					<Button variant="outline" class="flex-1" onclick={openShareLink}>
						<ExternalLink class="size-4 mr-2" />
						Open in Browser
					</Button>
					<Button class="flex-1" onclick={closeShareDialog}>
						Done
					</Button>
				</div>
			</CardContent>
		</Card>
	</div>
{/if}

