---
# slippi-auto-recorder-mgbe
title: Replace native recorders with OBS sidecar integration
status: completed
type: feature
priority: normal
created_at: 2026-03-20T06:39:26Z
updated_at: 2026-03-20T06:54:07Z
---

Replace 1400 lines of platform-specific recording code (windows-capture, screencapturekit) with OBS websocket control. OBS installed on-demand, managed as headless sidecar. Windows-only. Delete macOS recorder.

## Tasks

- [x] Phase 1: Create OBS lifecycle module (obs/mod, install, config, process, connection)
- [x] Phase 2: Create OBS recorder implementation (recorder/obs.rs)
- [x] Phase 4: Delete native recorders & clean up Cargo.toml
- [x] Phase 3: App lifecycle integration (exit handler)
- [x] Phase 5: Settings UI — OBS mode toggle
- [x] Phase 6: Frontend — OBS status & install prompt
- [x] Verify cargo check passes


## Summary of Changes

- Created `src-tauri/src/obs/` module with install, config, process, and connection submodules
- Created `src-tauri/src/recorder/obs.rs` implementing the Recorder trait via OBS websocket
- Deleted `windows_v2.rs` (836 lines) and `macos.rs` (578 lines) native recorders
- Removed `real-recording` feature flag and 8 platform-specific dependencies
- Added `obws`, `reqwest`, `dirs`, and `winreg` (Windows) dependencies
- Added exit handler in lib.rs to kill managed OBS on app shutdown
- Removed `configure_target_window()` from recording commands
- Added `ensure_obs_ready` and `install_obs` Tauri commands
- Added OBS mode settings (obsMode, obsPort, obsPassword) to frontend settings store
- Added OBS Integration UI section in RecordingSettings.svelte with managed/connect toggle
