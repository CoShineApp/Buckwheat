<script lang="ts">
	import { Card, CardContent } from '$lib/components/ui/card';
	import { auth } from '$lib/stores/auth.svelte';
	import { HardDrive } from '@lucide/svelte';
	import { formatFileSize } from '$lib/utils/format';
</script>

{#if auth.isAuthenticated && auth.profile}
	<Card>
		<CardContent class="pt-6">
			<div class="space-y-2">
				<div class="flex items-center justify-between text-sm">
					<div class="flex items-center gap-2">
						<HardDrive class="size-4" />
						<span class="font-medium">Cloud Storage</span>
					</div>
					<span class="text-muted-foreground">
						{formatFileSize(auth.profile.storage_used)} / {formatFileSize(auth.profile.storage_limit)}
					</span>
				</div>

				<div class="w-full bg-secondary rounded-full h-2 overflow-hidden">
					<div 
						class="h-full bg-primary transition-all duration-300"
						class:bg-yellow-500={auth.storageUsedPercent > 80}
						class:bg-red-500={auth.storageUsedPercent > 95}
						style="width: {auth.storageUsedPercent}%"
					></div>
				</div>

				<p class="text-xs text-muted-foreground">
					{auth.storageUsedPercent.toFixed(1)}% used ({auth.storageUsedGB.toFixed(2)} GB / {auth.storageLimitGB} GB)
				</p>

				{#if auth.storageUsedPercent > 90}
					<p class="text-xs text-yellow-600 dark:text-yellow-400 font-medium">
						⚠️ Storage almost full! Consider deleting old recordings.
					</p>
				{/if}
			</div>
		</CardContent>
	</Card>
{/if}

