---
# slippi-auto-recorder-ygzh
title: 'Stage 4: Split monolithic frontend components'
status: completed
type: task
priority: normal
created_at: 2026-03-20T05:53:15Z
updated_at: 2026-03-20T05:56:24Z
---

Split AppSettings.svelte and AppLayout.svelte into smaller components. Skip EditorControls as it's well-structured.

## Summary of Changes

Split two monolithic frontend components into smaller, focused pieces:

**4A: AppSettings.svelte (586 -> ~130 lines)**
- Extracted `GeneralSettings.svelte` (Appearance/theme card)
- Extracted `SlippiSettings.svelte` (Slippi directory + watch toggle)
- Extracted `RecordingSettings.svelte` (Recording path, auto-start, storage limit/usage)
- Extracted `WindowSelector.svelte` (Game window detection, preview, selection)
- AppSettings.svelte is now a thin shell importing sub-components, keeping only Clips, Settings Storage, and Danger Zone inline

**4B: AppLayout.svelte (390 -> ~145 lines)**
- Extracted `scoreWindow()` to `src/lib/utils/window-scoring.ts`
- Extracted `AppSidebar.svelte` (full sidebar with navigation, status indicator, theme toggle)
- AppLayout.svelte keeps init logic and header/content area

**4C: EditorControls.svelte - Skipped** (well-structured at 404 lines)
