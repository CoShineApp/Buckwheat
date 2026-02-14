<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { recordingsStore } from "$lib/stores/recordings.svelte";
	import { Trash2, FolderOpen, X } from "@lucide/svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { handleTauriError, showSuccess } from "$lib/utils/errors";

	// Derive directly from selectedIds so Svelte tracks the reactive state
	const selectedCount = $derived(recordingsStore.selectedIds.size);

	async function handleBulkDelete() {
		if (!confirm(`Are you sure you want to delete ${selectedCount} recording(s)?`)) return;
		
		try {
			await recordingsStore.deleteSelected();
			showSuccess(`Deleted ${selectedCount} recording(s)`);
		} catch (e) {
			handleTauriError(e, "Failed to delete recordings");
		}
	}

	async function handleOpenFolder() {
		// Open the first selected recording's folder
		const firstSelected = recordingsStore.recordings.find((r) =>
			recordingsStore.selectedIds.has(r.id)
		);
		
		if (firstSelected?.video_path) {
			try {
				await invoke("open_file_location", { path: firstSelected.video_path });
				showSuccess("Opened recordings folder");
			} catch (e) {
				handleTauriError(e, "Failed to open folder");
			}
		}
	}

	function handleClearSelection() {
		recordingsStore.clearSelection();
	}
</script>

{#if selectedCount > 1}
	<div class="fixed bottom-6 left-1/2 z-50 -translate-x-1/2 transform">
		<div
			class="flex items-center gap-3 rounded-lg border bg-background px-6 py-3 shadow-lg"
		>
			<span class="text-sm font-medium">
				{selectedCount} {selectedCount === 1 ? "recording" : "recordings"} selected
			</span>

			<div class="h-6 w-px bg-border"></div>

			<div class="flex gap-2">
				<Button variant="outline" size="sm" onclick={handleOpenFolder}>
					<FolderOpen class="size-4" />
					Open Folder
				</Button>

				<Button variant="destructive" size="sm" onclick={handleBulkDelete}>
					<Trash2 class="size-4" />
					Delete
				</Button>
			</div>

			<div class="h-6 w-px bg-border"></div>

			<Button variant="ghost" size="sm" onclick={handleClearSelection}>
				<X class="size-4" />
			</Button>
		</div>
	</div>
{/if}

