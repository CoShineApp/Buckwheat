<script lang="ts">
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "$lib/components/ui/card";
	import { Button } from "$lib/components/ui/button";
	import { Checkbox } from "$lib/components/ui/checkbox";
	import {
		Table,
		TableBody,
		TableCell,
		TableHead,
		TableHeader,
		TableRow,
	} from "$lib/components/ui/table";
	import { recordingsStore } from "$lib/stores/recordings.svelte";
	import { formatRelativeTime, formatFileSize, getStageName } from "$lib/utils/characters";
	import CharacterIcon from "./CharacterIcon.svelte";
	import { Play, FolderOpen, Trash2, Upload, RefreshCw } from "@lucide/svelte";
	import { invoke } from "@tauri-apps/api/core";
	import { handleTauriError, showSuccess } from "$lib/utils/errors";

	let isRefreshing = $state(false);

	async function refreshRecordings() {
		isRefreshing = true;
		try {
			await recordingsStore.refresh();
		} finally {
			isRefreshing = false;
		}
	}

	function handleSelectAll() {
		if (recordingsStore.allSelected) {
			recordingsStore.clearSelection();
		} else {
			recordingsStore.selectAll();
		}
	}

	async function handlePlayVideo(videoPath: string | null) {
		if (!videoPath) return;
		
		try {
			await invoke("open_video", { videoPath });
			console.log("🎬 Playing video:", videoPath);
		} catch (e) {
			handleTauriError(e, "Failed to open video");
		}
	}

	async function handleOpenFolder(videoPath: string | null) {
		if (!videoPath) return;
		
		try {
			await invoke("open_recording_folder", { videoPath });
			console.log("📂 Opened folder:", videoPath);
		} catch (e) {
			handleTauriError(e, "Failed to open folder");
		}
	}

	async function handleDelete(id: string, videoPath: string | null, slpPath: string) {
		if (!confirm("Are you sure you want to delete this recording?")) return;
		
		try {
			await invoke("delete_recording", { videoPath, slpPath });
			console.log("🗑️ Deleted recording:", id);
			showSuccess("Recording deleted successfully");
			await refreshRecordings();
		} catch (e) {
			handleTauriError(e, "Failed to delete recording");
		}
	}

	function handleUpload(id: string) {
		console.log("☁️ Uploading recording:", id);
		showSuccess("Upload to cloud (coming soon)");
	}
</script>

<Card>
	<CardHeader>
		<div class="flex items-center justify-between">
			<div>
				<CardTitle>Recordings</CardTitle>
				<CardDescription>
					{recordingsStore.recordings.length} total recordings
				</CardDescription>
			</div>
			<Button
				variant="outline"
				size="sm"
				onclick={refreshRecordings}
				disabled={isRefreshing}
			>
				<RefreshCw class={`size-4 ${isRefreshing ? "animate-spin" : ""}`} />
				Refresh
			</Button>
		</div>
	</CardHeader>
	<CardContent>
		{#if recordingsStore.recordings.length === 0}
			<!-- Empty State -->
			<div class="flex flex-col items-center justify-center py-12 text-center">
				<div class="mb-4 rounded-full bg-muted p-4">
					<Play class="size-8 text-muted-foreground" />
				</div>
				<h3 class="mb-2 text-lg font-semibold">No recordings yet</h3>
				<p class="mb-4 max-w-md text-sm text-muted-foreground">
					Start your first recording to see your Melee matches here. All recordings will be
					automatically paired with their replay files.
				</p>
			</div>
		{:else}
			<!-- Table -->
			<div class="rounded-md border">
				<Table>
					<TableHeader>
						<TableRow>
							<TableHead class="w-12">
								<Checkbox
									checked={recordingsStore.allSelected}
									indeterminate={recordingsStore.someSelected}
									onchange={handleSelectAll}
								/>
							</TableHead>
							<TableHead>Match</TableHead>
							<TableHead>Stage</TableHead>
							<TableHead>Duration</TableHead>
							<TableHead>Size</TableHead>
							<TableHead>Date</TableHead>
							<TableHead class="w-48 text-right">Actions</TableHead>
						</TableRow>
					</TableHeader>
					<TableBody>
						{#each recordingsStore.recordings as recording (recording.id)}
							<TableRow>
								<!-- Checkbox -->
								<TableCell>
									<Checkbox
										checked={recordingsStore.selectedIds.has(recording.id)}
										onchange={() => recordingsStore.toggleSelection(recording.id)}
									/>
								</TableCell>

								<!-- Match Info (Characters) -->
								<TableCell>
									{#if recording.slippi_metadata}
										<div class="flex items-center gap-3">
											<div class="flex items-center gap-1">
												{#each recording.slippi_metadata.players as player, idx}
													<CharacterIcon
														characterId={player.character_id}
														colorIndex={player.character_color}
														size="sm"
													/>
													{#if idx === 0 && recording.slippi_metadata.players.length > 1}
														<span class="mx-1 text-sm text-muted-foreground">vs</span>
													{/if}
												{/each}
											</div>
											<div class="flex flex-col gap-0.5">
												{#if recording.slippi_metadata.players.length >= 2}
													{@const winner = recording.slippi_metadata.players.find(p => p.port === recording.slippi_metadata?.winner_port)}
													{@const loser = recording.slippi_metadata.players.find(p => p.port !== recording.slippi_metadata?.winner_port)}
													
													{#if winner}
														<div class="flex items-center gap-1.5 text-sm">
															<span class="font-semibold text-green-600 dark:text-green-400">{winner.player_tag}</span>
															<span class="text-xs text-muted-foreground">defeated</span>
															<span class="font-medium text-muted-foreground">{loser?.player_tag || "Unknown"}</span>
														</div>
													{:else}
														<span class="text-sm font-medium">
															{recording.slippi_metadata.players[0]?.player_tag || "Player 1"}
															vs
															{recording.slippi_metadata.players[1]?.player_tag || "Player 2"}
														</span>
													{/if}
												{:else if recording.slippi_metadata.players[0]}
													<span class="text-sm font-medium">
														{recording.slippi_metadata.players[0].player_tag}
													</span>
												{/if}
											</div>
										</div>
									{:else}
										<span class="text-sm text-muted-foreground">No metadata</span>
									{/if}
								</TableCell>

								<!-- Stage -->
								<TableCell>
									<span class="text-sm">
										{recording.slippi_metadata ? getStageName(recording.slippi_metadata.stage) : "—"}
									</span>
								</TableCell>

								<!-- Duration -->
								<TableCell>
									<span class="text-sm">
										{recording.duration ? `${Math.floor(recording.duration / 60)}:${String(recording.duration % 60).padStart(2, '0')}` : "—"}
									</span>
								</TableCell>

								<!-- File Size -->
								<TableCell>
									<span class="text-sm">
										{recording.file_size ? formatFileSize(recording.file_size) : "—"}
									</span>
								</TableCell>

								<!-- Date -->
								<TableCell>
									<span class="text-sm text-muted-foreground">
										{formatRelativeTime(recording.start_time)}
									</span>
								</TableCell>

								<!-- Actions -->
								<TableCell>
									<div class="flex justify-end gap-1">
										<Button
											variant="ghost"
											size="sm"
											onclick={() => handlePlayVideo(recording.video_path)}
											title="Play video"
										>
											<Play class="size-4" />
										</Button>
										<Button
											variant="ghost"
											size="sm"
											onclick={() => handleOpenFolder(recording.video_path)}
											title="Open folder"
										>
											<FolderOpen class="size-4" />
										</Button>
										<Button
											variant="ghost"
											size="sm"
											onclick={() => handleUpload(recording.id)}
											title="Upload to cloud"
										>
											<Upload class="size-4" />
										</Button>
										<Button
											variant="ghost"
											size="sm"
											onclick={() => handleDelete(recording.id, recording.video_path, recording.slp_path)}
											title="Delete"
										>
											<Trash2 class="size-4 text-destructive" />
										</Button>
									</div>
								</TableCell>
							</TableRow>
						{/each}
					</TableBody>
				</Table>
			</div>
		{/if}
	</CardContent>
</Card>

