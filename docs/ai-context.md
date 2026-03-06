# Ferrite - AI Context

Rust (edition 2021) + egui 0.28 markdown editor. Immediate-mode GUI — no retained widget state, UI rebuilds each frame.

## Architecture

| Module | Purpose |
|--------|---------|
| `app/` | Main application (~15 modules: keyboard, file_ops, formatting, navigation, etc.) |
| `state.rs` | All application state (`AppState`, `Tab`, `TabKind`, `SpecialTabKind`, `FileType`) |
| `editor/ferrite/` | Custom rope-based editor (`ropey`) for large files (buffer, cursor, history, view, rendering) |
| `editor/widget.rs` | Editor widget wrapper, integrates FerriteEditor via egui memory |
| `markdown/editor.rs` | WYSIWYG rendered editing |
| `markdown/parser.rs` | Comrak markdown parsing, AST operations |
| `markdown/mermaid/` | Native mermaid rendering (11 diagram types); flowchart is modular (`flowchart/{types,parser,layout/,render/,utils}`) |
| `markdown/csv_viewer.rs` | CSV/TSV table viewer with lazy byte-offset row parsing |
| `markdown/tree_viewer.rs` | JSON/YAML/TOML hierarchical tree viewer |
| `terminal/` | Integrated terminal emulator (PTY via `portable-pty`, VTE ANSI parser, screen buffer, themes, split layouts) |
| `ui/` | UI panels (ribbon, settings, file_tree, outline, search, terminal_panel, productivity_panel, frontmatter_panel, welcome) |
| `config/` | Settings persistence, session/crash recovery, text expansion snippets |
| `theme/` | Light/dark theme management (ThemeManager, light.rs, dark.rs) |
| `export/` | HTML export with themed CSS, clipboard operations |
| `preview/` | Sync scrolling between Raw and Rendered views |
| `vcs/git.rs` | Git integration (status tracking, branch display, auto-refresh via `git2`) |
| `workspaces/` | Folder mode (file tree, file watcher, workspace settings, persistence) |
| `workers/` | Async worker infrastructure (feature-gated `async-workers`, tokio runtime) |
| `platform/` | Platform-specific code (macOS Apple Events) |
| `single_instance.rs` | Lock file + TCP IPC so double-clicking files opens tabs in existing window |
| `fonts.rs` | Font loading, lazy CJK, complex script lazy loading (11 families), family selection |
| `update.rs` | Update checker (GitHub Releases API) |

## FerriteEditor

Custom high-performance editor at `src/editor/ferrite/`. Uses `ropey` rope for O(log n) text operations.

**Key files:** `editor.rs` (main widget), `buffer.rs` (rope), `view.rs` (viewport), `history.rs` (undo/redo), `line_cache.rs` (galley LRU cache)

**Capabilities:** Virtual scrolling (renders only visible lines), multi-cursor (Ctrl+Click), code folding, bracket matching, IME/CJK input, syntax highlighting, find/replace.

**Memory:** ~1x file size in RAM (rope-based vs ~6x with egui TextEdit).

**Integration:** `EditorWidget` in `widget.rs` creates/retrieves `FerriteEditor` from egui memory, syncs with `Tab.content`.

**Deep docs:** `docs/technical/editor/architecture.md`

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

## Where Things Live

| Want to... | Look in... |
|------------|------------|
| Add keyboard shortcut | `app/keyboard.rs` → `handle_keyboard_shortcuts()` |
| Add a file operation | `app/file_ops.rs` |
| Add text formatting command | `app/formatting.rs` |
| Add line operation (duplicate, move) | `app/line_ops.rs` |
| Add navigation feature | `app/navigation.rs` |
| Modify the title bar | `app/title_bar.rs` |
| Modify the status bar | `app/status_bar.rs` |
| Modify the central editor panel | `app/central_panel.rs` |
| Add a special tab (settings-like panel) | `state.rs` → `SpecialTabKind`, `app/central_panel.rs` → `render_special_tab_content()` |
| Add a setting | `config/settings.rs` → `Settings` struct |
| Add a translation string | `locales/en.yaml` + use `t!("key")` |
| Modify markdown rendering | `markdown/editor.rs` or `markdown/widgets.rs` |
| Modify markdown parsing | `markdown/parser.rs` |
| Add mermaid diagram type | `markdown/mermaid/` → new module |
| Modify flowchart layout | `markdown/mermaid/flowchart/layout/` |
| Modify flowchart rendering | `markdown/mermaid/flowchart/render/` |
| Add flowchart node shape | `flowchart/types.rs` (NodeShape) + `flowchart/render/nodes.rs` |
| Modify editor core behavior | `editor/ferrite/editor.rs` |
| Modify editor text buffer | `editor/ferrite/buffer.rs` |
| Change undo/redo behavior | `editor/ferrite/history.rs` |
| Modify code folding | `editor/folding.rs` |
| Modify minimap | `editor/minimap.rs` |
| Add/modify a UI panel | `ui/` → create or edit panel module |
| Modify terminal features | `terminal/` (pty, screen, widget, layout) |
| Modify terminal panel UI | `ui/terminal_panel.rs` |
| Modify productivity hub | `ui/productivity_panel.rs` |
| Change themes | `theme/light.rs` or `theme/dark.rs` |
| Add export format | `export/` → new module |
| Modify Git integration | `vcs/git.rs` |
| Modify workspace features | `workspaces/` |
| Add global app state | `state.rs` → `AppState` struct |
| Add per-tab state | `state.rs` → `Tab` struct |
| Modify platform-specific code | `platform/` (currently macOS only) |

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

- Finishing v0.2.7 release (performance, polish, new features)
- Task 44 done: Keep text selected after markdown formatting — fixed selection preservation and focus restoration
- Task 41 done: Word wrap scroll verification — fixed scroll sensitivity, documented findings
- Key areas: editor performance, wikilinks/backlinks, vim mode, callouts, single-instance, welcome page, Unicode font loading (Phase 1 done)
- PortableApps.com packaging complete — submit to Beta Testing forum after v0.2.7 release
- v0.2.8 planned: LSP integration, HarfRust text shaping for complex scripts (Arabic, Bengali, Devanagari)
- v0.3.0 planned: RTL/BiDi text support, mermaid crate extraction, math rendering

## Recently Changed

- **2026-03-06**: Task 45 — CJK Font Preloading Verification. Verified that explicit CJK font preferences (Japanese, Korean, Simplified/Traditional Chinese) are correctly preloaded at startup. Implementation was already complete: `preload_explicit_cjk_font()` in `fonts.rs` loads the appropriate font, `bump_font_generation()` increments the counter, and `editor.rs` invalidates the line cache when fonts change. No code changes required. Docs: `docs/technical/fonts/cjk-font-preloading-verification.md`.
- **2026-03-06**: Task 44 — Keep Text Selected After Markdown Formatting. Fixed format toolbar buttons to preserve both selection and focus. Used deferred format action pattern with pre-captured selection in `central_panel.rs` (raw and split view). Added focus restoration via `tab.needs_focus = true` in `mod.rs` after applying formatting. Users can now chain formatting operations (Bold→Italic) without reselecting, and editor maintains focus after clicking toolbar buttons.
- **2026-03-06**: Task 41 — Word Wrap Scroll Verification & Fix. Verified Task 40 optimizations. Fixed scroll sensitivity: `smooth_scroll_delta` is already in points (pixels), so removed incorrect `line_height` multiplication that caused 20× overscroll. All 47 scroll tests pass. Docs: `docs/technical/editor/word-wrap-performance.md`.
- **2026-03-06**: Task 40 — Word Wrap Scroll Performance. Incremental `rebuild_height_cache()` (O(changed) vs O(N)), counter-based O(1) LRU in LineCache replacing O(N) VecDeque scan, `cumulative_visual_rows` array for O(1)/O(log N) visual row mapping, fixed `get_galley_with_job` cache key to include font/color/theme. Docs: `docs/technical/editor/word-wrap-performance.md`.
- **2026-03-06**: Task 43 — Linux File Dialog Error Handling for Hyprland and Portal Failures. Added `DialogResult<T>` type in `src/files/dialogs.rs` to detect xdg-desktop-portal failures on Linux (Hyprland, Sway, i3, etc.). Portal failure detection via `XXDG_CURRENT_DESKTOP` env var. Error modal with distro-specific install instructions (pacman, apt, dnf). "Copy Install Command" button. Logging with `log::warn!`. Docs: `docs/technical/platform/linux-portal-dialogs.md`.
- **2026-03-06**: Task 42 — Ctrl+Scroll Wheel Zoom. Mapped Ctrl+scroll to `egui::gui_zoom::zoom_in/zoom_out` (same as Ctrl++/Ctrl+-). Added `ZoomIn/ZoomOut/ResetZoom` as `ShortcutCommand` variants with keyboard bindings. Docs: `docs/technical/editor/ctrl-scroll-zoom.md`.
- **2026-03-06**: Task 34 — Word Wrap Scroll Correctness Fixes. Fixed 9 functions in `view.rs` and `widget.rs` that assumed uniform line heights when word wrap is enabled. `pixel_to_line()`, `line_to_pixel()`, `scroll_to_center_line()`, `is_line_visible()`, `ensure_line_visible()` now use `cumulative_heights` / `y_offset_to_line()` binary search for wrap-aware coordinate conversion. `tab.scroll_offset`, sync scroll, and viewport restoration in `widget.rs` now use `current_scroll_y()` / `scroll_to_absolute()`. Performance optimization deferred to Tasks 40/41. Docs: `docs/technical/editor/word-wrap-scroll-fixes.md`.
- **2026-03-04**: Task 33 — Complex Script Font Preferences. Settings UI for per-script font selection (11 scripts). Docs: `docs/technical/config/complex-script-font-preferences.md`.
- **2026-03-04**: Fixed Open Folder in Flatpak (Task 39). Portal dialog `$HOME` fallback. Docs: `docs/technical/platform/flatpak-file-dialog-portal.md`.
- **2026-03-04**: Visual frontmatter editor (Task 32). FM tab in outline panel, form-based YAML editing. Docs: `docs/technical/ui/frontmatter-panel.md`.
- **2026-03-02**: Fixed binary file open crash (`stats.rs` byte index panic). Added `is_binary_content()` detection.
- **2026-02-26**: Nix/NixOS flake support (PR #92). PortableApps.com packaging with CI automation.
- **2026-02-23**: Unicode Complex Script Font Loading (Phase 1). 11 script families, lazy loading in `fonts.rs`.
