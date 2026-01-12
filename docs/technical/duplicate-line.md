# Duplicate Line / Selection (Ctrl+Shift+D)

## Overview

Implements line and selection duplication functionality via Ctrl+Shift+D keyboard shortcut. Users can duplicate either the entire current line (when no selection exists) or duplicate selected text (inserting it immediately after the selection).

## Key Files

| File | Purpose |
|------|---------|
| `src/app.rs` | `DuplicateLine` variant in `KeyboardAction` enum, shortcut detection, `handle_duplicate_line()` method |
| `src/state.rs` | Tab struct with cursor/selection state, `record_edit()` for undo support |

## Implementation Details

### Keyboard Shortcut Detection

Added in `handle_keyboard_shortcuts()`:

```rust
// Ctrl+Shift+D: Duplicate line or selection
if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::D) {
    debug!("Keyboard shortcut: Ctrl+Shift+D (Duplicate Line)");
    return Some(KeyboardAction::DuplicateLine);
}
```

### Duplication Logic

The `handle_duplicate_line()` method handles two cases:

#### Case 1: No Selection (Line Duplication)
- Find line boundaries using byte positions
- Insert `\n` + line content after the current line
- Cursor stays on original line at same column

#### Case 2: With Selection (Selection Duplication)
- Get selected text using byte indices
- Insert selected text immediately after selection end
- New selection covers the duplicated text

### Character-to-Byte Index Conversion

**Critical implementation detail**: egui uses character indices for cursor positions, but Rust strings use byte indices for slicing and insertion. The helper closure handles this conversion:

```rust
let char_to_byte = |text: &str, char_idx: usize| -> usize {
    text.char_indices()
        .nth(char_idx)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or(text.len())
};
```

This is essential for correct behavior with:
- Multi-byte UTF-8 characters (emojis, CJK, etc.)
- Mixed ASCII and Unicode content

### Undo/Redo Support

Uses `Tab::record_edit()` to save the old content and cursor position before modification, ensuring single-step undo/redo.

## Usage

| Action | Behavior |
|--------|----------|
| Cursor on line, no selection | `Ctrl+Shift+D` → Full line duplicated below |
| Text selected | `Ctrl+Shift+D` → Selection duplicated at end, becomes new selection |

## Tests

Manual testing scenarios:

1. **Empty line**: Place cursor on empty line, press Ctrl+Shift+D → empty line appears below
2. **Middle of document**: Cursor in middle of line → line duplicated, cursor stays on original
3. **Word selection**: Select a word → word duplicated immediately after, duplicated text selected
4. **Multi-line selection**: Select 3 lines → all 3 duplicated as block
5. **Unicode content**: Test with emoji/CJK text → no character corruption
6. **Undo**: After duplication, Ctrl+Z reverts in single step
7. **Redo**: After undo, Ctrl+Y restores duplication

## Related Features

- [Go to Line](./go-to-line.md) - Ctrl+G line navigation
- [Keyboard Shortcuts](./keyboard-shortcuts.md) - Global shortcut system
- [Undo/Redo System](./undo-redo.md) - Edit history management
