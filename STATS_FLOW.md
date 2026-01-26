# Stats Saving Flow - From Game End to Database

## The Complete (Over-Engineered) Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ 1. GAME ENDS                                                    │
│    Rust recorder detects game end, stops recording              │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 2. RUST BACKEND                                                  │
│    - Stops video recording                                       │
│    - Emits "recording-stopped" event with video path            │
│    Location: src-tauri/src/commands/recording.rs:81            │
│    Location: src-tauri/src/commands/slippi.rs:264              │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 3. FRONTEND EVENT LISTENER                                      │
│    Listens to "recording-stopped" event                          │
│    Location: src/lib/stores/recordings.svelte.ts:366           │
│                                                                  │
│    Steps:                                                        │
│    a) Process clip markers (if any)                             │
│    b) Trigger cache sync: invoke("refresh_recordings_cache")   │
│    c) Refresh recordings list: this.refresh()                   │
│    d) Parse stats: this.parseStatsForRecording()                │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ├─────────────────┐
                     │                 │
                     ▼                 ▼
        ┌──────────────────┐  ┌──────────────────────┐
        │ 4A. CACHE SYNC   │  │ 4B. STATS PARSING   │
        │ (Rust/peppi)     │  │ (Frontend/slippi-js) │
        └──────────────────┘  └──────────────────────┘
                     │                 │
                     │                 │
                     ▼                 ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4A. CACHE SYNC PATH (Rust/peppi)                                │
│    Location: src-tauri/src/library/sync.rs                      │
│                                                                  │
│    Steps:                                                        │
│    1. Scans file system for .mp4 files                          │
│    2. For each recording:                                       │
│       - Finds matching .slp file                                │
│       - Parses .slp with Rust `peppi` library                   │
│       - Extracts metadata (stage, duration, winner_port)        │
│       - Creates RecordingRow                                     │
│       - Creates GameStatsRow (if .slp exists)                  │
│    3. Calls: database::upsert_recording()                       │
│    4. Calls: database::upsert_game_stats()                      │
│       └─> Creates game_stats row with:                          │
│           - stage, game_duration, total_frames                  │
│           - winner_port, loser_port (from peppi)                │
│           - player1_id, player2_id, ports, characters          │
│                                                                  │
│    ⚠️ PROBLEM: Only creates game_stats if .slp found during    │
│       sync. If .slp appears later, game_stats never created!   │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     │ (runs in parallel)
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 4B. STATS PARSING PATH (Frontend/slippi-js)                     │
│    Location: src/lib/stores/recordings.svelte.ts:549           │
│                                                                  │
│    Steps:                                                        │
│    1. Finds recording in list (by video path)                   │
│    2. Gets recordingId and slpPath                              │
│    3. Calls: parseAndSaveSlippiStats(slpPath, recordingId)      │
│       Location: src/lib/services/slippi-stats.ts:271           │
│                                                                  │
│    ┌──────────────────────────────────────────────────────────┐ │
│    │ parseAndSaveSlippiStats()                                │ │
│    │ 1. Calls: parseSlippiStats(slpPath, recordingId)         │ │
│    │    - Reads .slp file as binary                           │ │
│    │    - Parses with slippi-js library                       │ │
│    │    - Computes ALL player stats (kill_count, L-cancel,   │ │
│    │      openings, damage, etc.)                              │ │
│    │    - Determines winner by kill_count = 4                 │ │
│    │    - Returns GameStatsForDB object                       │ │
│    │                                                           │ │
│    │ 2. Calls: invoke("save_computed_stats", { stats })      │ │
│    │    Sends to Rust backend                                  │ │
│    └──────────────────────────────────────────────────────────┘ │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 5. RUST BACKEND: save_computed_stats()                         │
│    Location: src-tauri/src/commands/library.rs:203             │
│                                                                  │
│    Steps:                                                        │
│    1. Checks if game_stats exists:                              │
│       database::get_game_stats_by_id()                          │
│                                                                  │
│    2a. IF game_stats EXISTS:                                    │
│        - Updates game_stats with:                               │
│          UPDATE game_stats SET                                  │
│            match_id = ?, game_number = ?, game_end_method = ?, │
│            stage = ?, game_duration = ?, total_frames = ?,     │
│            is_pal = ?, played_on = ?                            │
│        ⚠️ DOES NOT update winner_port/loser_port                │
│                                                                  │
│    2b. IF game_stats DOES NOT EXIST:                           │
│        - Logs: "No existing game_stats found, will be          │
│          created by sync"                                        │
│        - DOES NOTHING! ❌                                       │
│        ⚠️ THIS IS THE BUG - game_stats never created!          │
│                                                                  │
│    3. Saves player_stats:                                       │
│       For each player in stats.players:                         │
│         - Creates PlayerStatsRow                                │
│         - Calls: database::upsert_player_stats()                │
│           └─> INSERT INTO player_stats (...)                    │
│               ON CONFLICT DO UPDATE                             │
│                                                                  │
│    ⚠️ PROBLEM: If cache sync hasn't run yet or .slp wasn't     │
│       found during sync, game_stats doesn't exist, so nothing  │
│       gets saved except player_stats!                           │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│ 6. DATABASE STATE                                               │
│                                                                  │
│    ✅ player_stats table:                                       │
│       - Has kill_count, character_id, port, etc.               │
│       - Source of truth for winner (kill_count = 4)             │
│                                                                  │
│    ❌ game_stats table:                                          │
│       - MAYBE has stage, duration, winner_port                  │
│       - ONLY if cache sync found .slp and created it            │
│       - If save_computed_stats ran first, game_stats is NULL!   │
│                                                                  │
│    Result:                                                       │
│    - Winners work (from player_stats.kill_count)               │
│    - Stages broken (game_stats.stage is NULL)                   │
│    - Total stats broken (needs game_stats for filtering)        │
└─────────────────────────────────────────────────────────────────┘
```

## The Problems

### Problem 1: Race Condition
- **Cache sync** (Rust/peppi) and **stats parsing** (Frontend/slippi-js) run in parallel
- If stats parsing finishes first, `game_stats` doesn't exist yet
- `save_computed_stats` only UPDATES, never CREATES `game_stats`
- Result: `game_stats` is NULL → no stage, no game-level metadata

### Problem 2: Two Parsing Paths
- **Rust/peppi** (cache sync): Creates `game_stats` with basic metadata
- **Frontend/slippi-js** (stats parsing): Creates `player_stats` with detailed stats
- They don't coordinate → `game_stats` might not exist when `player_stats` is saved

### Problem 3: Missing Creation Logic
- `save_computed_stats` assumes `game_stats` already exists
- If it doesn't exist, it just logs and continues
- Should CREATE `game_stats` if missing, not just update

## The Fix

**Option 1: Make save_computed_stats create game_stats if missing**
```rust
// In save_computed_stats()
if let Ok(Some(_existing)) = database::get_game_stats_by_id(&conn, &stats.recording_id) {
    // Update existing
} else {
    // CREATE game_stats if missing
    let game_stats = GameStatsRow {
        id: stats.recording_id.clone(),
        stage: Some(stats.stage),
        game_duration: Some(stats.game_duration),
        // ... etc
    };
    database::upsert_game_stats(&conn, &game_stats)?;
}
```

**Option 2: Simplify to single parsing path**
- Remove Rust/peppi parsing from cache sync
- Only use Frontend/slippi-js parsing
- Create both `game_stats` and `player_stats` in one go
