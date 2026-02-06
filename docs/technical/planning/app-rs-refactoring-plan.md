# app.rs Refactoring Plan

**Current state:** 7,634 lines in a single file  
**Target:** Split into ~8 focused modules, each under 1,500 lines  
**Priority:** v0.2.7 (after v0.2.6.1 patch release)

## Problem

`app.rs` is the largest file in the codebase and handles too many responsibilities:
- Window management and title bar rendering
- Keyboard shortcut dispatch
- File operations (open, save, save as)
- Workspace management
- Line operations (duplicate, move, delete)
- Find/replace coordination
- Format commands
- Drag-and-drop and image handling
- Auto-close brackets and smart paste
- Undo/redo coordination
- Export (HTML)
- Dialogs (recovery, close confirmation)
- Ribbon action dispatch
- Git refresh coordination
- Terminal and productivity panel toggling
- The massive `render_ui()` method (2,887 lines)

## Current Structure Analysis

| Section | Lines | Description |
|---------|-------|-------------|
| Types & imports | 1-186 | KeyboardAction enum, helper structs |
| FerriteApp struct | 188-281 | 93 lines of fields |
| `new()` | 289-481 | Initialization |
| `open_initial_paths()` | 482-584 | CLI file opening |
| Window/session helpers | 585-1277 | Window state, recovery, auto-save |
| **`render_ui()`** | **1278-4165** | **2,887 lines - THE MONSTER** |
| File operations | 4166-4518 | open, save, save_as, workspace |
| Search/git/watcher | 4519-4697 | File watcher, git, search navigation |
| Image/drag-drop | 4698-4994 | Image handling, dropped files |
| File tree context | 4995-5200 | Create/rename/delete files |
| Input preprocessing | 5201-5760 | Undo/redo, smart paste, auto-close |
| Keyboard shortcuts | 5761-6022 | Shortcut detection and dispatch |
| Tab/view navigation | 6022-6182 | Tab switching, view mode cycling |
| Scroll interpolation | 6183-6264 | Rendered ↔ source line mapping |
| Theme handling | 6265-6291 | Theme switching |
| Undo/redo | 6292-6362 | Undo/redo execution |
| Format commands | 6363-6572 | Markdown formatting, TOC |
| Panel toggles | 6573-6742 | Terminal, zen, pipeline, etc. |
| Line operations | 6743-7073 | Go-to-line, duplicate, move, delete |
| Export | 7074-7202 | HTML export, copy-as-HTML |
| Structured docs | 7203-7314 | JSON/YAML format/validate |
| Outline/heading nav | 7315-7475 | Outline updates, heading navigation |
| Text utilities | 7476-7565 | byte↔char, line↔col conversions |
| Find/replace | 7567-7776 | Find, replace, select occurrence |
| Ribbon dispatch | 7777-7939 | Ribbon action → handler |
| Dialogs | 7940-8172 | Go-to-line, close confirm, settings |
| `eframe::App` impl | 8174-8455 | update(), on_exit(), save() |
| Standalone helpers | 8457-8503 | char_index_to_line_col, etc. |

## Proposed Module Structure

Create a new `src/app/` directory with the following modules:

```
src/app/
├── mod.rs              # FerriteApp struct, eframe::App impl, update() orchestration
├── types.rs            # KeyboardAction, HeadingNavRequest, DeferredFormatAction, etc.
├── init.rs             # new(), open_initial_paths(), ensure_echo_worker()
├── render.rs           # render_ui() - further decomposed with helper methods
├── title_bar.rs        # Title bar rendering (extracted from render_ui)
├── editor_panels.rs    # Editor/tab content area rendering (extracted from render_ui)
├── keyboard.rs         # handle_keyboard_shortcuts(), all key consumption methods
├── file_ops.rs         # open, save, save_as, workspace open/close, drag-drop
├── line_ops.rs         # duplicate, move, delete line, go-to-line
├── formatting.rs       # format commands, TOC, structured doc format/validate
├── find_replace.rs     # find/replace coordination, select next occurrence
├── input_handling.rs   # auto-close brackets, smart paste, undo/redo consume
├── navigation.rs       # tab switching, view mode, outline nav, heading nav, scroll interp
├── export.rs           # HTML export, copy-as-HTML
├── dialogs.rs          # recovery dialog, close confirm, auto-save recovery, go-to-line
├── session.rs          # session save/recovery, window state tracking
└── helpers.rs          # text utilities (byte↔char, line↔col conversions)
```

## Migration Strategy

### Phase 1: Extract types and helpers (low risk)
1. Move `KeyboardAction`, `HeadingNavRequest`, `DeferredFormatAction`, `AutoSaveRecoveryInfo` to `types.rs`
2. Move standalone functions (`char_index_to_line_col`, `line_col_to_char_index`, `modifier_symbol`) to `helpers.rs`
3. Move text utility methods (`byte_to_char_offset`, `offset_to_line_col`, `find_line_byte_range`) to `helpers.rs`

**Estimated result:** app.rs shrinks by ~200 lines. Low risk, purely structural.

### Phase 2: Extract self-contained handler groups (medium risk)
4. Move file operations to `file_ops.rs`: `handle_open_file`, `handle_save_file`, `handle_save_as_file`, `handle_open_workspace`, `handle_close_workspace`, `handle_dropped_files`, `handle_dropped_image`, `handle_file_tree_context_action`, `handle_create_file`, `handle_create_folder`, `handle_rename_file`, `handle_delete_file`, `is_supported_image`, `get_assets_dir`, `generate_unique_image_filename`
5. Move line operations to `line_ops.rs`: `handle_go_to_line`, `handle_duplicate_line`, `handle_move_line`, `handle_delete_line`
6. Move find/replace to `find_replace.rs`: `handle_open_find`, `handle_find_next`, `handle_find_prev`, `handle_select_next_occurrence`, `handle_replace_current`, `handle_replace_all`
7. Move formatting to `formatting.rs`: `handle_format_command`, `handle_format_command_with_selection`, `handle_insert_toc`, `handle_format_structured_document`, `handle_validate_structured_syntax`, `get_formatting_state`
8. Move export to `export.rs`: `handle_export_html`, `handle_copy_as_html`
9. Move input handling to `input_handling.rs`: `consume_undo_redo_keys`, `filter_cut_event_if_no_selection`, `consume_move_line_keys`, `consume_smart_paste`, `handle_auto_close_pre_render`, `handle_auto_close_post_render`, `is_url`, `is_image_url`, `get_closing_bracket`, `is_closing_bracket`
10. Move keyboard shortcuts to `keyboard.rs`: `handle_keyboard_shortcuts`
11. Move navigation to `navigation.rs`: `handle_close_current_tab`, `handle_next_tab`, `handle_prev_tab`, `handle_toggle_view_mode`, `handle_toggle_outline`, `handle_toggle_terminal`, `handle_toggle_zen_mode`, `handle_toggle_fullscreen`, `handle_toggle_pipeline`, `handle_set_theme`, `handle_cycle_theme`, `handle_undo`, `handle_redo`, `navigate_to_heading`, `find_heading_near_line`, `update_outline_if_needed`, scroll interpolation methods
12. Move dialogs to `dialogs.rs`: `show_recovery_dialog_if_needed`, `show_auto_save_recovery_dialog`, `render_dialogs`
13. Move session to `session.rs`: `update_window_state`, `window_title`, `handle_close_request`, `update_session_recovery`, `mark_session_dirty`, `inject_csv_delimiters`, `restore_csv_delimiters`, `cleanup_tab_state`, `process_auto_saves`, `cleanup_auto_save_for_tab`, `check_auto_save_recovery`, `force_session_save`

**Estimated result:** app.rs shrinks to ~3,200 lines (render_ui + init + update + struct).

### Phase 3: Decompose render_ui (high value, medium risk)
14. Extract title bar rendering (~400 lines) to `title_bar.rs`
15. Extract ribbon/toolbar rendering to the existing ribbon module or a new helper
16. Extract the tab bar rendering section
17. Extract the main editor content area (raw/rendered/split/csv/tree views) to `editor_panels.rs`
18. Extract bottom panels (status bar, terminal, find/replace) rendering

**Estimated result:** render_ui shrinks from 2,887 lines to ~800 lines of orchestration.

### Phase 4: Extract ribbon dispatch
19. Move `handle_ribbon_action` to `mod.rs` or a dedicated file. This is a large match statement that dispatches to the handler methods.

## Implementation Pattern

All extracted modules will use `impl FerriteApp` blocks:

```rust
// src/app/file_ops.rs
use super::FerriteApp;

impl FerriteApp {
    pub(crate) fn handle_open_file(&mut self) {
        // ... moved from app.rs
    }
    
    pub(crate) fn handle_save_file(&mut self) {
        // ... moved from app.rs
    }
}
```

The `mod.rs` file re-exports everything and contains the `FerriteApp` struct definition and `eframe::App` impl.

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Borrow checker issues with split methods | Use `pub(crate)` visibility; methods still operate on `&mut self` |
| Circular dependencies between modules | All modules are `impl` blocks on the same struct; no circular deps possible |
| render_ui closure captures | Title bar and panel rendering use closures that borrow `self`; may need to extract state into local variables before passing to sub-functions |
| Merge conflicts during refactoring | Do in a dedicated branch; avoid parallel work on app.rs |
| Regression risk | Run `cargo clippy` and manual test after each phase; keep commits atomic per phase |

## Success Criteria

- [ ] No file in `src/app/` exceeds 1,500 lines
- [ ] `render_ui` orchestration is under 800 lines
- [ ] `cargo clippy` passes with no new warnings
- [ ] All keyboard shortcuts still work
- [ ] All view modes still work
- [ ] Session recovery still works
- [ ] Terminal panel still works
- [ ] Productivity panel still works

## Priority Order

1. **Phase 1** first (types/helpers) - quick win, zero risk
2. **Phase 2** next (handlers) - biggest line reduction
3. **Phase 3** last (render_ui) - most complex but highest value
4. **Phase 4** can be done anytime

Each phase can be a separate PR/commit for easy review.
