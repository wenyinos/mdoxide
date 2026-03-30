# Session Handover

## Environment
- **Project:** Ferrite (markdown editor, Rust + egui)
- **Tech Stack:** Rust 2021, egui 0.28
- **Context file:** Always read `ai-context.md` first — it contains project rules, architecture, and conventions.
- **Branch:** master

## Core Handover Rules
- **NO HISTORY:** Do not include a project history document or past task details unless they directly impact this specific task.
- **SCOPE:** Focus ONLY on the current task detailed below.
- Run `cargo build` or `cargo check` after code changes.
- Mark tasks in Task Master: `in-progress` when starting, `done` when verified.
- Document by feature under `docs/technical/` and add an entry to `docs/index.md`.
- Prefer Task Master MCP tools over CLI when available.
- Use Context7 MCP when needed.

## Current Task: 35 — LSP Server Activation Based on Open Files

### Task Details
| Field | Value |
|-------|-------|
| **ID** | 35 |
| **Title** | LSP Server Activation Based on Open Files |
| **Complexity** | 8 |
| **Priority** | high |
| **Dependencies** | Tasks 24, 25, 33 — all done |

### Critical Note
**This task is likely already complete.** Task 33 (On-Demand LSP Server Startup) implemented ALL functionality described in Task 35:
- On-demand server start via `sync_active_doc_to_lsp()` in `src/app/file_ops.rs`
- Per-server doc count tracking (`lsp_open_doc_count`) and tab-to-server mapping (`lsp_tab_server`)
- 30-second idle shutdown via `check_lsp_idle_shutdown()`
- `didClose` on tab close via `cleanup_tab_state()` in `src/app/mod.rs`
- All edge cases (multi-tab same lang, rapid switching, crashes, unknown extensions)

**Recommended action:** Review the implementation (see docs below), verify the test scenarios, and mark as done if satisfied.

### Description
Implement on-demand LSP server activation that detects which language servers are needed based on currently open file tabs, starting servers only when matching files are opened and stopping them after idle timeout when the last tab for a language is closed.

### Implementation Details
1. **Tab Activation Detection** (`src/app/file_ops.rs`):
   - `sync_active_doc_to_lsp()` checks active tab's extension via `detect_lsp_server_for_path()`
   - If server status is `Disconnected` → calls `start_lsp_server_on_demand()`
   - If `Starting`/`Initializing` → waits; if `Ready` → sends `didOpen`/`didChange`

2. **Server Lifecycle** (`src/app/file_ops.rs` + `src/app/mod.rs`):
   - `lsp_open_doc_count: HashMap<String, usize>` tracks open docs per server
   - `lsp_tab_server: HashMap<usize, (PathBuf, String)>` maps tab ID to (path, server_key)
   - `cleanup_tab_state()` sends `didClose` on tab close and decrements count
   - `check_lsp_idle_shutdown()` stops servers idle ≥ 30s

3. **Extension Mapping** (`src/lsp/detection.rs`):
   - `detect_lsp_server_for_path()` maps extensions to server specs
   - User overrides applied via `lsp_server_overrides` in settings

### Key Files
| File | Purpose |
|------|---------|
| `src/app/file_ops.rs` | `sync_active_doc_to_lsp()`, `start_lsp_server_on_demand()`, `check_lsp_idle_shutdown()`, `handle_lsp_events()` |
| `src/app/mod.rs` | `FerriteApp` fields (`lsp_tab_server`, `lsp_open_doc_count`, `lsp_idle_since`), `cleanup_tab_state()` |
| `src/lsp/detection.rs` | `detect_lsp_server_for_path()` extension-to-server mapping |
| `src/lsp/manager.rs` | `LspManager` — server spawn, lifecycle, crash recovery |
| `src/state.rs` | `start_lsp_for_workspace()` (now no-op) |
| `docs/technical/lsp/lsp-on-demand-startup.md` | Full feature documentation |

### Test Strategy
1. Fresh workspace: Open Rust tab → server spawns. Switch to Markdown tab → NO server spawn.
2. Tab reactivation: Close Rust tab → idle timer starts. Reopen within 30s → no respawn.
3. Idle shutdown: Close all Rust tabs → server shuts after 30s (log check).
4. Multi-lang: Rust+Python tabs → 2 servers. Close all Rust → 1 server.
5. Multiple tabs same language: Open two Rust tabs → one server. Close one → server stays. Close both → idle shutdown.
6. Unknown extensions: Open .xyz file → no server, no errors.
7. Crash recovery: Kill rust-analyzer → reactivate Rust tab → restarts.
8. Status bar shows 'rust-analyzer: Starting…→Ready'. Toggle `lsp_enabled=false` → no spawn.

## Verification
Before starting:
`cargo build`

## Model Selection
Task complexity 8 → **Default model**
