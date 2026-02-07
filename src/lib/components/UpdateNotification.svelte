<script lang="ts">
	import { updater } from '$lib/stores/updater.svelte';
	import { onMount } from 'svelte';
	import { Download, X, RefreshCw, Loader2 } from '@lucide/svelte';
	import { Button } from '$lib/components/ui/button';

	onMount(() => {
		// Check for updates on app start (with a delay to not block initial load)
		setTimeout(() => {
			updater.checkForUpdates();
		}, 3000);
	});

	const { state } = updater;
</script>

{#if state.available && state.update}
	<div class="fixed bottom-4 right-4 z-50 max-w-sm rounded-lg border border-emerald-500/30 bg-emerald-950/90 p-4 shadow-xl backdrop-blur-sm">
		<button
			onclick={() => updater.dismiss()}
			class="absolute right-2 top-2 text-emerald-400/60 hover:text-emerald-400"
		>
			<X class="h-4 w-4" />
		</button>

		<div class="flex items-start gap-3">
			<div class="rounded-full bg-emerald-500/20 p-2">
				<Download class="h-5 w-5 text-emerald-400" />
			</div>
			<div class="flex-1">
				<h4 class="font-semibold text-emerald-100">Update Available</h4>
				<p class="mt-1 text-sm text-emerald-300/80">
					Version {state.update.version} is ready to install
				</p>

				{#if state.downloading}
					<div class="mt-3">
						<div class="h-2 overflow-hidden rounded-full bg-emerald-900">
							<div
								class="h-full bg-emerald-500 transition-all duration-300"
								style="width: {state.progress}%"
							></div>
						</div>
						<p class="mt-1 text-xs text-emerald-400">{state.progress}% downloaded</p>
					</div>
				{:else}
					<Button
						onclick={() => updater.downloadAndInstall()}
						class="mt-3 bg-emerald-600 hover:bg-emerald-500"
						size="sm"
					>
						<RefreshCw class="mr-2 h-4 w-4" />
						Update Now
					</Button>
				{/if}
			</div>
		</div>
	</div>
{/if}

{#if state.checking}
	<div class="fixed bottom-4 right-4 z-50 flex items-center gap-2 rounded-lg border border-zinc-700 bg-zinc-900/90 px-4 py-2 text-sm text-zinc-400 shadow-lg backdrop-blur-sm">
		<Loader2 class="h-4 w-4 animate-spin" />
		Checking for updates...
	</div>
{/if}

{#if state.error}
	<div class="fixed bottom-4 right-4 z-50 max-w-sm rounded-lg border border-red-500/30 bg-red-950/90 p-3 text-sm text-red-300 shadow-lg backdrop-blur-sm">
		<button
			onclick={() => (state.error = null)}
			class="absolute right-2 top-2 text-red-400/60 hover:text-red-400"
		>
			<X class="h-4 w-4" />
		</button>
		Update check failed: {state.error}
	</div>
{/if}
