---
# slippi-auto-recorder-ap8e
title: Add macOS support to OBS sidecar integration
status: completed
type: task
priority: normal
created_at: 2026-04-10T02:47:45Z
updated_at: 2026-04-10T02:50:39Z
---

Extend the OBS sidecar to support macOS. Add macOS OBS detection (/Applications, brew), DMG/brew install, display_capture scene config, .app bundle process spawning, and update get_recorder() to return ObsRecorder on macOS.

## Tasks\n\n- [x] install.rs: macOS OBS detection + brew/DMG install\n- [x] config.rs: display_capture scene for macOS\n- [x] process.rs: .app bundle spawning\n- [x] recorder/mod.rs: return ObsRecorder on macOS\n- [x] Cargo.toml: winreg already cfg(windows)-only\n- [x] Verify cargo check passes


## Summary of Changes

- install.rs: Added macOS OBS detection (/Applications/OBS.app, brew --prefix), DMG download, hdiutil mount/copy/unmount install flow
- config.rs: Platform-specific capture sources — game_capture on Windows, display_capture on macOS
- process.rs: Set LSUIElement=1 env var on macOS to hide OBS dock icon
- recorder/mod.rs: get_recorder() now returns ObsRecorder on both Windows and macOS
- Cargo.toml: winreg was already cfg(windows)-only, no change needed
