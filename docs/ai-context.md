# Ferrite - AI Context !ONLY context related to the project here, do not put in task-related context

Rust (edition 2021) + egui 0.28 markdown editor. Immediate-mode GUI, no retained widget state.

## Architecture

| Module | Purpose |
|--------|---------|
| `app.rs` | Main update loop, keyboard shortcuts, event dispatch |
| `state.rs` | All application state (`AppState`, `Tab`, `TabState`) |
| `editor/widget.rs` | Editor widget wrapper, integrates FerriteEditor |
| `editor/ferrite/` | Custom rope-based editor for large files |
| `markdown/editor.rs` | WYSIWYG rendered editing |
| `markdown/mermaid/` | Native mermaid diagram rendering |
| `ui/` | UI panels (ribbon, settings, file_tree, etc.) |
| `terminal/` | Integrated terminal emulator (PTY, VTE, screen buffer, themes, layouts) |
| `ui/terminal_panel.rs` | Terminal panel UI (tabs, splits, floating windows, drag-and-drop) |
| `ui/productivity_panel.rs` | Productivity hub (task management, Pomodoro timer, quick notes) |
| `workers/` | Async worker infrastructure (feature-gated `async-workers`) |
| `config/settings.rs` | Persistent settings |
| `config/snippets.rs` | Text expansion snippets system |
| `config/session.rs` | Session persistence and crash recovery |
| `theme/` | Light/dark theme management |

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
| Add keyboard shortcut | `app.rs` → `handle_keyboard_shortcuts()` |
| Add setting | `config/settings.rs` → `Settings` struct |
| Add translation | `locales/en.yaml` + use `t!("key")` |
| Modify markdown rendering | `markdown/editor.rs` or `markdown/widgets.rs` |
| Add mermaid diagram type | `markdown/mermaid/` → new module |
| Modify editor behavior | `editor/ferrite/editor.rs` |

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

**v0.2.6.1 (Feb 2026 - in progress):** Bug fixes, code signing, terminal & productivity integration
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
