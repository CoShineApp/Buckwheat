<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { cloudStorage } from '$lib/stores/cloud-storage.svelte';
	import { auth } from '$lib/stores/auth.svelte';
	import { Cloud, Upload, X, CheckCircle, AlertCircle, XCircle } from '@lucide/svelte';
	import { toast } from 'svelte-sonner';

	function clearCompleted() {
		cloudStorage.clearCompletedQueue();
	}

	function removeItem(id: string) {
		cloudStorage.removeFromQueue(id);
	}

	function cancelUpload(id: string) {
		cloudStorage.cancelUpload(id);
		toast.info('Upload cancelled');
	}

	function getStatusIcon(status: string) {
		switch (status) {
			case 'completed':
				return CheckCircle;
			case 'error':
				return AlertCircle;
			case 'cancelled':
				return XCircle;
			default:
				return Upload;
		}
	}

	function getStatusColor(status: string) {
		switch (status) {
			case 'completed':
				return 'text-green-500';
			case 'error':
				return 'text-red-500';
			case 'cancelled':
				return 'text-yellow-500';
			case 'uploading':
				return 'text-blue-500';
			default:
				return 'text-muted-foreground';
		}
	}
</script>

{#if auth.isAuthenticated}
	<Card>
		<CardHeader>
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					<Cloud class="size-5" />
					<CardTitle>Upload Queue</CardTitle>
				</div>
				{#if cloudStorage.uploadQueue.length > 0}
					<Button variant="ghost" size="sm" onclick={clearCompleted}>
						Clear Completed
					</Button>
				{/if}
			</div>
			<CardDescription>
				{cloudStorage.uploadQueue.length === 0 
					? 'No uploads in progress' 
					: `${cloudStorage.uploadQueue.length} item(s) in queue`}
			</CardDescription>
		</CardHeader>

		{#if cloudStorage.uploadQueue.length > 0}
			<CardContent class="space-y-3">
				{#each cloudStorage.uploadQueue as item (item.id)}
					<div class="flex items-center gap-3 p-3 rounded-lg border bg-card">
						<svelte:component 
							this={getStatusIcon(item.status)} 
							class="size-5 {getStatusColor(item.status)}" 
						/>

						<div class="flex-1 min-w-0">
							<p class="text-sm font-medium truncate">
								{item.videoPath.split(/[\\/]/).pop()}
							</p>
							
							{#if item.status === 'uploading' || item.status === 'pending'}
								<div class="mt-1 w-full bg-secondary rounded-full h-1.5 overflow-hidden">
									<div 
										class="h-full bg-primary transition-all duration-300"
										style="width: {item.progress}%"
									></div>
								</div>
								<p class="text-xs text-muted-foreground mt-1">
									{item.progress.toFixed(0)}%
								</p>
							{:else if item.status === 'error'}
								<p class="text-xs text-red-500 mt-1">{item.error}</p>
							{:else if item.status === 'completed'}
								<p class="text-xs text-green-500 mt-1">Upload complete</p>
							{/if}
						</div>

						{#if item.status === 'uploading'}
							<Button 
								variant="ghost" 
								size="sm" 
								onclick={() => cancelUpload(item.id)}
								title="Cancel upload"
							>
								<X class="size-4" />
							</Button>
						{:else if item.status === 'completed' || item.status === 'error' || item.status === 'cancelled'}
							<Button 
								variant="ghost" 
								size="sm" 
								onclick={() => removeItem(item.id)}
							>
								<X class="size-4" />
							</Button>
						{/if}
					</div>
				{/each}
			</CardContent>
		{/if}
	</Card>
{:else}
	<Card>
		<CardHeader>
			<CardTitle>Cloud Upload</CardTitle>
			<CardDescription>Log in to upload recordings to cloud storage</CardDescription>
		</CardHeader>
	</Card>
{/if}

