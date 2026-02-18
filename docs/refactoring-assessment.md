# Ferrite – Refactoring Assessment

**Date:** Feb 2026  
**Scope:** Rust source under `src/` only. No code changed; assessment only.

---

## Summary

| Metric | Value |
|--------|--------|
| **Total `src/**/*.rs` lines** | **87,254** |
| **Rust source files** | 145 (under `src/`) |
| **Largest file** | `markdown/editor.rs` (5,159 lines) |
| **Files ≥ 1,000 lines** | 32 |
| **Files ≥ 2,000 lines** | 10 |

---

## File Size Tiers

### Very large (≥ 3,000 lines) – high refactor priority

| Lines | File | Notes |
|-------|------|--------|
| 5,159 | `markdown/editor.rs` | WYSIWYG markdown editor; single biggest file. |
| 4,879 | `state.rs` | AppState, Tab, TabKind, SpecialTabKind, event handling – central state. |
| 3,838 | `markdown/widgets.rs` | Editable heading/list/table/code/link widgets. |
| 3,356 | `editor/ferrite/editor.rs` | FerriteEditor core (buffer, cursor, view, rendering orchestration). |
| 3,235 | `config/settings.rs` | Settings struct, validation, shortcuts, persistence. |

### Large (2,000–2,999 lines)

| Lines | File | Notes |
|-------|------|--------|
| 2,187 | `app/mod.rs` | Already split from monolithic app.rs; still holds a lot of orchestration. |
| 1,927 | `ui/terminal_panel.rs` | Terminal panel UI (tabs, splits, floating windows, context menus). |
| 1,806 | `ui/settings.rs` | Settings panel UI (appearance, editor, files, keyboard, terminal). |

### Medium–large (1,000–1,999 lines)

| Lines | File |
|-------|------|
| 1,771 | `fonts.rs` |
| 1,723 | `markdown/parser.rs` |
| 1,716 | `app/central_panel.rs` |
| 1,624 | `editor/ferrite/view.rs` |
| 1,464 | `markdown/csv_viewer.rs` |
| 1,336 | `editor/minimap.rs` |
| 1,332 | `markdown/mermaid/mod.rs` |
| 1,246 | `app/file_ops.rs` |
| 1,227 | `editor/outline.rs` |
| 1,118 | `config/session.rs` |
| 1,116 | `markdown/mermaid/sequence.rs` |
| 1,079 | `terminal/mod.rs` |
| 1,062 | `markdown/tree_viewer.rs` |
| 1,007 | `markdown/formatting.rs` |
| 999 | `ui/outline_panel.rs` |
| 984 | `markdown/mermaid/state.rs` |
| 955 | `ui/pipeline.rs` |
| 950 | `ui/productivity_panel.rs` |
| 941 | `preview/sync_scroll.rs` |
| 938 | `editor/find_replace.rs` |
| 867 | `markdown/ast_ops.rs` |
| 866 | `editor/widget.rs` |
| 832 | `terminal/widget.rs` |
| 816 | `editor/ferrite/line_cache.rs` |
| 793 | `markdown/syntax.rs` |
| 785 | `markdown/mermaid/flowchart/parser.rs` |
| 784 | `ui/ribbon.rs` |
| 781 | `editor/ferrite/buffer.rs` |
| 774 | `app/navigation.rs` |
| 771 | `editor/ferrite/history.rs` |
| 769 | `editor/ferrite/input/keyboard.rs` |
| 753 | `terminal/screen.rs` |
| 734 | `ui/window.rs` |
| 704 | `editor/stats.rs` |
| 684 | `vcs/git.rs` |
| 678 | `ui/search.rs` |
| 662 | `editor/matching.rs` |

### Medium (500–999 lines)

24 files in this range (e.g. `theme/mod.rs`, `ui/dialogs.rs`, `export/html.rs`, flowchart layout/render modules, etc.).

### Small (&lt; 500 lines)

Remaining ~75 files – mod.rs re-exports, small modules, focused utilities.

---

## Refactoring Priorities (assessment only)

### 1. `markdown/editor.rs` (5,159 lines)

- **Why:** Largest file; WYSIWYG editing, sync with source, and many node types in one place.
- **Idea:** Split by responsibility, e.g.:
  - Core editor loop and source ↔ WYSIWYG sync
  - Per-node-type rendering/edit (headings, paragraphs, lists, blocks, etc.)
  - Toolbar/formatting integration
- **Ref:** Similar to flowchart refactor (monolith → focused modules).

### 2. `state.rs` (4,879 lines)

- **Why:** All app/tab/special-tab state and a large share of event/action handling.
- **Idea:** Consider:
  - `state/mod.rs` – re-exports and core types
  - `state/app_state.rs` – AppState, global UI state
  - `state/tab.rs` – Tab, TabState, TabKind, SpecialTabKind
  - `state/events.rs` or integration with app – event enums and dispatch
- **Care:** Touched by many modules; refactor in small, compile-after-each steps.

### 3. `markdown/widgets.rs` (3,838 lines)

- **Why:** Many editable widget types in one file.
- **Idea:** Split by widget kind, e.g. `widgets/headings.rs`, `widgets/lists.rs`, `widgets/tables.rs`, `widgets/code.rs`, `widgets/links.rs`, with a thin `widgets/mod.rs`.

### 4. `editor/ferrite/editor.rs` (3,356 lines)

- **Why:** Core editor logic, input, and rendering orchestration in one file.
- **Idea:** Further extract: e.g. input handling (beyond existing `input/`), high-level render coordination, or “mode” handling (edit vs view) into sibling modules. Keep `editor.rs` as the main entry that delegates.

### 5. `config/settings.rs` (3,235 lines)

- **Why:** Settings definition, validation, shortcuts, and a lot of UI-related options.
- **Idea:** Split by domain: e.g. `settings/appearance.rs`, `settings/editor.rs`, `settings/files.rs`, `settings/shortcuts.rs`, `settings/validation.rs`, with a central `Settings` and `mod.rs` that composes them.

### 6. `app/mod.rs` (2,187 lines)

- **Why:** Already reduced from the old 7,634-line app.rs; still holds a lot of coordination.
- **Ref:** `docs/technical/planning/app-rs-refactoring-plan.md` – Phase 3 (decompose `render_ui`) and Phase 4 (ribbon dispatch) are the next levers to shrink this further.

### 7. `ui/terminal_panel.rs` (1,927) and `ui/settings.rs` (1,806)

- **Why:** Large UI modules with many sections and branches.
- **Idea:** Extract sub-views or sections (e.g. terminal: tabs vs splits vs context menus; settings: per-tab or per-section modules) into separate files under `ui/terminal_panel/` or `ui/settings/` if it improves readability and review size.

---

## What’s Already in Good Shape

- **`app/`** – Already split from a single 7,634-line file into ~15 modules; `app/mod.rs` is the main remaining large chunk.
- **`markdown/mermaid/flowchart/`** – Already refactored from a single ~3,600-line file into 12 modules (types, parser, layout/, render/, utils); good template for other big modules.
- **`editor/ferrite/`** – Already modular (buffer, cursor, view, history, line_cache, input/, rendering/); `editor.rs` is the main remaining large file.
- **Small, focused modules** – Many files under ~500 lines (e.g. theme, platform, workers, export, preview, path_utils, error) are in a good place for maintenance.

---

## Suggested Order of Work (if you refactor later)

1. **Markdown editor** – Split `markdown/editor.rs` by responsibility (and optionally align with `widgets` split).
2. **State** – Split `state.rs` into a small `state/` crate or module group with clear boundaries.
3. **Markdown widgets** – Split `markdown/widgets.rs` by widget type.
4. **Ferrite editor** – Further trim `editor/ferrite/editor.rs` by extracting cohesive blocks.
5. **Config** – Split `config/settings.rs` by domain.
6. **App** – Continue app plan Phase 3/4 to reduce `app/mod.rs`.
7. **UI panels** – Optionally split `ui/terminal_panel.rs` and `ui/settings.rs` if needed for readability or parallel work.

---

## Notes

- **Line counts** are raw lines (including comments and blanks); they reflect size and navigation cost, not necessarily complexity.
- **Flowchart refactor** is the best in-repo example: same behavior, tests passing, much better structure.
- **No code was changed** for this assessment; this document is for planning only.
