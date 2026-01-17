# Multi-Cursor Editing (Partial Implementation)

> **Status: DEFERRED** - Core infrastructure implemented, but text editing with multiple cursors is not functional due to egui limitations. The Ctrl+D "select next occurrence" feature works standalone for find/replace workflows.

## What Works ✅

### Ctrl+D: Select Next Occurrence
This is the main usable feature from this implementation:

1. **First press**: Selects the word under the cursor
2. **Repeated presses**: Finds and selects the next occurrence of that word
3. **Wraps around**: Continues searching from document start after reaching the end
4. **Skips duplicates**: Won't select already-selected occurrences

**Use case**: Quickly find all occurrences of a variable name or word for review or replacement.

### Ctrl+Click: Add Cursor
- Hold Ctrl and click to add a cursor at that position
- Multiple cursors are visually rendered with blue carets
- Selection highlights shown in semi-transparent blue

### Escape: Exit Multi-Cursor Mode
- Press Escape to collapse all cursors back to a single cursor
- Returns to the primary cursor position

### Visual Rendering
- Additional cursors shown as blue vertical lines (2px wide)
- Selections shown as semi-transparent blue rectangles
- Colors adapt to dark/light theme
- Primary cursor handled by egui's native TextEdit

## What Does NOT Work ❌

### Text Operations (Typing/Deleting)
**This is the critical limitation that makes full multi-cursor editing non-functional.**

- When you type, text only appears at the PRIMARY cursor
- Backspace/Delete only works at the primary cursor
- All other cursors remain static and don't receive input

**Why?** egui's `TextEdit` widget is fundamentally designed for single-cursor editing. It handles all text input internally with no hooks for multi-cursor operations.

### Column Selection (Alt+Click+Drag)
Not implemented. Would require:
- Detecting drag events with Alt modifier
- Converting drag rectangle to line/column positions
- Creating cursors across multiple lines

### Compound Undo/Redo
Multi-cursor edits are not grouped as single undo operations. The current undo system stores entire document snapshots, not cursor-aware edit operations.

## Technical Implementation

### Data Structures (`src/state.rs`)

```rust
/// A single cursor or selection
pub struct Selection {
    pub anchor: usize,           // Fixed end (char index)
    pub head: usize,             // Cursor position (char index)
    pub preferred_column: Option<usize>, // For vertical nav
}

/// Collection of cursors/selections
pub struct MultiCursor {
    selections: Vec<Selection>,  // Sorted, non-overlapping
    primary_index: usize,        // For status bar, scroll
}
```

### Tab Integration
```rust
pub struct Tab {
    pub cursors: MultiCursor,              // New: multi-cursor state
    pub cursor_position: (usize, usize),   // Legacy: synced from primary
    pub selection: Option<(usize, usize)>, // Legacy: synced from primary
    // ...
}
```

### Key Methods

| Method | Purpose |
|--------|---------|
| `Tab::add_cursor(pos)` | Add cursor at character position |
| `Tab::add_selection(anchor, head)` | Add selection range |
| `Tab::exit_multi_cursor_mode()` | Collapse to single cursor |
| `Tab::get_primary_selection_text()` | Get selected text or word at cursor |
| `Tab::find_next_occurrence(text, after)` | Find next match for Ctrl+D |
| `Tab::word_range_at_position(pos)` | Get word boundaries at position |

### Keyboard Handling (`src/app.rs`)

```rust
enum KeyboardAction {
    SelectNextOccurrence,  // Ctrl+D
    ExitMultiCursor,       // Escape (when multi-cursor active)
    // ...
}
```

### Rendering (`src/editor/widget.rs`)

Additional cursors rendered as painter overlays after TextEdit:
- `draw_cursor_caret()` - Draws blue vertical line at cursor position
- `draw_selection_highlight()` - Draws semi-transparent rectangle for selection

## Future Work Required

To make full multi-cursor editing work, one of these approaches would be needed:

### Option 1: Custom Editor Widget
Build a complete text editor from scratch that:
- Handles all keyboard input manually
- Maintains multiple cursor positions
- Applies text changes to all cursors with proper offset adjustments

### Option 2: Intercept and Replicate
After each edit through TextEdit:
- Detect what changed (diff old vs new content)
- Apply the same change at all other cursor positions
- Update cursor positions accounting for text length changes

Both options are significant undertakings and were deferred in favor of higher-priority features.

## Files Modified

| File | Changes |
|------|---------|
| `src/state.rs` | Added `Selection`, `MultiCursor` structs; Tab multi-cursor methods |
| `src/editor/widget.rs` | Added cursor/selection rendering; `EditorOutput.ctrl_click_pos` |
| `src/app.rs` | Added `SelectNextOccurrence`, `ExitMultiCursor` actions and handlers |

## Testing

Manual testing checklist:
- [x] Ctrl+D selects word under cursor (first press)
- [x] Ctrl+D finds next occurrences (repeated press)
- [x] Ctrl+Click adds cursor at click position
- [x] Multiple cursors are visually rendered (blue carets)
- [x] Escape clears to single cursor
- [x] Status bar shows primary cursor position
- [ ] ~~Typing at multiple cursors~~ (NOT WORKING)
- [ ] ~~Deleting at multiple cursors~~ (NOT WORKING)
- [ ] ~~Column selection~~ (NOT IMPLEMENTED)

## Summary

| Feature | Status |
|---------|--------|
| Ctrl+D select next occurrence | ✅ Working |
| Ctrl+Click add cursor | ✅ Working (visual only) |
| Visual multi-cursor rendering | ✅ Working |
| Escape to exit | ✅ Working |
| Text input at multiple cursors | ❌ Not working |
| Delete/backspace at multiple cursors | ❌ Not working |
| Alt+Click+Drag column selection | ❌ Not implemented |
| Compound undo/redo | ❌ Not implemented |
