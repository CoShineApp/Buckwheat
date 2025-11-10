# Development Setup

## Quick Start (No Xcode/Visual Studio Needed!)

By default, the app runs in **dev mode** with mock recording, so you can test file watching, UI, and other features without heavy build dependencies.

### Dev Mode (Mock Recording)
```bash
# Just run normally - fast builds, no Xcode/VS needed!
bun tauri dev
```

This uses **mock recorders** which:
- ✅ Compile in seconds
- ✅ Work on any platform
- ✅ Let you test folder watching, settings, UI
- ✅ Simulate recording with logs (no actual screen capture)

### Production Mode (Real Recording)

When you're ready to test actual screen recording:

```bash
# Enable real recording features
bun tauri dev -- --features real-recording
```

**Requirements for real recording:**
- **macOS**: Xcode installed (`xcode-select --install` won't work, need full Xcode)
- **Windows**: Visual Studio Build Tools with Windows SDK

## What Each Mode Does

| Feature | Dev Mode (default) | Production Mode (--features real-recording) |
|---------|-------------------|-------------------------------------------|
| Compilation | Fast (~30s) | Slower (~5-10min first time) |
| Dependencies | None | Xcode (macOS) / VS Build Tools (Windows) |
| File Watching | ✅ Works | ✅ Works |
| Settings | ✅ Works | ✅ Works |
| UI Testing | ✅ Works | ✅ Works |
| Screen Recording | 🧪 Mock (logs only) | ✅ Real (captures screen) |

## Building for Release

Release builds automatically enable real recording:

```bash
# CI/CD does this automatically
bun tauri build -- --features real-recording
```

## Tips

1. **Start in dev mode** to test non-recording features quickly
2. **Switch to production mode** only when testing actual recording
3. **Use logs** - Mock recorder shows what would happen in console

## Common Issues

### "Xcode required" error when NOT using real-recording?
Make sure you're NOT passing `--features real-recording` flag.

### Want to test recording without Xcode/VS?
You can't - real recording requires platform-specific APIs. Use mock mode instead!

