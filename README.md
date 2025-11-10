# Peppi

A screen recording tool for Melee. That's it. No ads, no bloat, just Melee.

## Why This Exists

I like Melee. I like [Ascent](https://github.com/ascent-org/windows-record). I wanted to bring Ascent into Melee.

Every other screen recording tool I've tried is filled with obtrusive ads and bloated with features I don't need. I only care about Melee, so I built a tool that only cares about Melee too.

## What It Does

- Watches your Slippi replay folder and automatically starts recording when games begin
- Records your screen using Ascent's `windows-record` library (it's really good)
- Pairs recordings with their `.slp` replay files automatically
- No ads, no nonsense, just Melee

## Tech Stuff

Built with:
- **Svelte 5** + **TypeScript** for the UI
- **Tauri 2** + **Rust** for the backend
- **Tailwind CSS** for styling
- **Ascent's windows-record** for actual recording (Windows only for now)

## Getting Started

### Prerequisites

- Node.js 18+ and npm
- Rust 1.70+ and Cargo
- Tauri CLI (or just use `npx tauri`)

### Running It

```bash
# Install dependencies
npm install

# Run in dev mode
npm run tauri dev

# Build it
npm run tauri build
```

## How It Works

1. Peppi watches your Slippi replay folder (auto-detects the path)
2. When a new `.slp` file appears, it starts recording
3. Recording gets paired with the replay file
4. You get nice organized recordings without doing anything

## Current Status

**Working:**
- ✅ Detects Slippi paths on Windows/macOS/Linux
- ✅ File watching setup
- ✅ Basic UI with sidebar
- ✅ Error handling

**In Progress:**
- 🔄 Actually recording stuff (Windows using `windows-record`)
- 🔄 Auto-starting recordings when games begin
- 🔄 UI for managing recordings

**Maybe Later:**
- macOS/Linux recording support
- Cloud upload (maybe?)
- Dolphin window detection
- Batch processing

## Project Structure

```
src/                    # Frontend (Svelte)
src-tauri/src/          # Backend (Rust)
  ├── commands/         # Tauri commands
  ├── game_detector/    # Watches for .slp files
  └── recorder/         # Actually records stuff
```

## Configuration

Peppi finds your Slippi folder automatically:
- **Windows**: `%APPDATA%\Slippi Launcher\netplay`
- **macOS**: `~/Library/Application Support/Slippi Launcher/netplay`
- **Linux**: `~/.config/Slippi Launcher/netplay`

You can change this in settings if needed.

## Contributing

Sure, why not? If you have ideas to make recording Melee easier, feel free to contribute. This is a community tool.

## Thanks

- **[Slippi](https://slippi.gg)** - For making Melee replays a thing
- **[Ascent](https://github.com/ascent-org/windows-record)** - For the recording library
- **The Melee Community** - For keeping this game alive

## License

See LICENSE file.

---

Made for Melee players, by a Melee player.
