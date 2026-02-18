# LSP Integration for Ferrite

**Feature:** Language Server Protocol (LSP) Client Support  
**Status:** Proposal  
**Author:** Ferrite Core Team  
**Version:** 0.1  
**Roadmap:** v0.2.8 — [ROADMAP.md](../ROADMAP.md)

---

## 1. Problem Statement

Ferrite is a powerful editor with syntax highlighting, code folding, and a custom text editor — but it currently lacks semantic code intelligence. Developers working in Rust, Python, TypeScript, and other languages must switch to VS Code or a full IDE to get autocomplete, inline diagnostics, and go-to-definition. LSP support would close this gap without sacrificing Ferrite's lightweight, offline-first nature.

---

## 2. Goals & Non-Goals

### Goals

- Add **opt-in** LSP client support for any language that has a language server.
- Keep the experience **graceful** — Ferrite works perfectly without any language server installed.
- Stay **fully offline** — no network calls, all processing is local.
- **Minimize performance impact** on the egui render loop.

### Non-Goals

- Bundling language servers inside Ferrite's binary.
- Building a language server (we are a **client only**).
- Full IDE-level debugger integration (out of scope for this version).

---

## 3. User Stories

| As a… | I want… | So that… |
|-------|--------|----------|
| Rust developer | Inline error squiggles | I can catch compile errors without leaving Ferrite |
| Python developer | Autocomplete suggestions | I can write code faster |
| Developer | Go-to-definition | I can navigate large codebases without a separate IDE |
| Casual user | LSP to be invisible and opt-in | My markdown editing experience is unaffected |

---

## 4. Proposed Features (v1)

### 4.1 Core

- **Auto-detect** language server based on file extension (e.g. `.rs` → rust-analyzer, `.py` → pylsp).
- **Spawn** language server as a child process via stdio on workspace open.
- **Graceful fallback** if no language server is installed — show a dismissable notification with install instructions.
- **Proper server lifecycle** — start, restart on crash, shutdown on workspace close.

### 4.2 Editor Integration

- **Inline diagnostics** — error/warning squiggles under affected text with hover tooltip showing message.
- **Autocomplete popup** — trigger on typing or Ctrl+Space, navigable with arrow keys.
- **Hover documentation** — show docs/type info on cursor hover with configurable delay.
- **Go to Definition** — F12 or Ctrl+Click to jump to symbol definition.
- **Incremental document sync** — send only changed text ranges to the server, not the full document on every keystroke.

### 4.3 Settings & UX

- **LSP toggle per workspace** — opt-in, off by default.
- **Per-language server path override** in settings (for users with custom installs or multiple versions).
- **Status bar indicator** — LSP state: ⬤ rust-analyzer ready / ⟳ indexing... / ✗ not found.
- **Diagnostic count in status bar** — e.g. "2 errors, 1 warning".

---

## 5. Technical Approach

### 5.1 Crates

```toml
lsp-types = "0.97"   # LSP message type definitions
lsp-client = "0.1"   # JSON-RPC client transport (or equivalent)
tokio = { version = "1", features = ["full"] }  # Async runtime for background thread
```

*Note: Verify `lsp-client` availability and API; alternatives include `tower-lsp` (client side) or a minimal JSON-RPC + stdio wrapper.*

### 5.2 Architecture

```
┌─────────────────────────────────────┐
│           Ferrite egui UI            │
│  (renders diagnostics, completions)  │
└────────────────┬────────────────────┘
                 │ mpsc channel (results)
                 │ mpsc channel (requests)
┌────────────────▼────────────────────┐
│        LSP Manager (tokio)           │
│  - Spawns/manages server processes   │
│  - Sends/receives JSON-RPC           │
│  - ctx.request_repaint() on update   │
└────────────────┬────────────────────┘
                 │ stdio pipes
┌────────────────▼────────────────────┐
│     Language Server Process          │
│  (rust-analyzer, pylsp, tsserver…)   │
└─────────────────────────────────────┘
```

### 5.3 State (Ferrite / AppState)

```rust
struct LspState {
    diagnostics: HashMap<PathBuf, Vec<Diagnostic>>,
    completions: Option<Vec<CompletionItem>>,
    hover_info: Option<Hover>,
    server_status: HashMap<String, ServerStatus>,
}
```

- **Diagnostics** — keyed by file path; UI reads and draws squiggles from this map.
- **Completions / Hover** — optional; cleared or updated when new responses arrive; request IDs used to ignore stale results.
- **Server status** — per-workspace or per-language; drives status bar text and “not found” notifications.

### 5.4 Performance Constraints

- **No blocking the render loop** — LSP communication runs on a dedicated background thread (or tokio task); egui never blocks waiting for a server response.
- **Debounced autocomplete** — e.g. 150ms after last keystroke to avoid flooding the server.
- **Request cancellation** — stale completions from a previous cursor position must not be rendered; cancel in-flight requests when cursor or document changes.
- **RAM** — overhead is in the language server process, not Ferrite itself.

---

## 6. Rollout Plan (Phases)

| Phase | Scope |
|-------|--------|
| **Phase 1** | Infrastructure only — server spawn/shutdown, stdio transport, status bar indicator, opt-in toggle, graceful “not found” notification. |
| **Phase 2** | Inline diagnostics (highest user value, lowest UI complexity) — squiggles, hover tooltip, incremental sync, diagnostic count in status bar. |
| **Phase 3** | Hover documentation and go-to-definition. |
| **Phase 4** | Autocomplete popup (highest UI complexity) — debounce, cancellation, arrow-key navigation. |

MVP focus: Phase 1–2. Diagnostics alone are a major quality-of-life win and validate the architecture before investing in the more complex autocomplete UI.

---

## 7. Success Metrics

- With LSP toggled on, egui frame rate stays ≥ 60 fps on a mid-range machine.
- Language server detected and started within ~500 ms of workspace open.
- Zero LSP-related crashes when no language server is installed.
- Community feedback: feature request (e.g. GitHub issues) addressed.

---

## 8. Implementation Notes (How We Will Do This)

### 8.1 Where It Lives in the Codebase

- **New module:** `src/lsp/` (or `src/editor/lsp/`) containing:
  - `manager.rs` — spawn process, stdio, JSON-RPC send/receive, lifecycle.
  - `transport.rs` — stdio read/write, message framing (Content-Length + JSON).
  - `state.rs` — `LspState`, merging diagnostics/completions/hover from manager.
  - `detection.rs` — map file extension → server command (e.g. `rust-analyzer`, `pylsp`).
- **UI:** Status bar and diagnostic rendering in existing UI modules (e.g. `src/ui/` or central panel); squiggles drawn in the editor widget (e.g. in or beside the existing editor painting in `src/editor/`).
- **Settings:** New LSP section in `src/config/settings.rs` (or equivalent) for per-workspace toggle and per-language server path overrides.
- **App integration:** `AppState` (or equivalent) holds `LspState`; workspace open/close and tab switch trigger LSP init/shutdown and document sync.

### 8.2 Threading Model

- **Main thread:** egui UI, editor, user input. Only reads from `LspState` and sends requests via a channel (e.g. `mpsc::Sender<LspRequest>`).
- **Background thread/task:** Runs tokio (or a dedicated thread) that:
  - Receives requests (open doc, didChange, completion, hover, definition, etc.).
  - Talks to the language server over stdio.
  - Pushes results (diagnostics, completions, hover, definition location) back to main thread via channel; main thread updates `LspState` and calls `ctx.request_repaint()` so the next frame shows new data.

### 8.3 Document Sync

- On open: send `textDocument/didOpen` with full content (and URI, languageId).
- On edit: send `textDocument/didChange` with incremental updates (e.g. `TextDocumentContentChangeEvent` with `range` + `text`) so we don’t send the full buffer on every keystroke.
- Map Ferrite’s buffer/rope positions to LSP line/character (0-based) in the payloads.

### 8.4 Diagnostics Flow

- Server sends `textDocument/publishDiagnostics` (notification).
- Manager receives it, parses, and sends parsed diagnostics to main thread.
- Main thread updates `LspState.diagnostics` for that file and requests repaint.
- Editor painting: for each visible line, check diagnostics for that line/range and draw underlines (e.g. red for error, yellow for warning) and store hover text for the tooltip on hover.

### 8.5 Graceful Fallback

- If the server binary is not found (e.g. `which rust-analyzer` or configured path fails):
  - Do not spawn; set `ServerStatus::NotFound`.
  - Show a one-time dismissable notification: “Language server for Rust not found. Install rust-analyzer to enable diagnostics and completions.” with a link or short install instructions.
  - Status bar shows “✗ not found” and LSP toggle remains available so the user can disable the prompt.

---

## 9. References

- [Language Server Protocol Specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)
- [lsp-types crate](https://docs.rs/lsp-types/) — LSP data structures for Rust
- Ferrite [Editor Architecture](./technical/editor/architecture.md) and [FerriteEditor](./technical/editor/ferrite-editor.md) for integration points
