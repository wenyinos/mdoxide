# LSP status bar and server path overrides (Task 25)

## Status bar

- The bottom status bar shows a compact **LSP** line (left area, before toasts) built from `FerriteApp::lsp_status_bar_text()` in `src/app/file_ops.rs`.
- Per-server state comes from draining `LspManagerEvent::StatusChanged` in `handle_lsp_events()` into `FerriteApp::lsp_status_by_server`.
- Display rules:
  - **Disabled** — `lsp_enabled` is off.
  - **No workspace** — LSP on but no folder workspace open.
  - **No servers detected** — workspace open but `detect_servers_for_workspace()` found no supported extensions.
  - Otherwise — one segment per server key, e.g. `rust-analyzer: Ready`, joined with ` · `. Hover shows a multi-line breakdown; spawn errors include the worker message when useful.

The map resets when the workspace root changes or when override paths change (see below).

## Settings: `lsp_server_overrides`

- Field: `Settings::lsp_server_overrides: HashMap<String, String>` in `src/config/settings.rs`.
- Keys match language server identifiers used by detection (e.g. `rust-analyzer`, `clangd`). Values are optional filesystem paths to the executable; empty string or missing key means use the default program name on `PATH`.
- Applied in `AppState::start_lsp_for_workspace()` when building each `LspServerSpec` before `LspManager::start_server()`.

## Settings UI

- Editor tab → **Language servers** block below the LSP checkbox (`src/ui/settings.rs`).
- Passes `workspace_root` from `CentralPanel` into `SettingsPanel::show_inline()` so rows are listed for servers detected in the current workspace, plus any keys already present in `lsp_server_overrides`.
- Changing a path updates settings and triggers an LSP restart (stop all + `start_lsp_for_workspace`) when `handle_lsp_events()` detects a fingerprint change on `lsp_server_overrides` via `crate::lsp::overrides_fingerprint()`.

## Related

- [LSP server lifecycle](./lsp-server-lifecycle.md) — spawn, backoff, shutdown, `lsp_enabled` toggle.
