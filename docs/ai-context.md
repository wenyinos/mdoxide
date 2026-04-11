# Ferrite - AI Context

Rust (edition 2021) + egui 0.28 markdown editor. Immediate-mode GUI — no retained widget state, UI rebuilds each frame.

## Rules (DO NOT UPDATE)
- Never auto-update this file or `current-handover-prompt.md` — only update when explicitly requested.
- Only do the task specified; do not start the next task or go over scope.
- Run `cargo build` or `cargo check` after changes to verify code compiles.
- Follow existing code patterns and conventions.
- Document by feature (e.g., `lsp-inline-diagnostics.md`), not by task.
- Update `docs/index.md` when adding new documentation.
- Use Context7 MCP tool to fetch library documentation when needed (resolve library ID first, then fetch docs).

## Tech Stack
- **Language:** Rust 2021, egui 0.28 + eframe (immediate-mode GUI)
- **Text:** ropey (rope buffer), comrak (Markdown AST), syntect (syntax highlighting), harfrust (OTL shaping)
- **Terminal:** portable-pty + vte | **VCS:** git2 | **Dialogs:** rfd | **i18n:** rust-i18n | **Hashing:** blake3 | **PDF:** hayro
- **Memory:** mimalloc (Windows), jemalloc (Unix)

## Architecture

| Module | Purpose |
|--------|---------|
| `app/` | Main application (~15 modules: keyboard, file_ops, formatting, navigation, etc.) |
| `state.rs` | All application state (`AppState`, `Tab`, `TabKind`, `SpecialTabKind`, `FileType`) |
| `editor/ferrite/` | Rope-based editor (`ropey`): buffer, cursor, history, view, rendering, line_cache |
| `editor/widget.rs` | EditorWidget wrapper, integrates FerriteEditor via egui memory |
| `markdown/` | `editor.rs` (rendered view), `parser.rs` (comrak AST), `mermaid/` (11 diagram types), `csv_viewer.rs`, `tree_viewer.rs` |
| `terminal/` | Integrated terminal (PTY, VTE, screen, themes, split layouts) |
| `ui/` | Panels: ribbon, settings, file_tree, outline, search, terminal, productivity, frontmatter, welcome, command_palette |
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
- **Background file loading (≥5MB):** `open_file_smart()` on `FerriteApp` spawns background thread; `Tab.tab_content` (`TabContent::Loading`/`Ready`/`Error`) tracks state; `FileLoadMsg` channel polled in `update()`
- **Per-frame caching:** `Tab.content_version` (u64) gates cached `is_modified()`, `text_stats()`, `needs_cjk_cached()`, `needs_complex_script_cached()` — never scan full content per frame

## Where Things Live (common)

| Want to... | Look in... |
|------------|------------|
| Add a setting | `config/settings.rs` → `Settings` struct |
| Add keyboard shortcut | `app/keyboard.rs` → `handle_keyboard_shortcuts()` |
| Add command to palette | `config/settings.rs` → `ShortcutCommand`, `app/commands.rs` → icon, `app/central_panel.rs` → dispatch |
| Add/modify a UI panel | `ui/` → create or edit panel module |
| Modify editor core | `editor/ferrite/editor.rs` (behavior), `buffer.rs` (text), `view.rs` (viewport) |
| Modify markdown rendering | `markdown/editor.rs` or `markdown/widgets.rs` |
| Modify markdown parsing | `markdown/parser.rs` |
| Modify central panel | `app/central_panel.rs` |
| Add special tab | `state.rs` → `SpecialTabKind`, `app/central_panel.rs` |
| Add viewer tab | `state.rs` → `TabKind` variant + state struct, `app/central_panel.rs` → render method |
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
