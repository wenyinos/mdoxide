# LSP On-Demand Server Startup

LSP servers start lazily when the active tab's file extension matches a known language server. This replaces the previous workspace-scan approach that eagerly spawned all detected servers when a workspace opened.

## Motivation

Eager startup scanned workspace files (depth 3) on every workspace open and spawned servers for all detected languages — even if the user never opened a file of that type. On-demand startup matches VS Code / Neovim behavior: servers only start when actually needed.

## How It Works

### Startup Flow

1. Each frame, `handle_lsp_events()` calls `sync_active_doc_to_lsp()`.
2. `sync_active_doc_to_lsp()` checks the active tab's file extension via `detect_lsp_server_for_path()`.
3. If no LSP server maps to the extension, nothing happens (e.g. `.md` files).
4. If a server is mapped:
   - **Disconnected** → `start_lsp_server_on_demand()` spawns it, applying any user overrides from `lsp_server_overrides`.
   - **Starting / Initializing** → skip (wait for handshake to complete).
   - **Ready** → send `didOpen` / `didChange` as before.
   - **Error** → skip (backoff restart in the worker handles crash recovery).
5. Once the server reaches `Ready`, the next frame's `sync_active_doc_to_lsp()` sends `didOpen`.

### Idle Shutdown

When a tab is closed:
- `cleanup_tab_state()` sends `textDocument/didClose` if no other tab shares the same file.
- Open-document count per server is decremented.
- When a server's document count reaches 0, a 30-second idle timer starts.
- `check_lsp_idle_shutdown()` (called each frame) shuts down servers that have been idle ≥ 30 seconds, setting their status to `Disconnected`.
- The server restarts on demand if the user opens another matching file.

### Status Bar

The status bar shows only servers that have been started (are known to `lsp_status_by_server`). It no longer scans the workspace directory each frame. When no servers are running, it shows "LSP" with a hover tooltip explaining on-demand behavior.

## Data Structures

| Field | Type | Purpose |
|-------|------|---------|
| `lsp_tab_server` | `HashMap<usize, (PathBuf, String)>` | Maps tab ID → (normalized path, server key) for tabs with an active `didOpen` |
| `lsp_open_doc_count` | `HashMap<String, usize>` | Number of open documents per server key |
| `lsp_idle_since` | `HashMap<String, Instant>` | When a server's doc count dropped to 0 |

## Edge Cases

| Scenario | Behavior |
|----------|----------|
| Multiple tabs, same extension | Single shared server; started once |
| Multiple tabs, same file | `didOpen` sent once; `didClose` only when all tabs close |
| Tab closed while server initializing | Tab-server mapping not yet created; no `didClose` needed |
| Server crash | Worker backoff restart handles it; next tab activation triggers `didOpen` |
| `lsp_enabled = false` | All tracking maps cleared; `sync_active_doc_to_lsp` not called |
| Override path changes | All servers stopped; on-demand restart uses new program path |
| Workspace change | All tracking maps cleared; servers restart on demand |
| Markdown / unknown extension | `detect_lsp_server_for_path` returns `None`; no server started |

## Key Files

| File | What Changed |
|------|-------------|
| `src/app/file_ops.rs` | `sync_active_doc_to_lsp()` — on-demand start; `start_lsp_server_on_demand()` — helper; `check_lsp_idle_shutdown()` — idle timer; `handle_lsp_events()` — simplified toggle/override; `lsp_status_bar_text()` — no workspace scan |
| `src/app/mod.rs` | New fields on `FerriteApp`; `cleanup_tab_state()` — sends `didClose` |
| `src/state.rs` | `start_lsp_for_workspace()` — now a no-op; removed call from `open_workspace()` |

## Related

- [LSP Server Lifecycle](./lsp-server-lifecycle.md) — spawn, backoff, shutdown
- [LSP Status & Overrides](./lsp-status-and-overrides.md) — status bar, override paths
- [LSP Inline Diagnostics](./lsp-inline-diagnostics.md) — squiggles, hover, didOpen/didChange sync
