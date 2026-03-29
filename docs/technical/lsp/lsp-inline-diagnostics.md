# LSP Inline Diagnostics

**Task 26** — Wire the LSP client to surface `publishDiagnostics` results in the editor with inline squiggles, hover tooltips, incremental document sync, and status bar counts.

## Architecture

```
LSP server stdout
  → spawn_stdout_reader (background thread, JSON-RPC framing)
  → StdoutMsg::Message(Value) via mpsc channel
  → worker_main dispatches to handle_server_message()
  → handle_publish_diagnostics() converts to DiagnosticEntry vec
  → LspManagerEvent::Diagnostics sent to UI thread
  → handle_lsp_events() stores in AppState.diagnostics (DiagnosticMap)
  → FerriteEditor reads diagnostics via EditorWidget builder
  → render_diagnostic_squiggles() draws wavy underlines
  → Hover tooltip shows message at pointer position
```

## Data Structures

### DiagnosticEntry (`src/lsp/state.rs`)

```rust
pub struct DiagnosticEntry {
    pub start_line: usize,   // 0-indexed
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub severity: DiagnosticSeverity,  // Error | Warning | Information | Hint
    pub message: String,
    pub source: Option<String>,
}
```

### DiagnosticMap (`src/lsp/state.rs`)

`HashMap<PathBuf, Vec<DiagnosticEntry>>` with convenience methods:

| Method | Description |
|--------|-------------|
| `set(path, entries)` | Replace diagnostics for a file |
| `get(path)` | Get diagnostics for a file |
| `for_line_range(path, start, end)` | Filter to visible line range |
| `counts()` | Sum (errors, warnings) across all files |
| `clear()` | Remove all diagnostics |

## LSP Protocol Integration

### Handshake (`src/lsp/manager.rs`)

Task 26 implements the full LSP initialize/initialized handshake:

1. `send_initialize()` — sends `initialize` request with client capabilities (sync = full, diagnostics)
2. Server responds → `handle_server_message()` sets `server.initialized = true`
3. `send_initialized()` — sends empty `initialized` notification

### Document Sync

Full-text sync (`TextDocumentSyncKind::Full`):

| Notification | When | Source |
|-------------|------|--------|
| `textDocument/didOpen` | Tab becomes active + server is Ready | `sync_active_doc_to_lsp()` |
| `textDocument/didChange` | `last_edit_time` changes (debounced 300ms) | `sync_active_doc_to_lsp()` |
| `textDocument/didClose` | Not yet wired (future enhancement) |  |

Tracking state lives on `FerriteApp`:
- `lsp_opened_docs: HashSet<PathBuf>` — normalized paths already sent `didOpen`
- `lsp_doc_versions: HashMap<PathBuf, i32>` — version counter per path
- `lsp_last_edit_times: HashMap<PathBuf, Instant>` — snapshot of `Tab::last_edit_time` at last sync
- `lsp_last_change_sent: HashMap<PathBuf, Instant>` — debounce: when last `didChange` was sent

**Path normalization**: All path keys are normalized via `normalize_lsp_path()` (uppercased drive letter on Windows, `\\?\` prefix stripped) so that URIs from the server and filesystem paths from tabs always match.

### Diagnostics Flow

Server sends `textDocument/publishDiagnostics` → `handle_publish_diagnostics()`:
1. Converts URI to `PathBuf` via `uri_to_path()`
2. Parses each JSON diagnostic into `DiagnosticEntry`
3. Sends `LspManagerEvent::Diagnostics` to UI thread

`handle_lsp_events()` in `file_ops.rs` calls `self.state.diagnostics.set(path, entries)`.

## Rendering

### Squiggles (`src/editor/ferrite/highlights.rs`)

`render_diagnostic_squiggles()` draws wavy underlines for diagnostics overlapping visible lines:

- **Error** → red squiggle
- **Warning** → yellow/orange squiggle  
- **Info/Hint** → blue squiggle (dark) / darker blue (light)

The squiggle is drawn as a series of connected line segments forming a sine-wave pattern with amplitude 2px and period 4px, positioned at the baseline of the text.

Only diagnostics overlapping the visible line range (`start_line..end_line`) are processed.

### Hover Tooltips (`src/editor/ferrite/editor.rs`)

When the pointer hovers over the editor area:
1. Convert pointer position to buffer cursor via `pos_to_cursor()`
2. Check each diagnostic with `cursor_in_diagnostic_range()`
3. Show `egui::show_tooltip_at_pointer` with severity icon + message

### Status Bar (`src/app/status_bar.rs`)

Displays error/warning counts next to the LSP status line:
- Red label for errors: `"N err"`
- Yellow label for warnings: `"N warn"`
- Only shown when counts > 0

## Helper Functions (`src/lsp/mod.rs`)

| Function | Description |
|----------|-------------|
| `normalize_lsp_path(path)` | Normalize path for diagnostic map keys (uppercase drive, strip `\\?\`) |
| `path_to_uri(path)` | Convert `PathBuf` to `file://` URI (normalizes first, handles Windows) |
| `language_id_for_path(path)` | Map extension to LSP `languageId` (rust, python, go, etc.) |

## Key Files

| File | Changes |
|------|---------|
| `src/lsp/state.rs` | `DiagnosticSeverity`, `DiagnosticEntry`, `DiagnosticMap` |
| `src/lsp/manager.rs` | Full stdout read loop, initialize handshake, didOpen/didChange/didClose, publish diagnostics handling |
| `src/lsp/mod.rs` | `path_to_uri()`, `language_id_for_path()`, re-exports |
| `src/state.rs` | `AppState.diagnostics: DiagnosticMap` field |
| `src/app/mod.rs` | `lsp_opened_docs`, `lsp_doc_versions`, `lsp_content_hashes` tracking fields |
| `src/app/file_ops.rs` | `handle_lsp_events()` → Diagnostics variant, `sync_active_doc_to_lsp()` |
| `src/app/central_panel.rs` | Pass `tab_diagnostics` to `EditorWidget` |
| `src/editor/widget.rs` | `diagnostics` field + builder method on `EditorWidget` |
| `src/editor/ferrite/editor.rs` | `diagnostics` field, hover tooltip logic |
| `src/editor/ferrite/highlights.rs` | `render_diagnostic_squiggles()`, `draw_squiggle()`, `cursor_in_diagnostic_range()` |
| `src/app/status_bar.rs` | Error/warning count display |

## Testing

1. Open a Rust workspace with `rust-analyzer` installed
2. Introduce a syntax error (e.g., remove a semicolon)
3. Verify: red squiggle appears under the error
4. Hover over squiggle → tooltip shows diagnostic message
5. Fix the error → squiggle disappears after server re-analyzes
6. Status bar shows error/warning counts
