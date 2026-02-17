# Ferrite - AI Context !ONLY context related to the project here, do not put in task-related context

Rust (edition 2021) + egui 0.28 markdown editor. Immediate-mode GUI, no retained widget state.

## Architecture

| Module | Purpose |
|--------|---------|
| `app/` | Main application (15 modules: keyboard, file_ops, formatting, navigation, etc.) |
| `state.rs` | All application state (`AppState`, `Tab`, `TabKind`, `SpecialTabKind`, `FileType`) |
| `editor/widget.rs` | Editor widget wrapper, integrates FerriteEditor |
| `editor/ferrite/` | Custom rope-based editor for large files (buffer, cursor, history, view, rendering) |
| `markdown/editor.rs` | WYSIWYG rendered editing |
| `markdown/parser.rs` | Comrak markdown parsing, AST operations |
| `markdown/mermaid/` | Native mermaid diagram rendering (11 diagram types); flowchart is modular (`flowchart/types`, `parser`, `layout/`, `render/`, `utils`) |
| `markdown/csv_viewer.rs` | CSV/TSV table viewer with rainbow columns |
| `markdown/tree_viewer.rs` | JSON/YAML/TOML hierarchical tree viewer |
| `ui/welcome.rs` | Welcome/first-launch configuration panel (theme, language, editor prefs) |
| `ui/` | UI panels (ribbon, settings, file_tree, outline, search, etc.) |
| `ui/terminal_panel.rs` | Terminal panel UI (tabs, splits, floating windows, drag-and-drop) |
| `ui/productivity_panel.rs` | Productivity hub (task management, Pomodoro timer, quick notes) |
| `terminal/` | Integrated terminal emulator (PTY, VTE, screen buffer, themes, layouts) |
| `workers/` | Async worker infrastructure (feature-gated `async-workers`) |
| `config/settings.rs` | Persistent settings |
| `config/snippets.rs` | Text expansion snippets system |
| `config/session.rs` | Session persistence and crash recovery |
| `theme/` | Light/dark theme management (ThemeManager, light.rs, dark.rs) |
| `export/` | Document export (HTML with CSS, clipboard operations) |
| `preview/` | Preview sync scrolling between Raw and Rendered views |
| `vcs/git.rs` | Git integration (status tracking, branch display, auto-refresh) |
| `workspaces/` | Folder mode (file tree, watcher, workspace settings, persistence) |
| `files/dialogs.rs` | Native file dialogs (rfd) |
| `platform/` | Platform-specific code (macOS Apple Events) |
| `fonts.rs` | Font loading, lazy CJK, family selection |
| `update.rs` | Update checker (GitHub Releases API) |
| `error.rs` | Error types and centralized handling |

## FerriteEditor (v0.2.6 - Complete)

Custom high-performance editor at `src/editor/ferrite/`. Uses rope (`ropey`) for O(log n) text operations.

**Key files:** `editor.rs` (main), `buffer.rs` (rope), `view.rs` (viewport), `history.rs` (undo)

**v0.2.6 features:** Virtual scrolling, multi-cursor (Ctrl+Click), code folding, undo/redo (Ctrl+Z/Y), bracket matching, IME/CJK input, syntax highlighting.

**Memory:** 80MB file uses ~80MB RAM (was 460MB+ with egui TextEdit).

**Integration:** `EditorWidget` in `widget.rs` creates/retrieves `FerriteEditor` from egui memory, syncs with `Tab.content`.

**Read first:** `docs/technical/editor/architecture.md`

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
- **State:** `TabState` for per-tab, `AppState` for global
- **Errors:** User-facing via `show_toast()`, technical via `log::error!`

## Where Things Live

| Want to... | Look in... |
|------------|------------|
| Add keyboard shortcut | `app/keyboard.rs` → `handle_keyboard_shortcuts()` |
| Add a file operation (open/save) | `app/file_ops.rs` |
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
| Modify markdown parsing | `markdown/parser.rs` (comrak integration) |
| Add mermaid diagram type | `markdown/mermaid/` → new module |
| Modify flowchart parsing | `markdown/mermaid/flowchart/parser.rs` |
| Modify flowchart layout | `markdown/mermaid/flowchart/layout/` (sugiyama.rs, subgraph.rs) |
| Modify flowchart rendering | `markdown/mermaid/flowchart/render/` (nodes.rs, edges.rs) |
| Add flowchart node shape | `flowchart/types.rs` (NodeShape) + `flowchart/render/nodes.rs` |
| Modify editor core behavior | `editor/ferrite/editor.rs` |
| Modify editor text buffer | `editor/ferrite/buffer.rs` (rope-based) |
| Change undo/redo behavior | `editor/ferrite/history.rs` |
| Modify code folding | `editor/folding.rs` |
| Modify minimap | `editor/minimap.rs` |
| Add/modify a UI panel | `ui/` → create or edit panel module |
| Modify the ribbon toolbar | `ui/ribbon.rs` |
| Modify settings panel | `ui/settings.rs` |
| Modify terminal features | `terminal/` (PTY, screen, widget, layout) |
| Modify terminal panel UI | `ui/terminal_panel.rs` |
| Modify productivity hub | `ui/productivity_panel.rs` |
| Modify file tree | `ui/file_tree.rs` |
| Modify quick switcher | `ui/quick_switcher.rs` |
| Modify search in files | `ui/search.rs` |
| Change themes (light/dark) | `theme/light.rs` or `theme/dark.rs` |
| Add export format | `export/` → new module |
| Modify Git integration | `vcs/git.rs` |
| Modify workspace features | `workspaces/` (file_tree, watcher, settings) |
| Add global app state | `state.rs` → `AppState` struct |
| Add per-tab state | `state.rs` → `Tab` struct |
| Add font support | `fonts.rs` |
| Modify platform-specific code | `platform/` (currently macOS only) |

## Performance Rules

For FerriteEditor (large file support):

| Tier | When Allowed | Examples |
|------|--------------|----------|
| O(1) | Always | `line_count()`, `is_dirty()` |
| O(log N) | Always | `get_line(idx)`, index conversions |
| O(visible) | Per-frame | Syntax highlighting visible lines |
| O(N) | User-initiated ONLY | Find All, Save, Export |

**Never** call `buffer.to_string()` in per-frame code.

## Large File Handling

Files > 1MB get special treatment:
- Hash-based `is_modified()` instead of full comparison
- Reduced undo stack (10 vs 100 entries)
- No `original_bytes` storage

## Build & Test

```bash
cargo build          # Build debug
cargo run            # Run app
cargo clippy         # Lint
cargo test           # Run tests
```

## Terminal Emulator (PR #74 - Integrated)

Full integrated terminal at `src/terminal/`. Uses `portable-pty` for cross-platform PTY and `vte` for ANSI parsing.

**Key files:** `mod.rs` (Terminal, TerminalManager), `screen.rs` (buffer), `pty.rs` (shell), `widget.rs` (rendering), `layout.rs` (splits), `theme.rs` (color schemes), `handler.rs` (VTE handler), `sound.rs` (notifications)

**Features:** Multiple tabs, split panes (H/V), floating windows, drag-and-drop tab reorder, 16/256/truecolor ANSI, themes (Dracula, Nord, etc.), prompt detection, layout save/load, shell selection (PowerShell/CMD/WSL/bash).

**UI:** `ui/terminal_panel.rs` manages the bottom panel with tabs, split rendering, context menus, maximize pane (Ctrl+Shift+M).

## Productivity Hub (PR #74 - Integrated)

`ui/productivity_panel.rs` - Workspace-scoped productivity tools (Ctrl+Shift+H):
- **Tasks:** Markdown checkbox syntax (`- [ ]`), priority (`!`/`!!`), persistent in `.ferrite/tasks.json`
- **Pomodoro:** 25/5 work/break timer with sound notifications
- **Quick Notes:** Auto-save per workspace in `.ferrite/notes/`

## Async Workers (Feature-gated)

`src/workers/` - Background tokio runtime for non-blocking operations. Feature-gated behind `async-workers`. Currently has echo worker template for future AI/DB panels.

## Recently Changed

**v0.2.7 (Feb 2026 - in progress):** Performance, features & polish
- **Flowchart Modular Refactor (Task 17):** Split monolithic `flowchart.rs` (3600 lines) into 12 focused modules under `flowchart/` directory: `types.rs`, `parser.rs`, `layout/` (config, graph, subgraph, sugiyama), `render/` (colors, nodes, edges, subgraphs), `utils.rs`. Zero behavior changes, all 83 tests pass. See `docs/technical/mermaid/flowchart-modular-refactor.md`.
- **Welcome Page:** First-launch welcome tab for initial configuration (theme, language, editor settings, CJK font, line width, auto-save). Opens via `SpecialTabKind::Welcome` only when no CLI paths and no restored session tabs. Dropdowns use Latin-only `selector_display_name()` to avoid loading CJK fonts at startup.
- **Special Tabs:** Settings and About/Help now open as tabs (like Cursor/VS Code) instead of modal windows. `TabKind`/`SpecialTabKind` system is extensible for future panels. `show_inline()` methods on `SettingsPanel` and `AboutPanel` render content directly in the tab area.
- **CRASH FIX:** Large selection delete with word wrap caused `capacity overflow` panic. Stale `wrap_info`/`first_visible_line` after deletion → `Vec::with_capacity(usize underflow)`. Fixed: hard-clamp `first_visible_line` in `clamp_scroll_position`, `saturating_sub` in allocation, clamp `cursor_to_char_pos` to `buffer.len()`, new `truncate_wrap_info()` (trims stale entries without flickering).

**v0.2.6.1 (Feb 2026):** Bug fixes, code signing, terminal & productivity integration
- Fixed keyboard shortcut conflicts (FormatInlineCode/ToggleTerminal Ctrl+Backtick collision)
- Undo after formatting now creates discrete undo entries (break_group before/after format ops)
- Consecutive blockquotes merged in parser; blockquote border height fixed (paint-after-measure)
- Lazy CJK font loading reduces startup memory by ~80MB
- Integrated terminal emulator with splits, themes, floating windows
- Productivity hub with tasks, Pomodoro timer, quick notes
- Windows code signing via SignPath.io (production certificate)

**v0.2.6 (Jan 2026):** Complete FerriteEditor custom text editor
- Replaced egui TextEdit with rope-based editor for large file performance
- Virtual scrolling renders only visible lines (O(visible) per frame)
- Multi-cursor editing (Ctrl+Click), code folding, bracket matching
- Full undo/redo with operation-based history
- Memory: ~1x file size (was ~6x with TextEdit)
- IME/CJK input support
- See `docs/v0.2.6-manual-test-suite.md` for test coverage
