# Ferrite - AI Context

Rust (edition 2021) + egui 0.28 markdown editor. Immediate-mode GUI — no retained widget state, UI rebuilds each frame.

## Architecture

| Module | Purpose |
|--------|---------|
| `app/` | Main application (~15 modules: keyboard, file_ops, formatting, navigation, etc.) |
| `state.rs` | All application state (`AppState`, `Tab`, `TabKind`, `SpecialTabKind`, `FileType`) |
| `editor/ferrite/` | Rope-based editor (`ropey`): buffer, cursor, history, view, rendering, line_cache |
| `editor/widget.rs` | EditorWidget wrapper, integrates FerriteEditor via egui memory |
| `markdown/` | `editor.rs` (rendered view), `parser.rs` (comrak AST), `mermaid/` (11 diagram types), `csv_viewer.rs`, `tree_viewer.rs` |
| `terminal/` | Integrated terminal (PTY, VTE, screen, themes, split layouts) |
| `ui/` | Panels: ribbon, settings, file_tree, outline, search, terminal, productivity, frontmatter, welcome |
| `config/` | Settings persistence, session/crash recovery, snippets |
| `fonts.rs` | Font loading, lazy CJK, complex script lazy loading (11 families) |
| `theme/` | Light/dark themes |
| `vcs/git.rs`, `workspaces/`, `export/`, `preview/`, `platform/` | Git, folder mode, HTML export, sync scroll, platform-specific |

**FerriteEditor:** `src/editor/ferrite/` — rope-based, O(log n) ops, virtual scrolling, multi-cursor, code folding, IME/CJK. ~1x file size RAM. `EditorWidget` creates/retrieves from egui memory. Docs: `docs/technical/editor/architecture.md`

## Critical Patterns

```rust
// Always use saturating math for line indices
let idx = line_number.saturating_sub(1);

// Never unwrap in library code
if let Some(tab) = self.tabs.get_mut(self.active_tab) { ... }

// Prefer borrowing over clone
fn process(text: &str) -> Vec<&str> { text.lines().collect() }
```

## Common Gotchas

| Issue | Wrong | Right |
|-------|-------|-------|
| Byte vs char index | `text[start..end]` with char pos | Use `text.char_indices()` or byte offsets |
| Line indexing | Mixing 0/1-indexed | Explicit: `line.saturating_sub(1)` |
| CPU spin | Always `request_repaint()` | Use `request_repaint_after()` when idle |

## Conventions

- **Logging:** `log::info!`, `log::error!` (not println!)
- **i18n:** `t!("key.path")`, keys in `locales/en.yaml`
- **State:** `Tab` for per-tab, `AppState` for global
- **Errors:** User-facing via `show_toast()`, technical via `log::error!`
- **Large files (>1MB):** Hash-based `is_modified()`, reduced undo stack (10 vs 100), no `original_bytes`

## Where Things Live (common)

| Want to... | Look in... |
|------------|------------|
| Add a setting | `config/settings.rs` → `Settings` struct |
| Add keyboard shortcut | `app/keyboard.rs` → `handle_keyboard_shortcuts()` |
| Add/modify a UI panel | `ui/` → create or edit panel module |
| Modify editor core | `editor/ferrite/editor.rs` (behavior), `buffer.rs` (text), `view.rs` (viewport) |
| Modify markdown rendering | `markdown/editor.rs` or `markdown/widgets.rs` |
| Modify markdown parsing | `markdown/parser.rs` |
| Modify central panel | `app/central_panel.rs` |
| Add special tab | `state.rs` → `SpecialTabKind`, `app/central_panel.rs` |
| Add global/per-tab state | `state.rs` → `AppState` / `Tab` struct |
| Add i18n string | `locales/en.yaml` + `t!("key")` |
| Mermaid diagrams | `markdown/mermaid/` (flowchart has `types`, `parser`, `layout/`, `render/`) |
| Terminal | `terminal/` (pty, screen, widget, layout) |
| Git/VCS | `vcs/git.rs` |

## Performance Rules (FerriteEditor)

| Tier | When Allowed | Examples |
|------|--------------|----------|
| O(1) | Always | `line_count()`, `is_dirty()` |
| O(log N) | Always | `get_line(idx)`, index conversions |
| O(visible) | Per-frame | Syntax highlighting visible lines |
| O(N) | User-initiated ONLY | Find All, Save, Export |

**Never** call `buffer.to_string()` in per-frame code.

## Build & Test

```bash
cargo build          # Build debug
cargo run            # Run app
cargo clippy         # Lint
cargo test           # Run tests
```

## Current Focus

- v0.2.7 released (March 2026) — performance, polish, new features
- v0.2.8 in progress: Rendered view performance (Tasks 4-6 done), strict line breaks (Tasks 7-10 done), large file performance (Tasks 29-32 created), LSP integration (Tasks 23-26 done: module infra, lifecycle, status/overrides, inline diagnostics), HarfRust text shaping (Task 18 done — core module + LineCache pre-shape hook; shaped display/cursor follow-up pending)
- **Next up:** Cursor/hit-testing aligned to shaped clusters; extend shaped path to wrapped/syntax-highlighted lines; true OTL glyph ID rendering
- v0.3.0 planned: RTL/BiDi text support, mermaid crate extraction, math rendering

## Recently Changed

- **2026-03-28**: Task 26 — LSP inline diagnostics: `DiagnosticEntry`/`DiagnosticMap` in `lsp/state.rs`, full `initialize`/`initialized` handshake + stdout JSON-RPC read loop in `manager.rs`, `publishDiagnostics` routing to `AppState.diagnostics`, wavy squiggles in `highlights.rs`, hover tooltips in `editor.rs`, `didOpen`/`didChange` full-sync in `sync_active_doc_to_lsp()`, status bar error/warning counts. Doc: `docs/technical/lsp/lsp-inline-diagnostics.md`.
- **2026-03-28**: Task 25 — LSP status bar (`lsp_status_bar_text`, `StatusChanged` → `lsp_status_by_server`), `lsp_server_overrides` in settings + Editor “Language servers” paths, `overrides_fingerprint` restart in `handle_lsp_events`. Doc: `docs/technical/lsp/lsp-status-and-overrides.md`.
- **2026-03-28**: Task 22 — Shaped text measurements: `column_to_x_offset`/`x_to_column` helpers + `shaped_column_to_x`/`shaped_x_to_column` convenience wrappers in `shaping.rs`. Cursor rendering (`rendering/cursor.rs`), IME positioning (`editor.rs` `calculate_cursor_x`), mouse click-to-cursor (`mouse.rs`), and selection rendering (`selection.rs`) now use HarfRust-shaped advances for complex-script lines (Arabic, Bengali, etc.). Latin text unchanged. 24 shaping tests, 1367 total pass.
- **2026-03-28**: Task 19 — HarfRust pipeline integration: `group_clusters` in `shaping.rs`, `ShapedLine`/`ClusterGalley` in `line_cache.rs` with LRU shaped cache, per-cluster galley rendering in `editor.rs` for complex-script lines (non-wrapped, non-syntax). Falls back to standard egui path on failure or for Latin-only text.
- **2026-03-27**: Task 18 — HarfRust: `harfrust` 0.5.2 + `unicode-script`, `src/editor/ferrite/shaping.rs` (`shape_text`, `ShapedGlyph`), `fonts::ttf_bytes_for_font_id_shaping`, `LineCache` pre-shape for complex-script lines before galley build. Doc: `docs/technical/editor/harfrust-text-shaping.md`.
- **2026-03-25**: v0.2.8 Tasks 7-10 — Strict Line Breaks: `strict_line_breaks` in Settings, `hardbreaks` in parser, conditional `SoftBreak` rendering via egui memory, toggles in Settings UI + Welcome page. Docs: `docs/technical/markdown/strict-line-breaks.md`.
- **2026-03-25**: v0.2.8 Tasks 1-6 completed — macOS `.md` association (Task 1), Windows IME transform fix (Task 2), setext heading detection (Task 3), AST caching (Task 4), viewport culling with 500px overscan (Task 5), block-level height cache (Task 6). Tasks 29-32 created for large file performance. See `docs/technical/markdown/` and `docs/technical/editor/` for individual docs.
- **2026-03-25**: Task Master reset for v0.2.8: archived v0.2.7 tasks/PRD under `docs/ai-workflow/`.
- **2026-03-06**: v0.2.7 wrap-up — Tasks 32-45 (word wrap, zoom, CJK fonts, frontmatter panel, portal dialogs, etc.). See `docs/technical/` for individual docs.
