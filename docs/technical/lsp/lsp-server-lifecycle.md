# LSP Server Lifecycle Management

Task 24 — wires the LSP client to real workspace/editor lifecycle.

## Overview

When a workspace is opened and LSP is enabled, Ferrite auto-detects relevant language servers from the workspace file tree, spawns them via `LspManager`, monitors their health with automatic crash recovery (exponential backoff), and shuts them down cleanly on workspace close or when LSP is disabled.

## Settings

| Setting | Type | Default | Location |
|---------|------|---------|----------|
| `lsp_enabled` | `bool` | `false` | Settings > Editor > "LSP (Language Servers)" |

Disabled by default so markdown-only users are unaffected. Toggling on mid-session triggers server detection if a workspace is open; toggling off stops all servers immediately.

## Architecture

```
┌──────────┐   LspCommand    ┌───────────────┐
│  UI/App  │ ───────────────► │  Worker Thread │
│  thread  │ ◄─────────────── │  (manager.rs)  │
└──────────┘  LspManagerEvent └───────────────┘
       │                            │
       │ poll_events()              │ recv_timeout(500ms)
       │ each frame                 │ health-check loop
       ▼                            ▼
  Toast on SpawnFailed         Auto-restart on crash
```

### Detection (`detection.rs`)

- `detect_servers_for_workspace(root)` — walks the workspace (max depth 3) collecting unique file extensions and mapping them to `LspServerSpec` entries.
- `install_hint(program)` — returns a user-friendly install command string for common servers (rust-analyzer, pylsp, gopls, etc.).

### Manager (`manager.rs`)

**Commands** (UI → Worker):
- `Start { server_key, spec }` — spawn or replace a server.
- `Stop { server_key }` — kill a specific server.
- `StopAll` — kill all servers (workspace close, LSP disable).
- `Restart { server_key }` — kill and re-spawn.
- `Shutdown` — exit the worker thread.

**Events** (Worker → UI):
- `StatusChanged { server_key, status }` — status transitions.
- `SpawnFailed { server_key, program, error }` — binary not found or spawn error; UI shows a toast with the install hint.

### Crash Recovery

The worker thread uses `recv_timeout(500ms)` so it periodically health-checks all active servers via `try_wait()`. When a server exits unexpectedly:

1. Exponential backoff: 1s → 2s → 4s → … → 30s max.
2. Backoff resets to 1s after 60s of sustained uptime.
3. Pending restarts are queued and fired when their backoff elapses.
4. Explicit `Stop` or `StopAll` cancels any pending restart for that key.

### Lifecycle Hooks

| Event | Action |
|-------|--------|
| Workspace open | `start_lsp_for_workspace()` — detect + start servers |
| Workspace close | `stop_all_servers()` |
| `lsp_enabled` toggled on | detect + start if workspace open |
| `lsp_enabled` toggled off | `stop_all_servers()` |
| App exit (`Drop`) | `Shutdown` command → worker kills all children |

### Toast Notifications

When `SpawnFailed` is received, the app shows a 6-second toast:

> LSP: rust-analyzer not found (The system cannot find the file specified.). Install via: rustup component add rust-analyzer

## Files Changed

| File | Changes |
|------|---------|
| `src/lsp/manager.rs` | `StopAll` command, `SpawnFailed` event, backoff constants, `recv_timeout` health-check loop, `PendingRestart` queue |
| `src/lsp/detection.rs` | `install_hint()`, `detect_servers_for_workspace()` |
| `src/lsp/mod.rs` | Re-exports for new public API |
| `src/config/settings.rs` | `lsp_enabled: bool` field |
| `src/state.rs` | `start_lsp_for_workspace()`, hooks in `open_workspace` / `close_workspace` |
| `src/app/mod.rs` | `lsp_was_enabled` field, `handle_lsp_events()` call in update loop |
| `src/app/file_ops.rs` | `handle_lsp_events()` — toggle detection, event polling, toast display |
| `src/ui/settings.rs` | Checkbox in Editor section |
