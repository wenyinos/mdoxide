# Go to Line Dialog

## Overview

The Go to Line feature (Ctrl+G) provides a modal dialog for navigating directly to a specific line number in the editor. The cursor moves to the target line and the viewport is centered on it.

## Key Files

- `src/ui/dialogs.rs` - `GoToLineDialog` struct and `GoToLineResult` enum
- `src/ui/mod.rs` - Public exports for dialog types
- `src/state.rs` - `UiState.go_to_line_dialog` field
- `src/app.rs` - Keyboard shortcut handling and navigation logic

## Implementation Details

### Dialog Component (`src/ui/dialogs.rs`)

```rust
pub struct GoToLineDialog {
    pub line_input: String,      // User input text
    pub current_line: usize,     // Shown as placeholder (1-indexed)
    pub max_line: usize,         // Document line count
    pub error_message: Option<String>,
}

pub enum GoToLineResult {
    None,                        // Dialog still open
    Cancelled,                   // User cancelled (Escape)
    GoToLine(usize),            // Navigate to line (1-indexed)
}
```

### Key Design Decisions

1. **Global Enter key handling** - Enter key is checked at the start of `show()` before window rendering to ensure it works regardless of TextEdit focus state.

2. **Input validation** - Only numeric input accepted; empty input defaults to current line.

3. **Line clamping** - Target line is clamped to valid range `[1, max_line]` to handle out-of-bounds input gracefully.

4. **Viewport centering** - Uses existing `pending_scroll_to_line` mechanism to center the target line approximately 1/3 from the top of the viewport.

### Keyboard Shortcut (`src/app.rs`)

```rust
enum KeyboardAction {
    // ...
    GoToLine,  // Ctrl+G
}

// In handle_keyboard_shortcuts():
if i.modifiers.ctrl && !i.modifiers.shift && i.key_pressed(egui::Key::G) {
    return Some(KeyboardAction::GoToLine);
}
```

### Navigation Logic

```rust
fn handle_go_to_line(&mut self, target_line: usize) {
    // 1. Calculate character index for line start
    // 2. Update cursor position via Tab.cursors
    // 3. Set pending_scroll_to_line for viewport centering
}
```

## Dependencies Used

- `egui` - Window, TextEdit, layout, key handling

## Usage

1. Press **Ctrl+G** to open the dialog
2. Enter a line number (or leave empty to use current line)
3. Press **Enter** or click **Go** to navigate
4. Press **Escape** or click **Cancel** to close without navigating

## UI Appearance

- Modal window centered on screen
- Single-line text input with current line as placeholder
- "Range: 1 - N" hint showing valid line numbers
- Go and Cancel buttons

## Tests

Unit test in `src/ui/dialogs.rs`:

```rust
#[test]
fn test_go_to_line_dialog() {
    let dialog = GoToLineDialog::new(10, 100);
    assert_eq!(dialog.current_line, 10);
    assert_eq!(dialog.max_line, 100);
    assert!(dialog.line_input.is_empty());
}
```

Run tests:
```bash
cargo test go_to_line
```

## Related Features

- Outline Panel navigation (click heading to scroll)
- Search result navigation (F3/Shift+F3)
- `scroll_to_line` in `EditorWidget`
