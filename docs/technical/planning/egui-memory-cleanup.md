# egui Temporary Data Cleanup

## Overview

Cleans up stale temporary data stored in egui's `memory().data` when tabs are closed. This prevents memory accumulation from rendered markdown editor widget states.

## Key Files

- `src/markdown/editor.rs` - Contains `cleanup_rendered_editor_memory()` function
- `src/markdown/mod.rs` - Exports the cleanup function
- `src/app.rs` - Calls cleanup in `cleanup_tab_state()` method

## Problem

The rendered markdown editor (`MarkdownEditor` in rendered/WYSIWYG mode) stores temporary state in egui's `memory().data` for interactive widgets:

- **FormattedItemEditState** - Click-to-edit state for formatted paragraphs and list items
- **CodeBlockData** - Code block content and edit mode state
- **MermaidBlockData** - Mermaid diagram source and render state
- **TableData** - Table cell contents and structure
- **TableEditState** - Table cell focus and navigation state
- **RenderedLinkState** - Link edit popup state

These entries are keyed by egui `Id` values that include `ui.id()` (parent widget hierarchy) combined with `node.start_line` (document position). When:

1. A tab with many elements is opened
2. Temp state is created for headings, paragraphs, code blocks, etc.
3. Tab is closed
4. **Entries remain in egui memory indefinitely**

Over many open/close cycles, this causes unbounded memory growth.

## Solution

### cleanup_rendered_editor_memory()

Added a public cleanup function in `src/markdown/editor.rs`:

```rust
pub fn cleanup_rendered_editor_memory(ctx: &egui::Context) {
    ctx.memory_mut(|mem| {
        mem.data.remove_by_type::<FormattedItemEditState>();
        mem.data.remove_by_type::<CodeBlockData>();
        mem.data.remove_by_type::<MermaidBlockData>();
        mem.data.remove_by_type::<TableData>();
        mem.data.remove_by_type::<TableEditState>();
        mem.data.remove_by_type::<RenderedLinkState>();
    });
}
```

Uses egui's `remove_by_type::<T>()` API to remove ALL entries of each type. This is appropriate because:

1. These are temporary edit buffers - actual content lives in the document source
2. Data is lazily recreated when widgets are rendered
3. Only the active tab's temp data matters at any time

### Integration with Tab Close

The `cleanup_tab_state()` method in `FerriteApp` was updated:

```rust
fn cleanup_tab_state(&mut self, tab_id: usize, ctx: Option<&egui::Context>) {
    // Existing HashMap cleanup
    self.tree_viewer_states.remove(&tab_id);
    self.csv_viewer_states.remove(&tab_id);
    self.sync_scroll_states.remove(&tab_id);

    // New: Clean up egui temporary data
    if let Some(ctx) = ctx {
        cleanup_rendered_editor_memory(ctx);
    }
}
```

The `ctx` parameter is optional because one call site (CLI file open during startup) doesn't have context access. For that case, `None` is passed since the empty default tab has no temp data.

### Tab Close Paths Updated

| Location | Context Available |
|----------|-------------------|
| CLI file open (startup) | No - passes `None` |
| Tab bar X button | Yes - `ui.ctx()` |
| File deletion handler | Yes - passed from `update()` |
| Ctrl+W keyboard shortcut | Yes - passed from `handle_keyboard_shortcuts()` |
| Confirmation dialog (Save) | Yes - `ui.ctx()` |
| Confirmation dialog (Discard) | Yes - `ui.ctx()` |

## Types Not Cleaned

### SyntaxHighlightCache

The `SyntaxHighlightCache` in `src/editor/widget.rs` is stored at `egui::Id::NULL` (global) and contains at most 4 entries (one per editor widget type). It's NOT a memory leak because entries are overwritten when content changes.

### Generic Types (String, bool)

Some temp data uses generic types like `String` (heading edit buffers) and `bool` (edit tracking flags). These cannot be selectively cleaned with `remove_by_type` since it would affect all uses of those types. They persist but are overwritten when the same widget IDs are reused.

## Testing

1. **Manual test**: Open 10+ markdown files with code blocks, tables, and formatted text
2. Close all tabs
3. Repeat cycle 5+ times
4. Memory should stay stable (not grow unbounded)

Note: This primarily affects "rendered" view mode. Raw mode has simpler state management.

## Dependencies

- egui's `IdTypeMap::remove_by_type()` API

## Related Work

- **Task 6**: Viewer state cleanup (CSV/Tree/SyncScroll HashMaps)
- **docs/technical/planning/viewer-state-cleanup.md**: Documents the HashMap cleanup
