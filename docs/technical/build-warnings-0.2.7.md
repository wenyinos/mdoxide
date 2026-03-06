# Build Warnings Plan — v0.2.7 Pre-Release

**Date:** 2026-03-06
**Build status:** Compiles successfully (0 errors, 89 warnings)
**Command:** `cargo check`

All warnings are **unused imports** and **dead code** — no logic or correctness issues.

## Summary

| Category | Count | Severity |
|----------|-------|----------|
| Unused imports | ~41 (auto-fixable) | Trivial |
| Dead code (methods/fields/structs) | ~48 | Low |
| **Total** | **89** | **None blocking** |

## Fix Strategy

### Phase 1: Auto-fix unused imports (quick, safe)

Run `cargo fix --bin ferrite` to automatically remove the ~41 unused imports. These accumulated during the app.rs refactoring (modules were split but imports weren't fully cleaned up).

**Files affected:**
- `src/app/keyboard.rs` — `modifier_symbol`
- `src/app/navigation.rs` — `byte_to_char_offset`, `find_line_byte_range`, `CjkFontPreference`, `Settings`, `fonts`, `PendingAction`, `warn`
- `src/app/formatting.rs` — `ViewMode`, `PendingAction`
- `src/app/export.rs` — `portal_install_instructions`
- `src/app/find_replace.rs` — `Selection`, `warn`
- `src/app/dialogs.rs` — `ViewMode`, `FileType`, `FileOperationResult`, `GoToLineResult`, `warn`
- `src/app/status_bar.rs` — `modifier_symbol`, `Theme`, `get_structured_file_type`, `FileType`, `ThemeColors`
- `src/app/central_panel.rs` — `char_index_to_line_col`, `Theme`, `cleanup_ferrite_editor`, `DocumentOutline`, `FindReplacePanel`, `CsvViewerState`, `FormattingState`, `MarkdownFormatCommand`, `TreeViewerState`, `apply_raw_format`, `cleanup_rendered_editor_memory`, `FileType`, `PendingAction`, `Selection`, `HashMap`
- `src/app/mod.rs` — `CjkFontPreference`, `Settings`, `ShortcutCommand`, `Theme`, `EditorWidget`, `Minimap`, `OutlineType`, `SearchHighlights`, `SemanticMinimap`, `TextStats`, `extract_outline_for_file`, `copy_html_to_clipboard`, `generate_html_document`, `CsvViewer`, `DELIMITERS`, `EditorMode`, `MarkdownEditor`, `MarkdownFormatCommand`, `TocOptions`, `TreeViewer`, `apply_raw_format`, `delimiter_symbol`, `get_structured_file_type`, `get_tabular_file_type`, `insert_or_update_toc`, `PendingAction`, `FileOperationResult`, `GoToLineResult`, `SearchNavigationTarget`, `TitleBarButton`, `ViewModeSegment`, `ViewSegmentAction`, `trace`
- `src/editor/ferrite/mod.rs` — `SearchMatch`, `HighlightedSegment`, `VimState`
- `src/editor/mod.rs` — `Cursor`, `EditHistory`, `EditOperation`, `FerriteEditor`, `LineCache`, `Selection`, `TextBuffer`, `ViewState`
- `src/markdown/mermaid/flowchart/parser.rs` — `parse_direction`, `parse_edge_line_full`, `parse_node_from_text`

### Phase 2: Dead code cleanup (review needed)

These are methods, fields, and structs that exist but are never called. Many are intentional public APIs that may be used in future features. Review each before removing.

**Editor core (`src/editor/`):**
- `ferrite/editor.rs` — `undo_count()`, `redo_count()`
- `ferrite/line_cache.rs` — `from_layout_job()`, `get_galley_with_job()`, `measure_text()`, `invalidate_line()`, `is_empty()`, `capacity()`, `is_cached()`
- `ferrite/search.rs` — `search_matches()`, `search_match_count()`, `displayed_match_count()`, `has_more_matches()`, `current_search_match()`, `set_current_search_match()`
- `ferrite/view.rs` — `LARGE_FILE_THRESHOLD`, `pixel_to_line()`, `set_scroll_offset_y()`, `wrap_width()`, `uses_uniform_heights()`, `total_visual_rows()`, `logical_to_visual_row()`, `visual_row_to_logical()`
- `widget.rs` — `EditorOutput.scroll_offset_y`, `EditorOutput.total_lines`, `EditorWidget.frame`

**Files/dialogs (`src/files/`):**
- `dialogs.rs` — `DialogResult::ok()`, `DialogResult::is_portal_failure()`

**Terminal (`src/terminal/`):**
- `mod.rs` — `Terminal.id`, `Terminal::id()`, `Terminal::foreground_process()`, `Terminal::is_claude_code()`, `Terminal::size()`, `TerminalManager::macros()`, `TerminalManager::set_default_size()`, `TerminalManager::active_terminal()`, `TerminalManager::resize_all()`
- `screen.rs` — `CellAttributes::new()`, `CellAttributes::reset()`, `Cell::new()`, `TerminalScreen::scrollback()`, `TerminalScreen::move_cursor_relative()`, `TerminalScreen::reset_scroll_region()`, `TerminalScreen::get_visible_text()`

**UI (`src/ui/`):**
- `about.rs` — `AboutPanelOutput`, `AboutPanel::show()`
- `backlinks_panel.rs` — `BacklinksPanel::backlink_count()`
- `format_toolbar.rs` — `TOOLBAR_HEIGHT_COLLAPSED`
- `productivity_panel.rs` — `Task::to_markdown()`
- `settings.rs` — `SettingsPanelOutput.close_requested`, `SettingsPanel::show()`
- `terminal_panel.rs` — `TerminalPanelOutput.toggled`, `TerminalPanelState::show()`, `set_height()`, `height()`, `maybe_auto_save_layout()`

### Recommended Approach

1. **Before release:** Run `cargo fix --bin ferrite` for unused imports (Phase 1). Safe, no behavior change.
2. **After release:** Audit dead code in Phase 2. Some items (like `pixel_to_line`, `search_matches`) may be needed by future features or are part of the public API for an eventual crate extraction. Add `#[allow(dead_code)]` with justification comments for intentional API surface, remove genuinely dead code.

### Commands

```bash
# Phase 1: Auto-fix unused imports
cargo fix --bin ferrite --allow-dirty

# Verify
cargo check 2>&1 | grep "warning:" | wc -l

# Phase 2: Check remaining warnings
cargo clippy 2>&1
```
