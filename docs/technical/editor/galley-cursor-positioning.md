# Galley-Based Cursor Positioning

## Overview

Implemented pixel-accurate cursor positioning when clicking on formatted text (bold, italic, code, etc.) in WYSIWYG mode. Uses egui's Galley text layout system instead of proportional width estimation.

## Problem

When clicking on rendered markdown text containing formatting like `**bold**` or `*italic*`:
- The displayed text has different length than raw text (markers stripped)
- Character widths vary based on font metrics
- Simple proportional mapping (`click_x / width * text_len`) produced inaccurate cursor positions

## Solution

Use egui's `Galley::cursor_from_pos()` for font-metric-aware character index mapping:

```rust
fn compute_displayed_cursor_index(
    ui: &Ui,
    displayed_text: &str,
    click_pos: egui::Pos2,
    text_rect: egui::Rect,
    font_size: f32,
    editor_font: &EditorFont,
) -> usize {
    // Create galley with same font as rendering
    let font_family = fonts::get_styled_font_family(false, false, editor_font);
    let font_id = FontId::new(font_size, font_family);
    
    let galley = ui.fonts(|f| {
        f.layout_no_wrap(displayed_text.to_owned(), font_id, Color32::PLACEHOLDER)
    });

    // Convert click to local coordinates
    let local_pos = egui::Vec2::new(
        click_pos.x - text_rect.min.x,
        click_pos.y - text_rect.min.y,
    );

    // Get exact character index
    let cursor = galley.cursor_from_pos(local_pos);
    cursor.ccursor.index
}
```

## Key Files

| File | Purpose |
|------|---------|
| `src/markdown/editor.rs` | Helper function and click handlers |

## Implementation Details

### Helper Function Location
`compute_displayed_cursor_index()` at ~line 2418 in `editor.rs`

### Click Handlers Updated
Three locations now use the Galley-based positioning:
1. `render_paragraph_with_structural_keys()` (~line 1545)
2. `render_paragraph()` (~line 2262)  
3. `render_list_item()` (~line 3216)

### Font Consistency
- Uses the same base font (`get_styled_font_family(false, false, editor_font)`) as text rendering
- Font size matches the `font_size` parameter passed to render functions
- Color is `PLACEHOLDER` since it doesn't affect measurement

## API Reference

### egui Galley Methods Used

```rust
// Create galley for text measurement
ui.fonts(|f| f.layout_no_wrap(text, font_id, color)) -> Arc<Galley>

// Map position to cursor
galley.cursor_from_pos(local_pos: Vec2) -> Cursor

// Get character index
cursor.ccursor.index -> usize
```

## Limitations

- Uses base font (non-bold, non-italic) for measurement
- Mixed-style text may have slight positioning variance
- Assumes single-line text (no wrapping)

## Related Tasks

- **Task 4**: Maps displayed index → raw index (handles `**`, `*`, etc. markers)
- **Task 5**: Integration testing across all render locations

## Version

Introduced in v0.2.5.1 as part of cursor positioning hotfix.
