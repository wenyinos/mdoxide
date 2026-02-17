# Single-Instance Protocol

## Overview

Ensures only one Ferrite window runs at a time. When a second instance is launched (e.g., double-clicking a file in Windows Explorer), it forwards file paths to the already-running instance via local TCP, which opens them as tabs. The second process then exits immediately.

## Key Files

| File | Purpose |
|------|---------|
| `src/single_instance.rs` | Protocol implementation: lock file, TCP listener/client, polling |
| `src/main.rs` | Instance check before GUI launch (`try_acquire_instance`) |
| `src/app/mod.rs` | Stores `SingleInstanceListener`, polls in `update()` |
| `src/app/file_ops.rs` | `handle_instance_paths()` — opens received paths as tabs |

## Protocol

1. **Lock file**: `{config_dir}/instance.lock` contains the TCP port of the running instance as plain text
   - Windows: `%APPDATA%\ferrite\instance.lock`
   - Linux: `~/.config/ferrite/instance.lock`
   - macOS: `~/Library/Application Support/ferrite/instance.lock`

2. **Startup flow**:
   ```
   Read lock file → port exists?
     YES → connect to port
       SUCCESS → send paths, exit (secondary)
       FAIL → stale lock, delete, become primary
     NO → become primary
   ```

3. **Primary instance**:
   - Binds `TcpListener` on `127.0.0.1:0` (OS picks port)
   - Sets listener to non-blocking mode
   - Writes port to lock file
   - Polls listener every frame in `update()` loop

4. **Secondary instance**:
   - Connects to `127.0.0.1:{port}` with 1s timeout
   - Sends file paths as UTF-8 lines (one per line)
   - Sends `__FOCUS__` if no paths (just bring window forward)
   - Exits cleanly via `return Ok(())`

5. **Cleanup**: Lock file removed on `Drop` of `SingleInstanceListener`

## Edge Cases

| Scenario | Behavior |
|----------|----------|
| Stale lock (crashed instance) | TCP connect fails → lock deleted → new primary |
| No paths (bare launch) | `__FOCUS__` signal sent → existing window focused |
| Config dir unavailable | Warning logged, app runs without single-instance |
| Listener bind failure | App runs normally, just no IPC |
| Directory path received | Opened as workspace (same as drag-and-drop) |
| Multiple paths | All opened as tabs; first directory becomes workspace |

## Integration Points

- Polls alongside `handle_dropped_files()` and `handle_file_watcher_events()` in the update loop
- Uses `ViewportCommand::Focus` to bring window to front
- Reuses `state.open_file()` and `state.open_workspace()` for consistent behavior
- Lock file stored in same config directory as other Ferrite config (`get_config_dir()`)

## No New Dependencies

Uses only `std::net` (TcpListener/TcpStream) and `std::io` — no external crates added.
