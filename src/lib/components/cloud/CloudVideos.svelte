<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { 
		Table, 
		TableBody, 
		TableCell, 
		TableHead, 
		TableHeader, 
		TableRow 
	} from '$lib/components/ui/table';
	import { cloudStorage, type Upload, type CloudItem } from '$lib/stores/cloud-storage.svelte';
	import { auth } from '$lib/stores/auth.svelte';
	import { Cloud, Download, Trash2, RefreshCw, Loader2, Copy, Film, Scissors, ExternalLink } from '@lucide/svelte';
	import { toast } from 'svelte-sonner';
	import { onMount } from 'svelte';
	import { save } from '@tauri-apps/plugin-dialog';

	let isDeleting = $state<string | null>(null);
	let isDownloading = $state<string | null>(null);

	onMount(() => {
		// Refresh both uploads and clips
		cloudStorage.refreshAll();
	});

	async function handleDownload(item: CloudItem) {
		// Only recordings can be downloaded (clips are public)
		if (item.type !== 'recording') {
			return;
		}

		try {
			// Prompt user for save location
			const savePath = await save({
				defaultPath: item.filename,
				filters: [{
					name: 'Video',
					extensions: ['mp4']
				}]
			});

			if (!savePath) {
				// User cancelled
				return;
			}

			isDownloading = item.id;
			toast.info('Downloading video...');

			await cloudStorage.downloadVideo(item.id, savePath);
			
			toast.success('Video downloaded successfully!');
		} catch (error) {
			console.error('Download error:', error);
			toast.error(error instanceof Error ? error.message : 'Failed to download video');
		} finally {
			isDownloading = null;
		}
	}

	async function handleDelete(item: CloudItem) {
		const itemType = item.type === 'recording' ? 'recording' : 'clip';
		if (!confirm(`Are you sure you want to delete this ${itemType} from cloud storage?`)) {
			return;
		}

		isDeleting = item.id;
		try {
			if (item.type === 'recording') {
				await cloudStorage.deleteUpload(item.id);
			} else {
				// Delete clip from database
				const { error } = await auth.supabase
					.from('clips')
					.delete()
					.eq('id', item.id);
				if (error) throw error;
				await cloudStorage.refreshUserClips();
			}
			toast.success(`${itemType.charAt(0).toUpperCase() + itemType.slice(1)} deleted from cloud`);
		} catch (error) {
			console.error('Delete error:', error);
			toast.error(`Failed to delete ${itemType}`);
		} finally {
			isDeleting = null;
		}
	}

	async function handleCopyLink(shareCode: string) {
		try {
			const url = `https://clips.peppi.app/${shareCode}`;
			await navigator.clipboard.writeText(url);
			toast.success('Link copied to clipboard!');
		} catch (error) {
			toast.error('Failed to copy link');
		}
	}

	function openClipLink(shareCode: string) {
		window.open(`https://clips.peppi.app/${shareCode}`, '_blank');
	}

	async function handleRefresh() {
		try {
			await cloudStorage.refreshAll();
			toast.success('Refreshed cloud content');
		} catch (error) {
			console.error('Refresh error:', error);
			toast.error('Failed to refresh');
		}
	}

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
	}

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], {
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	const isLoading = $derived(cloudStorage.loading || cloudStorage.clipsLoading);
	const totalItems = $derived(cloudStorage.totalCloudItems);
	const recordingCount = $derived(cloudStorage.uploads.length);
	const clipCount = $derived(cloudStorage.userClips.length);
</script>

<Card>
	<CardHeader>
		<div class="flex items-center justify-between">
			<div class="flex items-center gap-2">
				<Cloud class="size-5" />
				<CardTitle>Cloud Storage</CardTitle>
			</div>
			<Button variant="ghost" size="sm" onclick={handleRefresh} disabled={isLoading}>
				<RefreshCw class={`size-4 mr-2 ${isLoading ? 'animate-spin' : ''}`} />
				Refresh
			</Button>
		</div>
		<CardDescription>
			{#if totalItems === 0}
				No items in cloud storage
			{:else}
				{totalItems} item{totalItems !== 1 ? 's' : ''} in cloud
				<span class="text-muted-foreground/70">
					({recordingCount} recording{recordingCount !== 1 ? 's' : ''}, {clipCount} clip{clipCount !== 1 ? 's' : ''})
				</span>
			{/if}
		</CardDescription>
	</CardHeader>

	{#if isLoading && totalItems === 0}
		<CardContent>
			<div class="flex items-center justify-center py-8">
				<Loader2 class="size-6 animate-spin text-muted-foreground" />
			</div>
		</CardContent>
	{:else if totalItems > 0}
		<CardContent>
			<div class="rounded-md border">
				<Table>
					<TableHeader>
						<TableRow>
							<TableHead class="w-[80px]">Type</TableHead>
							<TableHead>Filename</TableHead>
							<TableHead class="w-[100px]">Size</TableHead>
							<TableHead class="w-[150px]">Uploaded</TableHead>
							<TableHead class="w-[120px]">Share</TableHead>
							<TableHead class="text-right w-[100px]">Actions</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{#each cloudStorage.allCloudItems as item (item.id)}
							<TableRow>
								<TableCell>
									<div class="flex items-center gap-1.5">
										{#if item.type === 'recording'}
											<Film class="size-4 text-blue-500" />
											<span class="text-xs text-muted-foreground">Recording</span>
										{:else}
											<Scissors class="size-4 text-green-500" />
											<span class="text-xs text-muted-foreground">Clip</span>
										{/if}
									</div>
								</TableCell>
								<TableCell class="font-medium max-w-[200px] truncate" title={item.filename}>
									{item.filename}
								</TableCell>
								<TableCell>{formatBytes(item.file_size)}</TableCell>
								<TableCell class="text-sm">{formatDate(item.uploaded_at)}</TableCell>
								<TableCell>
									{#if item.type === 'clip' && item.share_code}
										<div class="flex items-center gap-1">
											<Button 
												variant="ghost" 
												size="sm"
												class="h-7 px-2"
												onclick={() => handleCopyLink(item.share_code!)}
												title="Copy share link"
											>
												<Copy class="size-3 mr-1" />
												<span class="text-xs font-mono">{item.share_code}</span>
											</Button>
											<Button 
												variant="ghost" 
												size="sm"
												class="h-7 w-7 p-0"
												onclick={() => openClipLink(item.share_code!)}
												title="Open in browser"
											>
												<ExternalLink class="size-3" />
											</Button>
										</div>
									{:else}
										<span class="text-xs text-muted-foreground">—</span>
									{/if}
								</TableCell>
								<TableCell class="text-right">
									<div class="flex justify-end gap-1">
										{#if item.type === 'recording'}
											<Button 
												variant="ghost" 
												size="sm"
												class="h-7 w-7 p-0"
												onclick={() => handleDownload(item)}
												disabled={isDownloading === item.id}
												title="Download"
											>
												{#if isDownloading === item.id}
													<Loader2 class="size-4 animate-spin" />
												{:else}
													<Download class="size-4" />
												{/if}
											</Button>
										{/if}
										<Button 
											variant="ghost" 
											size="sm"
											class="h-7 w-7 p-0"
											onclick={() => handleDelete(item)}
											disabled={isDeleting === item.id}
											title="Delete"
										>
											{#if isDeleting === item.id}
												<Loader2 class="size-4 animate-spin" />
											{:else}
												<Trash2 class="size-4" />
											{/if}
										</Button>
									</div>
								</TableCell>
							</TableRow>
						{/each}
					</TableBody>
				</Table>
			</div>
		</CardContent>
	{:else}
		<CardContent>
			<div class="flex flex-col items-center justify-center py-8 text-center">
				<Cloud class="size-12 text-muted-foreground/50 mb-3" />
				<p class="text-sm text-muted-foreground">
					No items in cloud storage yet.
				</p>
				<p class="text-xs text-muted-foreground/70 mt-1">
					Share clips or upload recordings to see them here.
				</p>
			</div>
		</CardContent>
	{/if}
</Card>
