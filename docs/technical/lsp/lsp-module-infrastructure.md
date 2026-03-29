# LSP module infrastructure

## Overview

Foundation for a Language Server Protocol **client** in Ferrite: a background worker with `mpsc` channels, JSON-RPC **stdio** framing (`Content-Length` headers), extension-to-server hints, and `ServerStatus` tracking. Editor features (diagnostics, completions, `initialize` handshake) are deferred to later tasks; Task 24+ wires lifecycle to workspaces.

## Key Files

- `src/lsp/mod.rs` — Public exports; `#![allow(dead_code)]` until UI calls the API each frame.
- `src/lsp/state.rs` — `ServerStatus` enum.
- `src/lsp/detection.rs` — `LspServerSpec`, `detect_lsp_server_for_path()`.
- `src/lsp/transport.rs` — Framing helpers and `MessageReader` for incremental reads.
- `src/lsp/manager.rs` — `LspManager`, worker thread, process spawn, stdout/stderr drains.
- `src/state.rs` — `AppState::lsp: LspManager`.

## Implementation Details

- **Threading:** One dedicated worker thread owns `HashMap` of child processes; UI sends `Start` / `Stop` / `Restart` / `Shutdown` via `mpsc`.
- **Transport:** Messages are UTF-8 JSON bodies prefixed with `Content-Length: <n>\r\n\r\n` per LSP spec.
- **Pipes:** Stdout/stderr are read on helper threads so servers cannot deadlock on full pipes before protocol I/O is implemented.
- **Detection:** Built-in mappings (e.g. `.rs` → `rust-analyzer`) are defaults only; user overrides belong in settings later.

## Dependencies Used

- **serde_json** — Serialize/deserialize JSON-RPC payloads as `serde_json::Value` in the transport layer.
- **std** — `mpsc`, `process`, `thread`, `io` (no extra LSP crates yet).

## Usage

- Obtain the manager from `app_state.lsp`.
- Call `start_server(key, spec)` with a key such as `"rust"` and an `LspServerSpec` from `detect_lsp_server_for_path` or custom paths.
- Each frame (or on idle), `poll_events()` and handle `LspManagerEvent::StatusChanged` for UI/status bar.

## Related docs

- [LSP Integration Plan](../../lsp-integration-plan.md) — product scope and roadmap.
