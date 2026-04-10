---
# slippi-auto-recorder-j2iq
title: 'Stage 2 refactoring: dedup formatBytes, type any cleanup, async/await'
status: completed
type: task
priority: normal
created_at: 2026-03-20T05:50:01Z
updated_at: 2026-03-20T05:52:23Z
---

Replace local formatBytes with formatFileSize, type GameWindow in scoreWindow, replace any type aliases in slippi-stats.ts, convert .then/.catch to async/await in recordings store

## Summary of Changes

1. Replaced local `formatBytes()` with shared `formatFileSize` from `$lib/utils/format` in 4 files: AppSettings.svelte, StorageUsageBar.svelte, CloudVideos.svelte, PublicClipViewer.svelte
2. Typed `scoreWindow(window: any)` to `scoreWindow(window: GameWindow)` in AppLayout.svelte
3. Replaced `any` type aliases with `Record<string, unknown>` and removed eslint-disable comments in slippi-stats.ts
4. Converted `.then().catch()` to async/await in recordings.svelte.ts
5. Skipped settings.rs error type standardization (per plan)
