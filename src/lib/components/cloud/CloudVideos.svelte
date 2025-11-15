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
	import { cloudStorage, type Upload } from '$lib/stores/cloud-storage.svelte';
	import { auth } from '$lib/stores/auth.svelte';
	import { Cloud, Download, Trash2, RefreshCw, Loader2 } from '@lucide/svelte';
	import { toast } from 'svelte-sonner';
	import { onMount } from 'svelte';
	import { save } from '@tauri-apps/plugin-dialog';

	let isDeleting = $state<string | null>(null);
	let isDownloading = $state<string | null>(null);

	onMount(() => {
		if (auth.isAuthenticated) {
			cloudStorage.refreshUploads();
		}
	});

	async function handleDownload(upload: Upload) {
		try {
			// Prompt user for save location
			const savePath = await save({
				defaultPath: upload.filename,
				filters: [{
					name: 'Video',
					extensions: ['mp4']
				}]
			});

			if (!savePath) {
				// User cancelled
				return;
			}

			isDownloading = upload.id;
			toast.info('Downloading video...');

			await cloudStorage.downloadVideo(upload.id, savePath);
			
			toast.success('Video downloaded successfully!');
		} catch (error) {
			console.error('Download error:', error);
			toast.error(error instanceof Error ? error.message : 'Failed to download video');
		} finally {
			isDownloading = null;
		}
	}

	async function handleDelete(uploadId: string) {
		if (!confirm('Are you sure you want to delete this recording from cloud storage?')) {
			return;
		}

		isDeleting = uploadId;
		try {
			await cloudStorage.deleteUpload(uploadId);
			toast.success('Recording deleted from cloud');
		} catch (error) {
			console.error('Delete error:', error);
			toast.error('Failed to delete recording');
		} finally {
			isDeleting = null;
		}
	}

	async function handleRefresh() {
		try {
			await cloudStorage.refreshUploads();
			toast.success('Refreshed cloud recordings');
		} catch (error) {
			console.error('Refresh error:', error);
			toast.error('Failed to refresh recordings');
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
		return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
	}
</script>

{#if auth.isAuthenticated}
	<Card>
		<CardHeader>
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<Cloud class="size-5" />
					<CardTitle>Cloud Recordings</CardTitle>
				</div>
				<Button variant="ghost" size="sm" onclick={handleRefresh} disabled={cloudStorage.loading}>
					<RefreshCw class="size-4 mr-2"  />
					Refresh
				</Button>
			</div>
			<CardDescription>
				{cloudStorage.uploads.length === 0 
					? 'No recordings in cloud storage' 
					: `${cloudStorage.uploads.length} recording(s) in cloud`}
			</CardDescription>
		</CardHeader>

		{#if cloudStorage.uploads.length > 0}
			<CardContent>
				<div class="rounded-md border">
					<Table>
						<TableHeader>
							<TableRow>
								<TableHead>Filename</TableHead>
								<TableHead>Size</TableHead>
								<TableHead>Uploaded</TableHead>
								<TableHead class="text-right">Actions</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{#each cloudStorage.uploads as upload (upload.id)}
								<TableRow>
									<TableCell class="font-medium">{upload.filename}</TableCell>
									<TableCell>{formatBytes(upload.file_size)}</TableCell>
									<TableCell>{formatDate(upload.uploaded_at)}</TableCell>
									<TableCell class="text-right">
										<div class="flex justify-end gap-2">
											<Button 
												variant="ghost" 
												size="sm"
												onclick={() => handleDownload(upload)}
												disabled={isDownloading === upload.id}
											>
												{#if isDownloading === upload.id}
													<Loader2 class="size-4 animate-spin" />
												{:else}
													<Download class="size-4" />
												{/if}
											</Button>
											<Button 
												variant="ghost" 
												size="sm"
												onclick={() => handleDelete(upload.id)}
												disabled={isDeleting === upload.id}
											>
												<Trash2 class="size-4" />
											</Button>
										</div>
									</TableCell>
								</TableRow>
							{/each}
						</TableBody>
					</Table>
				</div>
			</CardContent>
		{/if}
	</Card>
{:else}
	<Card>
		<CardHeader>
			<CardTitle>Cloud Recordings</CardTitle>
			<CardDescription>Log in to view your cloud recordings</CardDescription>
		</CardHeader>
	</Card>
{/if}

