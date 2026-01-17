# Table Editing Focus Fix

**Status**: Resolved  
**Version**: v0.2.5

## Summary

Fixed the cursor focus loss issue when editing table cells in Rendered and Split view modes. Table cells now buffer edits and only commit changes to the source when focus leaves the table, preventing the re-parsing loop that caused focus loss.

## Previous Issue

When editing table cells in Rendered or Split mode:
1. Cell text changes → `output.changed = true` immediately
2. Source markdown was updated on every keystroke
3. Document was re-parsed on next frame
4. New AST created new widget IDs
5. Previous TextEdit widget was gone, focus was lost

This made table editing in WYSIWYG modes frustrating as users had to click back into cells after each keystroke.

## Solution Implemented

### 1. Deferred Change Signaling

Table cells now track modifications separately from change signaling:

```rust
pub struct TableEditState {
    pub focused_cell: Option<(usize, usize)>,
    pub pending_focus: Option<(usize, usize)>,
    pub had_focus_last_frame: bool,    // Track focus across frames
    pub content_modified: bool,         // Track if edits were made
}
```

### 2. Focus Loss Detection

Changes are only committed when focus leaves the table entirely:

```rust
// Detect focus loss: had focus last frame but not this frame
let focus_lost = edit_state.had_focus_last_frame && !any_cell_has_focus;

if focus_lost && edit_state.content_modified {
    changed = true;  // Now signal the change
    edit_state.content_modified = false;
}
```

### 3. Improved TextEdit Rendering

Switched from `ui.add(TextEdit...)` to `TextEdit::show(ui)` for better cursor state management, matching how other working editable widgets (headings) are implemented.

## Current Behavior

- **While editing a cell**: Type freely, cursor stays in position, source is NOT updated
- **Click outside the table**: Focus leaves, changes are committed to source
- **Tab/Enter navigation**: Focus moves to another cell (still in table), changes NOT committed yet
- **Escape**: Clears focus, changes are committed
- **Add/Remove row/column buttons**: Changes committed immediately (structural actions)

## Files Modified

| File | Changes |
|------|---------|
| `src/markdown/widgets.rs` | Added `had_focus_last_frame` and `content_modified` to `TableEditState`, deferred change detection, switched to `TextEdit::show()` |

## Keyboard Navigation

The table supports full keyboard navigation:

| Key | Action |
|-----|--------|
| Tab | Move to next cell (right, then wrap to next row) |
| Shift+Tab | Move to previous cell (left, then wrap to previous row) |
| Enter | Move to next row (same column) |
| Escape | Exit table editing (commits changes) |

## Testing

1. Open a markdown file with a table in Split or Rendered mode
2. Click on a table cell
3. Type text - cursor should stay in position
4. Tab between cells - focus should move correctly
5. Click outside the table - changes should appear in raw source
6. Verify undo/redo works for table edits

## Trade-offs

The deferred update approach means:
- **Pro**: Smooth editing experience without focus loss
- **Pro**: Cursor positioning works correctly on click
- **Con**: Preview/raw view doesn't update until you click away from the table

This matches the behavior of other click-to-edit widgets (paragraphs, list items) and provides a consistent editing experience.

## Related Documentation

- [Editable Tables](./editable-tables.md) - Full table widget documentation
- [Split View](./split-view.md) - Split view mode documentation
- [WYSIWYG Editor](./wysiwyg-editor.md) - Overall editor architecture
