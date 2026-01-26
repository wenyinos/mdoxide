# Document Navigation Buttons

## Overview

Document navigation buttons provide a quick way to jump to the top, middle, or bottom of a document. They appear as a subtle floating overlay in the top-left corner of the editor area.

## User Interface

### Button Appearance

- **Position**: Top-left corner of the editor, with 8px margin
- **Visibility**: Buttons are hidden by default and only appear when the mouse is near the button area (within 20px expanded hover zone)
- **Styling**: Semi-transparent buttons that become more visible on hover
  - Idle: 40% opacity
  - Hover: 87% opacity with subtle border

### Button Icons

| Button | Icon | Action |
|--------|------|--------|
| Top | ⤒ | Jump to document start |
| Middle | ◉ | Jump to document center |
| Bottom | ⤓ | Jump to document end |

### Theme Support

Buttons automatically adapt to the current theme:
- **Dark mode**: Dark background with light text
- **Light mode**: Light background with dark text

## Behavior

### In Raw Editor (FerriteEditor)

When clicking a navigation button in the raw editor:

- **Top**: Scrolls to line 0, places cursor at position (0, 0)
- **Middle**: Scrolls to center the middle line of the document, places cursor at that line
- **Bottom**: Scrolls to show the last line, places cursor at end of document

### In Rendered Mode (MarkdownEditor)

When clicking a navigation button in rendered mode:

- **Top**: Scrolls to the top of the rendered content (offset 0)
- **Middle**: Scrolls to center the middle of the content
- **Bottom**: Scrolls to the bottom of the content

Note: In rendered mode, cursor position is not tracked as it is in raw mode.

## Keyboard Shortcuts

The navigation buttons complement existing keyboard shortcuts:

| Shortcut | Action |
|----------|--------|
| `Ctrl+Home` | Jump to document start (same as Top button) |
| `Ctrl+End` | Jump to document end (same as Bottom button) |

There is currently no keyboard shortcut for jumping to the middle of the document.

## Implementation Details

### Module Location

The navigation button implementation is located in `src/ui/nav_buttons.rs`.

### Key Functions

```rust
/// Renders navigation buttons overlay and returns any requested action.
pub fn render_nav_buttons(ui: &mut Ui, editor_rect: Rect, is_dark_mode: bool) -> NavAction

/// Action requested by navigation button click.
pub enum NavAction {
    None,    // No button clicked
    Top,     // Jump to top
    Middle,  // Jump to middle
    Bottom,  // Jump to bottom
}
```

### Integration Points

1. **FerriteEditor** (`src/editor/ferrite/editor.rs`):
   - Navigation buttons are rendered at the end of the `ui()` method
   - Actions are handled by calling `view.scroll_to_line()` and `set_cursor()`

2. **MarkdownEditor** (`src/markdown/editor.rs`):
   - Navigation buttons are rendered after the scroll area in `show_rendered_editor()`
   - Actions store the target scroll offset in egui memory for the next frame
   - The stored offset is read and applied before the scroll area is created

### Constants

```rust
const BUTTON_SIZE: f32 = 24.0;    // Button dimensions
const BUTTON_SPACING: f32 = 2.0;  // Vertical spacing between buttons
const MARGIN: f32 = 8.0;          // Distance from editor edge
const IDLE_ALPHA: u8 = 100;       // Transparency when not hovered
const HOVER_ALPHA: u8 = 220;      // Transparency when hovered
```

## Test Strategy

1. **Click each button** - Verify jumps to correct position
2. **Test in raw mode** - Verify cursor moves with scroll
3. **Test in rendered mode** - Verify scroll without cursor movement
4. **Test with various file sizes** - Ensure works with small and large documents
5. **Verify buttons don't obstruct content** - Buttons should fade when not in use
6. **Test hover states** - Buttons should become more visible on hover
7. **Test theme compatibility** - Verify appearance in both dark and light modes
