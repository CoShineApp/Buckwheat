<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { clipsStore, type ClipSession } from '$lib/stores/clips.svelte';
	import { cloudStorage } from '$lib/stores/cloud-storage.svelte';
	import { navigation } from '$lib/stores/navigation.svelte';
	import { formatDuration, formatFileSize } from '$lib/utils/format';
	import { Play, Share2, Trash2, RefreshCw, Scissors, Copy, ExternalLink, Cloud, Loader2, CloudOff, Clock, HardDrive, Check } from '@lucide/svelte';
	import { toast } from 'svelte-sonner';
	import { onMount } from 'svelte';
	import { invoke, convertFileSrc } from '@tauri-apps/api/core';
	import { listen } from '@tauri-apps/api/event';

	let isDeleting = $state<string | null>(null);
	let isUploading = $state<string | null>(null);
	let shareDialog = $state<{ clip: ClipSession; shareCode: string; url: string; alreadyExists: boolean } | null>(null);

	onMount(() => {
		clipsStore.refresh();
		cloudStorage.refreshUserClips();
		
		const unsubscribe = listen<string[]>('clip:created', () => {
			console.log('📢 clip:created event received, refreshing clips...');
			clipsStore.refresh();
		});
		
		return () => {
			unsubscribe.then(fn => fn());
		};
	});

	function handlePlay(clip: ClipSession) {
		navigation.navigateToReplay(clip.id, { isClip: true });
	}

	function isClipInCloud(clip: ClipSession): boolean {
		return cloudStorage.isClipUploaded(clip.filename);
	}

	function getExistingShareCode(clip: ClipSession): string | null {
		return cloudStorage.getClipShareCode(clip.filename);
	}

	async function handleShare(clip: ClipSession) {
		const existingShareCode = getExistingShareCode(clip);
		if (existingShareCode) {
			shareDialog = {
				clip,
				shareCode: existingShareCode,
				url: `https://clips.peppi.app/${existingShareCode}`,
				alreadyExists: true,
			};
			return;
		}

		isUploading = clip.id;
		try {
			const deviceId = await invoke<string>('get_device_id');
			toast.info('Uploading clip...');
			const { clip: cloudClip, alreadyExists } = await cloudStorage.createPublicClip(
				clip.video_path,
				deviceId,
				{
					slippi_metadata: clip.slippi_metadata,
					duration: clip.duration,
				}
			);
			
			shareDialog = {
				clip,
				shareCode: cloudClip.share_code,
				url: `https://clips.peppi.app/${cloudClip.share_code}`,
				alreadyExists,
			};
			
			if (alreadyExists) {
				toast.success('Clip was already uploaded!');
			} else {
				toast.success('Clip uploaded successfully!');
			}
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
			// TODO: Use Tauri shell plugin when available
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
			await cloudStorage.refreshUserClips();
			toast.success('Clips refreshed');
		} catch (error) {
			console.error('Refresh error:', error);
			toast.error('Failed to refresh clips');
		}
	}

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleDateString('en-US', { 
			month: 'short',
			day: 'numeric',
			hour: '2-digit', 
			minute: '2-digit' 
		});
	}

	function getClipName(clip: ClipSession): string {
		// Extract a cleaner name from filename
		const name = clip.filename.replace('.mp4', '');
		if (name.startsWith('Clip')) {
			return name;
		}
		return name;
	}
</script>

<div class="flex h-full flex-col gap-6 p-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div class="flex items-center gap-4">
			<div>
				<h1 class="text-2xl font-bold tracking-tight">Clips</h1>
				<p class="text-sm text-muted-foreground">
					{clipsStore.clips.length} clip{clipsStore.clips.length !== 1 ? 's' : ''} saved locally
				</p>
			</div>
		</div>
		<Button variant="outline" onclick={handleRefresh} disabled={clipsStore.loading} class="gap-2">
			<RefreshCw class={`size-4 ${clipsStore.loading ? 'animate-spin' : ''}`} />
			Refresh
		</Button>
	</div>

	<!-- Content -->
	{#if clipsStore.loading}
		<div class="flex-1 flex items-center justify-center">
			<div class="flex flex-col items-center gap-4">
				<div class="relative">
					<div class="w-16 h-16 rounded-full border-4 border-muted"></div>
					<div class="absolute inset-0 w-16 h-16 rounded-full border-4 border-t-violet-500 animate-spin"></div>
				</div>
				<p class="text-muted-foreground">Loading clips...</p>
			</div>
		</div>
	{:else if clipsStore.clips.length === 0}
		<!-- Empty State -->
		<div class="flex-1 flex items-center justify-center">
			<div class="flex flex-col items-center gap-6 max-w-md text-center">
				<div class="w-24 h-24 rounded-2xl bg-gradient-to-br from-violet-500/10 to-fuchsia-500/10 flex items-center justify-center border border-violet-500/20">
					<Scissors class="size-12 text-violet-400" />
				</div>
				<div class="space-y-2">
					<h2 class="text-xl font-semibold">No Clips Yet</h2>
					<p class="text-muted-foreground">
						Use the video editor to create clips from your recordings. 
						Trim and crop any moment, then save it as a clip to share with others.
					</p>
				</div>
			</div>
		</div>
	{:else}
		<!-- Clips Grid - Larger cards -->
		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
			{#each clipsStore.clips as clip (clip.id)}
				{@const inCloud = isClipInCloud(clip)}
				{@const shareCode = getExistingShareCode(clip)}
				
				<div class="group relative rounded-xl overflow-hidden bg-card border border-border/50 hover:border-border transition-all duration-300 hover:shadow-xl hover:shadow-black/20">
					<!-- Thumbnail - Much larger now -->
					<button
						type="button"
						onclick={() => handlePlay(clip)}
						class="relative w-full aspect-video bg-black cursor-pointer overflow-hidden"
					>
						{#if clip.thumbnail_path}
							<img
								src={convertFileSrc(clip.thumbnail_path)}
								alt={clip.filename}
								class="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
							/>
						{:else}
							<div class="w-full h-full flex items-center justify-center bg-gradient-to-br from-zinc-800 to-zinc-900">
								<Scissors class="size-10 text-zinc-600" />
							</div>
						{/if}
						
						<!-- Play overlay -->
						<div class="absolute inset-0 flex items-center justify-center bg-black/40 opacity-0 group-hover:opacity-100 transition-all duration-300">
							<div class="w-14 h-14 rounded-full bg-white/90 flex items-center justify-center shadow-2xl transform scale-90 group-hover:scale-100 transition-transform">
								<Play class="size-7 text-black fill-black ml-1" />
							</div>
						</div>
						
						<!-- Duration badge -->
						<div class="absolute bottom-2 right-2 px-2 py-0.5 rounded bg-black/80 text-white text-xs font-medium backdrop-blur-sm">
							{formatDuration(clip.duration)}
						</div>
						
						<!-- Cloud status badge - Much more prominent -->
						{#if inCloud}
							<div class="absolute top-2 left-2 flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-500/90 text-white text-xs font-medium backdrop-blur-sm shadow-lg">
								<Cloud class="size-3.5" />
								<span>In Cloud</span>
							</div>
						{/if}
					</button>
					
					<!-- Info section -->
					<div class="p-4 space-y-3">
						<!-- Clip name -->
						<div>
							<h3 class="font-semibold text-sm truncate" title={clip.filename}>
								{getClipName(clip)}
							</h3>
							<div class="flex items-center gap-3 mt-1 text-xs text-muted-foreground">
								<span class="flex items-center gap-1">
									<Clock class="size-3" />
									{formatDate(clip.start_time)}
								</span>
								<span class="flex items-center gap-1">
									<HardDrive class="size-3" />
									{formatFileSize(clip.file_size)}
								</span>
							</div>
						</div>
						
						<!-- Cloud status indicator -->
						{#if inCloud}
							<div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-emerald-500/10 border border-emerald-500/20">
								<Check class="size-4 text-emerald-500" />
								<span class="text-xs text-emerald-400 font-medium">Uploaded to cloud</span>
								{#if shareCode}
									<button 
										onclick={() => handleShare(clip)}
										class="ml-auto text-xs text-emerald-400 hover:text-emerald-300 underline underline-offset-2"
									>
										{shareCode}
									</button>
								{/if}
							</div>
						{:else}
							<div class="flex items-center gap-2 px-3 py-2 rounded-lg bg-zinc-500/10 border border-zinc-500/20">
								<CloudOff class="size-4 text-zinc-500" />
								<span class="text-xs text-zinc-500 font-medium">Local only</span>
							</div>
						{/if}
						
						<!-- Action buttons -->
						<div class="flex gap-2 pt-1">
							<Button 
								variant="default" 
								size="sm" 
								class="flex-1 gap-2"
								onclick={() => handlePlay(clip)}
							>
								<Play class="size-4" />
								Play
							</Button>
							
							{#if inCloud}
								<Button 
									variant="outline"
									size="sm"
									class="gap-2 border-emerald-500/30 text-emerald-400 hover:bg-emerald-500/10 hover:text-emerald-300"
									onclick={() => handleShare(clip)}
								>
									<Copy class="size-4" />
									Copy Link
								</Button>
							{:else}
								<Button 
									variant="outline"
									size="sm"
									class="gap-2"
									onclick={() => handleShare(clip)}
									disabled={isUploading === clip.id}
								>
									{#if isUploading === clip.id}
										<Loader2 class="size-4 animate-spin" />
										Uploading...
									{:else}
										<Cloud class="size-4" />
										Upload
									{/if}
								</Button>
							{/if}
							
							<Button 
								variant="ghost" 
								size="sm"
								class="w-9 p-0 text-muted-foreground hover:text-destructive"
								onclick={() => handleDelete(clip)}
								disabled={isDeleting === clip.id}
								title="Delete clip"
							>
								{#if isDeleting === clip.id}
									<Loader2 class="size-4 animate-spin" />
								{:else}
									<Trash2 class="size-4" />
								{/if}
							</Button>
						</div>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Share Dialog -->
{#if shareDialog}
	<button 
		type="button"
		class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 cursor-default"
		onclick={closeShareDialog}
		onkeydown={(e) => e.key === 'Escape' && closeShareDialog()}
	>
		<Card 
			class="w-full max-w-md m-4 cursor-auto"
			onclick={(e) => e.stopPropagation()}
		>
			<CardHeader>
				<CardTitle>
					{#if shareDialog.alreadyExists}
						Share Clip
					{:else}
						Clip Shared Successfully!
					{/if}
				</CardTitle>
				<CardDescription>
					{#if shareDialog.alreadyExists}
						This clip is already in the cloud. Copy the link to share it!
					{:else}
						Your clip has been uploaded and is now publicly accessible
					{/if}
				</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				<div class="space-y-2">
					<span class="text-sm font-medium">Share Code</span>
					<div class="flex gap-2">
						<input
							type="text"
							readonly
							value={shareDialog.shareCode}
							class="flex-1 px-3 py-2 bg-secondary rounded-md font-mono text-sm"
							aria-label="Share code"
						/>
						<Button size="sm" onclick={copyShareLink}>
							<Copy class="size-4" />
						</Button>
					</div>
				</div>

				<div class="space-y-2">
					<span class="text-sm font-medium">Public URL</span>
					<div class="flex gap-2">
						<input
							type="text"
							readonly
							value={shareDialog.url}
							class="flex-1 px-3 py-2 bg-secondary rounded-md text-sm"
							aria-label="Public URL"
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
	</button>
{/if}
