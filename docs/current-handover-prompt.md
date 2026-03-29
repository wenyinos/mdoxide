# Handover: v0.2.8 — Task 34

**Branch:** `master`

---

## Rules (DO NOT REMOVE)

- Never auto-update this file unless explicitly requested
- Complete the task before asking for the next instruction
- Run `cargo build` or `cargo check` after code changes
- Mark tasks in Task Master: `in-progress` when starting, `done` when verified
- Document by feature under `docs/technical/` or `docs/guides/` and add an entry to `docs/index.md`
- Prefer Task Master MCP tools over CLI when available
- Use context7 MCP when needed

---

## Environment

- **Project:** Ferrite (markdown editor, Rust + egui)
- **Path:** `g:\DEV\markDownNotepad`
- **Stack:** Rust 2021, egui 0.28

---

## Current Task

### Task 34 — Hide LSP Server CMD Window on Windows

| Field | Value |
|-------|-------|
| **ID** | 34 |
| **Priority** | high |
| **Dependencies** | Task 24 (LSP server lifecycle) — done |
| **Subtasks** | None |

**What:** Suppress visible `cmd.exe` console windows when spawning LSP server processes on Windows by setting the `CREATE_NO_WINDOW` process creation flag in `spawn_server()`.

**Implementation notes:**

1. **In `src/lsp/manager.rs` `spawn_server()` function** — add a `#[cfg(windows)]` block before `.spawn()`:
   ```rust
   #[cfg(windows)]
   {
       use std::os::windows::process::CommandExt;
       cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
   }
   ```

2. **Flag value:** `CREATE_NO_WINDOW = 0x08000000` — Microsoft docs confirmed. Flag is ignored for GUI apps; LSP servers (rust-analyzer, pyright) are console apps so it applies.

3. **Non-Windows platforms** — Unchanged; the `#[cfg(windows)]` gate makes it a no-op on Linux/macOS.

4. **Current `spawn_server()`** (line 288):
   ```rust
   fn spawn_server(spec: &LspServerSpec) -> Result<Child, String> {
       let mut cmd = Command::new(&spec.program);
       cmd.args(&spec.args);
       cmd.stdin(Stdio::piped());
       cmd.stdout(Stdio::piped());
       cmd.stderr(Stdio::piped());
       cmd.spawn().map_err(|e| format!("{e}"))
   }
   ```

**Test strategy:**
1. Windows: Open Rust file → rust-analyzer spawns → no cmd.exe window appears
2. Windows: Open .py → pyright spawns → no cmd window
3. Multiple servers: zero console windows
4. Linux/macOS: compile + run → no regression
5. Missing binary → notification appears, no zombie cmd window
6. Kill server → auto-restart (Task 24) → still no window

---

## Key Files

| File | Purpose |
|------|---------|
| `src/lsp/manager.rs` | `spawn_server()` — where LSP child process is created via `Command::new()` |

---

## Recent Context (from Task 31)

Task 31 eliminated per-frame string allocation in CSV Raw View:
- `csv_viewer.rs`: Added `raw_view_text`/`raw_view_hash` to `CsvViewerState`, blake3 hash-guarded rebuild in `show_raw_view()`
- Doc: `docs/technical/viewers/csv-raw-view-caching.md`

Task 34 is independent — a small platform-specific fix in the LSP module.

---

## Verification (end of session)

- [ ] Task 34 marked appropriately in Task Master
- [ ] `cargo build` or `cargo check` succeeds
- [ ] Feature documented in `docs/technical/` and `docs/index.md` if behavior or architecture changed materially
