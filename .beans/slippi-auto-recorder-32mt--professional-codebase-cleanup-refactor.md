---
# slippi-auto-recorder-32mt
title: Professional codebase cleanup refactor
status: completed
type: task
priority: normal
created_at: 2026-03-20T05:47:08Z
updated_at: 2026-03-20T06:07:28Z
---

## Tasks

- [x] Stage 1: Dead Code & Metadata Cleanup
- [x] Stage 2: Deduplicate Utilities & Fix Types
- [x] Stage 3: Backend Robustness
- [x] Stage 4: Split Monolithic Frontend Components
- [x] Stage 5: Split Large Backend Modules


## Summary of Changes

### Stage 1: Dead Code & Metadata Cleanup
- Deleted duplicate `commands.svelte.ts` (never imported)
- Deleted unused `slippi_new.rs` (never referenced in mod)
- Fixed HTML title: "Vite + Svelte + TS" → "Peppi"
- Aligned Cargo.toml version to 1.0.3
- Removed unused shadcn components (calendar, kbd, popover) — reduced type errors by 25

### Stage 2: Deduplicate Utilities & Fix Types
- Replaced 4 duplicate `formatBytes()` with shared `formatFileSize` from `$lib/utils/format`
- Typed `scoreWindow(window: any)` → `GameWindow`
- Replaced `any` type aliases with `Record<string, unknown>` in slippi-stats.ts
- Converted `.then().catch()` to async/await in recordings store

### Stage 3: Backend Robustness
- Replaced `unwrap()` on mutex locks with `expect("Database mutex poisoned")`
- Replaced `expect()` in setup with `?` operator for proper error propagation
- Marked `uploadVideo()` stub with console.warn and early return

### Stage 4: Split Monolithic Frontend Components
- Split AppSettings.svelte (594→~130 lines) into GeneralSettings, SlippiSettings, RecordingSettings, WindowSelector
- Split AppLayout.svelte (390→~145 lines) into AppSidebar + window-scoring utility
- Skipped EditorControls (already well-structured at 404 lines)

### Stage 5: Split Large Backend Modules
- Split commands/library.rs (735 lines) → 4 sub-modules (recordings, stats, storage, slp)
- Split commands/clips.rs (545 lines) → 3 sub-modules (markers, editing, compression)
- Split database/recordings.rs (1541 lines) → 4 sub-modules (types, crud, stats, storage)
- All public APIs preserved via re-exports; lib.rs unchanged
