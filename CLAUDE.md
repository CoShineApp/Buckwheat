# CLAUDE.md
**MOST IMPORTANT**: before you do anything else, run the `beans prime` command and heed its output.

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Peppi (Slippi Auto Recorder) — a Tauri 2 desktop app that automatically detects and records Super Smash Bros. Melee gameplay. File watcher detects new `.slp` replay files → triggers screen recording → stores videos with parsed replay metadata.

## Commands

**Use `bun`, not npm.**

- `bun run dev` — Start Vite dev server (frontend only, port 1420)
- `bun run tauri dev` — Full dev mode (Rust backend + frontend). Uses mock recorder by default.
- `bun run tauri build` — Production build
- `bun run check` — TypeScript + svelte-check
- `bun run lint` — Prettier + ESLint
- `bun run format` — Auto-format with Prettier

Rust-side:
- `cargo build` (in `src-tauri/`) — Debug build with mock recorder
- `cargo build --release --features real-recording` — Release with actual screen capture

## Architecture

### Event-Driven Flow
1. `GameDetector` (Rust) watches Slippi replay directory for new `.slp` files
2. Emits Tauri events (`slp-file-created`, `recording-started`, `recording-stopped`, etc.)
3. Frontend stores listen via `listen()` and update reactive state
4. SQLite database caches recording metadata for fast startup

### Frontend (src/)
- **Svelte 5 with runes** (`$state()`, `$props()`, `$derived()`, `$effect()`) — no legacy `$:` or `export let`
- **Stores** (`src/lib/stores/`): `.svelte.ts` files using runes for global state (recordings, settings, auth, clips, etc.)
- **Commands** (`src/lib/commands.ts`): Type-safe wrappers around `invoke()` for all Tauri commands
- **UI**: shadcn-svelte (bits-ui) components in `src/lib/components/ui/`. Always use shadcn-svelte before building custom components.
- **Pages**: Home, Settings, ReplayViewer, Cloud, Profile, Clips, Stats — routed via `navigation.svelte.ts`

### Backend (src-tauri/src/)
- **AppState** (`app_state.rs`): Shared Mutex-wrapped state (game detector, recorder, settings, database, clip markers)
- **Recorder trait** (`recorder/`): Platform-abstracted recording — `windows_v2.rs` (Windows.Graphics.Capture), `macos.rs` (ScreenCaptureKit), `mock.rs` (dev)
- **Commands** (`commands/`): Tauri command handlers — slippi, recording, library, clips, settings, window, cloud
- **Database** (`database/`): SQLite with WAL mode via rusqlite — recordings, game_stats, player_stats tables
- **Library** (`library/`): Background sync, storage limits, thumbnail generation
- **FFmpeg** (`ffmpeg-sidecar`): Used for clip extraction from recordings

### Feature Flags
- `real-recording` Cargo feature enables actual screen capture (Windows/macOS). Without it, the mock recorder is used.

## Key Conventions

- Use `$lib` alias for imports from `src/lib/`
- Tailwind CSS only — no inline `style` or `<style>` blocks
- Components: PascalCase filenames. Non-components: kebab-case `.svelte.ts`
- Formatting: tabs, semicolons, trailing commas
- Tauri commands return `Result` types in Rust; handle with try-catch in TypeScript
- Icons from `@lucide/svelte`
- Linear issue branches: `linear/[issue-id]-[short-description]`
