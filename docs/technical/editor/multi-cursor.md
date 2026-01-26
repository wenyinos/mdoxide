# Multi-Cursor Editing

## Overview

Multi-cursor editing allows users to edit text at multiple positions simultaneously. This implementation lives entirely within `FerriteEditor` and supports:

- **Ctrl+Click**: Add cursor at clicked position
- **Simultaneous typing**: Text inserted at all cursor positions
- **Simultaneous deletion**: Backspace/Delete affects all cursors
- **Cursor navigation**: Arrow keys move all cursors together
- **Escape**: Clear extra cursors, return to single cursor

## Key Files

| File | Purpose |
|------|---------|
| `src/editor/ferrite/editor.rs` | Main implementation - `selections: Vec<Selection>`, edit methods |
| `src/editor/ferrite/cursor.rs` | `Selection` and `Cursor` types |

## Implementation Details

### Data Structure

```rust
// In FerriteEditor struct
pub(crate) selections: Vec<Selection>,
pub(crate) primary_selection_index: usize,
```

Replaced single `selection: Selection` with a vector of selections. The primary selection is tracked by index and used for:
- IME input positioning
- Bracket matching reference point
- Single-cursor fallback operations

### Key Methods

#### Cursor Management
- `add_cursor(cursor: Cursor)` - Add a new cursor position
- `clear_extra_cursors()` - Remove all but primary cursor
- `has_multiple_cursors()` - Check if multi-cursor mode is active
- `merge_overlapping_selections()` - Combine overlapping selections after edits

#### Multi-Cursor Edit Operations
- `insert_text_at_all_cursors(text: &str)` - Insert text at every cursor
- `backspace_at_all_cursors()` - Delete char before each cursor
- `delete_at_all_cursors()` - Delete char after each cursor
- `move_all_cursors(key, modifiers)` - Apply navigation to all cursors

### Offset Adjustment Strategy

When editing with multiple cursors, changes at earlier positions affect the character offsets of later cursors. The solution has two critical parts:

#### Part 1: Capture Positions BEFORE Modifications

**Critical**: Cursor positions must be captured as character offsets BEFORE any buffer modifications. After the buffer changes, the old `Cursor` line/column values become invalid and can cause panics.

```rust
// CORRECT: Capture positions first
let original_positions: Vec<(usize, usize)> = self.selections
    .iter()
    .enumerate()
    .map(|(idx, s)| (idx, cursor_to_char_pos(&self.buffer, &s.head)))
    .collect();

// Then modify buffer...
// Then recalculate cursor positions from char offsets
```

#### Part 2: Delete from End to Start

Processing deletions from end to start ensures earlier positions remain valid:

1. **Capture all original char positions** before any modifications
2. **Calculate delete targets** (char_pos - 1 for backspace, char_pos for delete)
3. **Sort descending and deduplicate** - process from end first
4. **Perform deletions** - no offset adjustment needed when going backwards
5. **Recalculate cursor positions** based on how many deletions occurred before each

```rust
// Example: backspace at all cursors
// 1. Capture original positions
let original_positions: Vec<(usize, usize)> = self.selections
    .iter()
    .enumerate()
    .map(|(idx, s)| (idx, cursor_to_char_pos(&self.buffer, &s.head)))
    .collect();

// 2. Get unique delete targets (char BEFORE cursor)
let mut delete_targets: Vec<usize> = original_positions
    .iter()
    .filter_map(|(_, pos)| if *pos > 0 { Some(*pos - 1) } else { None })
    .collect();
delete_targets.sort_by(|a, b| b.cmp(a));
delete_targets.dedup();

// 3. Delete from end to start
for delete_at in &delete_targets {
    self.buffer.remove(*delete_at, 1);
}

// 4. Recalculate positions
delete_targets.sort(); // ascending for offset calculation
for (idx, original_pos) in original_positions {
    let deletions_before = delete_targets.iter()
        .filter(|&&del_pos| del_pos < original_pos)
        .count();
    let new_pos = original_pos.saturating_sub(deletions_before);
    // Convert new_pos back to Cursor...
}
```

### Cursor Merging

After edits or navigation, cursors may overlap. The merge algorithm:

1. Sort selections by start position
2. Iterate through, merging adjacent/overlapping selections
3. Update `primary_selection_index` if it becomes invalid

### Navigation Keys

When multiple cursors exist, navigation keys (arrows, Home, End) move ALL cursors:

```rust
if is_navigation && self.has_multiple_cursors() {
    self.move_all_cursors(*key, modifiers);
    continue;
}
```

Each cursor moves independently based on its position (e.g., ArrowUp from different columns results in different final columns).

## Usage

### Adding Cursors
- **Ctrl+Click**: Add cursor at clicked position
- Cursors are rendered as vertical lines (same as primary cursor)
- Selections are rendered with semi-transparent background

### Editing
- Type normally - text appears at all cursor positions
- Backspace/Delete - works at all positions
- Arrow keys - all cursors move together

### Clearing Cursors
- **Escape**: Return to single cursor mode
- Clicking without Ctrl: Also clears extra cursors

## Testing

```bash
cargo run --release
```

### Test Cases
1. Ctrl+Click 3+ different positions
2. Type "hello" - should appear at all positions
3. Press Backspace 3x - should delete "llo" from all
4. Press ArrowRight - all cursors move right
5. Press Escape - return to single cursor

### Edge Cases Handled
- Cursors at same position: Deduplicated, single edit
- Cursors that merge after deletion: Combined automatically
- Cursor at buffer start: Backspace is no-op for that cursor
- Cursor at buffer end: Delete is no-op for that cursor
- Lines deleted during edit: Cursor positions recalculated from char offsets (no panic)
- Multiple cursors on same line: Each deletes independently, positions tracked correctly

## Performance Considerations

- Cursor operations are O(n) where n = number of cursors
- Sorting for offset adjustment is O(n log n)
- Merge is O(n) with single pass
- Typical use: 2-10 cursors, so performance is not a concern

## Limitations

- No "Ctrl+D" for select next occurrence (future enhancement)
- No column selection mode (future enhancement)
- Undo/redo doesn't track multi-cursor state (uses single selection)
