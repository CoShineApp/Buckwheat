<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { cloudStorage } from '$lib/stores/cloud-storage.svelte';
	import { Upload, X, CheckCircle, AlertCircle, XCircle, Loader2 } from '@lucide/svelte';
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
			case 'uploading':
			case 'pending':
				return Loader2;
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
			case 'pending':
				return 'text-blue-500';
			default:
				return 'text-muted-foreground';
		}
	}

	function isActiveUpload(status: string) {
		return status === 'uploading' || status === 'pending';
	}

	const hasItems = $derived(cloudStorage.uploadQueue.length > 0);
	const activeCount = $derived(cloudStorage.uploadQueue.filter(i => isActiveUpload(i.status)).length);
	const hasCompletedOrFailed = $derived(
		cloudStorage.uploadQueue.some(i => ['completed', 'error', 'cancelled'].includes(i.status))
	);
</script>

<!-- Only show when there are items in the queue -->
{#if hasItems}
	<Card>
		<CardHeader class="py-3 px-4">
			<div class="flex items-center justify-between">
				<div class="flex items-center gap-2">
					{#if activeCount > 0}
						<Loader2 class="size-4 animate-spin text-blue-500" />
					{:else}
						<Upload class="size-4" />
					{/if}
					<CardTitle class="text-sm">
						{#if activeCount > 0}
							Uploading {activeCount} file{activeCount !== 1 ? 's' : ''}...
						{:else}
							Upload Queue
						{/if}
					</CardTitle>
				</div>
				{#if hasCompletedOrFailed}
					<Button variant="ghost" size="sm" class="h-7 text-xs" onclick={clearCompleted}>
						Clear
					</Button>
				{/if}
			</div>
		</CardHeader>

		<CardContent class="py-2 px-4 space-y-2">
			{#each cloudStorage.uploadQueue as item (item.id)}
				<div class="flex items-center gap-2 py-1.5 px-2 rounded-md border bg-muted/30">
					<svelte:component 
						this={getStatusIcon(item.status)} 
						class="size-4 flex-shrink-0 {getStatusColor(item.status)} {isActiveUpload(item.status) ? 'animate-spin' : ''}" 
					/>

					<div class="flex-1 min-w-0">
						<p class="text-xs font-medium truncate">
							{item.videoPath.split(/[\\/]/).pop()}
						</p>
						
						{#if isActiveUpload(item.status)}
							<div class="mt-1 w-full bg-secondary rounded-full h-1 overflow-hidden">
								<div 
									class="h-full bg-primary transition-all duration-300"
									style="width: {item.progress}%"
								></div>
							</div>
						{:else if item.status === 'error'}
							<p class="text-[10px] text-red-500 truncate">{item.error}</p>
						{/if}
					</div>

					<span class="text-[10px] text-muted-foreground flex-shrink-0">
						{#if isActiveUpload(item.status)}
							{item.progress.toFixed(0)}%
						{:else if item.status === 'completed'}
							Done
						{:else if item.status === 'error'}
							Failed
						{:else if item.status === 'cancelled'}
							Cancelled
						{/if}
					</span>

					{#if item.status === 'uploading'}
						<Button 
							variant="ghost" 
							size="sm" 
							class="h-6 w-6 p-0"
							onclick={() => cancelUpload(item.id)}
							title="Cancel upload"
						>
							<X class="size-3" />
						</Button>
					{:else if item.status !== 'pending'}
						<Button 
							variant="ghost" 
							size="sm"
							class="h-6 w-6 p-0" 
							onclick={() => removeItem(item.id)}
							title="Remove"
						>
							<X class="size-3" />
						</Button>
					{/if}
				</div>
			{/each}
		</CardContent>
	</Card>
{/if}
