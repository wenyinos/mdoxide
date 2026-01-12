# Move Line Up/Down

## Overview

Move Line Up/Down (Alt+↑/↓) allows users to reposition the current line by swapping it with the adjacent line above or below. The cursor follows the moved line, maintaining column position.

## Key Files

| File | Purpose |
|------|---------|
| `src/app.rs` | Key consumption before render, `handle_move_line()` implementation |
| `src/state.rs` | `pending_cursor_restore` field for cursor positioning |
| `src/editor/widget.rs` | Applies pending cursor position after TextEdit render |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Alt+Up` | Move current line up (swap with line above) |
| `Alt+Down` | Move current line down (swap with line below) |

## Implementation Details

### Key Consumption Strategy

The implementation uses a **pre-render key consumption** pattern to prevent egui's TextEdit from processing the arrow keys:

1. **Before render**: `consume_move_line_keys()` uses `ctx.input_mut()` with `consume_key()` to intercept Alt+Arrow keys
2. **During render**: TextEdit renders without processing the consumed arrow keys
3. **After render**: `handle_move_line()` executes with correct cursor position

This pattern is critical because without consuming the keys before render, TextEdit would process the arrow key first (moving the cursor), causing an off-by-one error in line detection.

```rust
// Before render - consume keys to prevent TextEdit processing
fn consume_move_line_keys(&mut self, ctx: &egui::Context) -> Option<isize> {
    ctx.input_mut(|i| {
        if i.consume_key(egui::Modifiers::ALT, egui::Key::ArrowUp) {
            return Some(-1);
        }
        if i.consume_key(egui::Modifiers::ALT, egui::Key::ArrowDown) {
            return Some(1);
        }
        None
    })
}
```

### Line Swap Algorithm

The move operation uses a simple swap approach:

1. Get current line number from `tab.cursor_position` (0-indexed)
2. Check boundary conditions (can't move first line up, can't move last line down)
3. Split content into lines vector
4. Swap current line with adjacent line using `Vec::swap()`
5. Join lines back into content string

### Cursor Following

To ensure the cursor follows the moved line, the implementation uses `pending_cursor_restore`:

1. Calculate new cursor position (new line start + original column)
2. Set `tab.pending_cursor_restore = Some(new_cursor_char)`
3. On next frame, EditorWidget applies this position to egui's TextEdit state

This is necessary because setting cursor position directly would be overwritten by TextEdit's internal state restoration.

## Behavior

| Scenario | Result |
|----------|--------|
| Cursor on line 2, Alt+Up | Line 2 swaps with line 1, cursor moves to line 1 |
| Cursor on line 2, Alt+Down | Line 2 swaps with line 3, cursor moves to line 3 |
| Cursor on first line, Alt+Up | No action (boundary) |
| Cursor on last line, Alt+Down | No action (boundary) |
| Cursor at column 5, move line | Cursor stays at column 5 on new line |

## Undo/Redo Support

Each move operation is recorded via `tab.record_edit()`, allowing:
- Single undo to restore line to original position
- Redo to re-apply the move

## Related Patterns

This implementation shares patterns with:
- **Undo/Redo keys** (`consume_undo_redo_keys`) - Same pre-render key consumption pattern
- **Duplicate Line** (`handle_duplicate_line`) - Similar line manipulation and cursor handling
- **Go to Line** - Uses same `pending_cursor_restore` mechanism for cursor positioning

## Testing

Manual test cases:
1. Move single line up and down multiple times
2. Verify boundary behavior at document start/end
3. Verify cursor column is preserved after move
4. Verify undo restores original position
5. Rapid consecutive moves (Alt+Down multiple times)
