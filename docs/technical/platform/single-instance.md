# Single-Instance Protocol

## Overview

Ensures only one Ferrite window runs at a time. When a second instance is launched (e.g., double-clicking a file in Windows Explorer), it forwards file paths to the already-running instance via local TCP, which opens them as tabs. The second process then exits immediately.

## Key Files

| File | Purpose |
|------|---------|
| `src/single_instance.rs` | Protocol implementation: lock file, TCP client, background accept thread, channel-based path delivery |
| `src/main.rs` | Instance check **early** in startup (before config/icon loading) via `try_acquire_instance` |
| `src/app/mod.rs` | Stores `SingleInstanceListener`, provides egui context for repaint wakeup |
| `src/app/file_ops.rs` | `handle_instance_paths()` — drains channel and opens received paths as tabs |

## Architecture

```
Secondary instance                 Primary instance
─────────────────                  ────────────────

 main() starts                      Background thread
   ↓                                (blocking accept loop)
 Parse CLI args                          ↓
   ↓                              listener.accept() ← blocks
 Read lock file → port                  ↓
   ↓                              Connection arrives!
 TCP connect(port)  ───────────→  Read paths from stream
   ↓                                     ↓
 Write paths + shutdown(Write)    Send paths via mpsc channel
   ↓                                     ↓
 Exit process                     ctx.request_repaint() ← wakes UI
                                         ↓
                                  UI thread: poll() drains channel
                                         ↓
                                  Open file as tab (instant)
```

## Protocol

1. **Lock file**: `{config_dir}/instance.lock` contains the TCP port of the running instance as plain text
   - Windows: `%APPDATA%\ferrite\instance.lock`
   - Linux: `~/.config/ferrite/instance.lock`
   - macOS: `~/Library/Application Support/ferrite/instance.lock`

2. **Startup flow** (runs early, before config/logging/icon loading):
   ```
   Parse CLI args → Read lock file → port exists?
     YES → connect to port (500ms timeout)
       SUCCESS → send paths, shutdown(Write), exit
       FAIL → stale lock, delete, become primary
     NO → become primary
   ```

3. **Primary instance**:
   - Binds `TcpListener` on `127.0.0.1:0` (OS picks port)
   - Writes port to lock file
   - Spawns background thread (`single-instance-accept`) that blocks on `accept()`
   - Accepted connections are read with 100ms timeout (localhost data arrives in <1ms)
   - Paths sent to UI via `mpsc::channel`; UI woken via `ctx.request_repaint()`
   - UI thread drains channel with `try_recv()` (non-blocking, nanoseconds)

4. **Secondary instance** (exits in <100ms):
   - Connects to `127.0.0.1:{port}` with 500ms timeout
   - Sends file paths as UTF-8 lines (one per line)
   - Sends `__FOCUS__` if no paths (just bring window forward)
   - Calls `stream.shutdown(Write)` to send FIN immediately
   - Exits cleanly via `return Ok(())`

5. **Cleanup**: Lock file removed on `Drop` of `SingleInstanceListener`

## Performance Design

The protocol is designed for instant response (<50ms end-to-end):

| Component | Technique | Latency |
|-----------|-----------|---------|
| Secondary startup | Single-instance check runs before config/logging/icon loading | ~50ms |
| TCP delivery | Explicit `shutdown(Write)` sends EOF immediately | <1ms |
| Primary accept | Dedicated blocking thread, no polling delay | <1ms |
| UI wakeup | `ctx.request_repaint()` from background thread bypasses idle intervals | <1ms |
| Channel drain | `mpsc::try_recv()` is non-blocking | nanoseconds |

Previously, the protocol used per-frame polling of a non-blocking listener on the UI thread. In deep idle (500ms repaint interval), this caused 1-2 second delays. The background thread + repaint wakeup architecture eliminates this entirely.

## Edge Cases

| Scenario | Behavior |
|----------|----------|
| Stale lock (crashed instance) | TCP connect fails → lock deleted → new primary |
| No paths (bare launch) | `__FOCUS__` signal sent → existing window focused |
| Config dir unavailable | Warning logged, app runs without single-instance |
| Listener bind failure | App runs normally, just no IPC |
| Directory path received | Opened as workspace (same as drag-and-drop) |
| Multiple paths | All opened as tabs; first directory becomes workspace |
| App shutting down | Channel sender dropped → accept thread exits cleanly |

## Integration Points

- Background thread provides repaint context via `Arc<Mutex<Option<egui::Context>>>`
- `handle_instance_paths()` calls `set_repaint_ctx()` each frame (cheap when already set)
- Uses `ViewportCommand::Focus` to bring window to front
- Reuses `state.open_file()` and `state.open_workspace()` for consistent behavior
- Lock file stored in same config directory as other Ferrite config (`get_config_dir()`)

## No New Dependencies

Uses only `std::net` (TcpListener/TcpStream), `std::sync::mpsc`, `std::thread`, and `egui::Context` — no external crates added.
