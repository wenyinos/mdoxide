# PRD: Accurate Cursor Positioning for Formatted Click-to-Edit Elements

## Overview

Fix the cursor positioning issue when clicking on formatted list items and paragraphs (elements containing bold, italic, code, links, etc.) in rendered and split view modes. Currently, when a user clicks on these elements to enter edit mode, the cursor appears at an incorrect position - often significantly to the right of where they clicked.

## Problem Statement

Ferrite uses a "click-to-edit" pattern for formatted content:
1. **Display Mode**: Shows rendered markdown (bold text appears styled, `**` markers hidden)
2. **Edit Mode**: Shows raw markdown (includes `**`, `*`, `` ` ``, `[](url)` markers)

When the user clicks to enter edit mode, the cursor must be positioned in the **raw text** based on where they clicked in the **displayed text**. The challenge is that these have different character counts and the mapping is non-linear.

### Current Behavior
- Clicking near the middle of "Click **here** to see" positions cursor incorrectly
- The drift is worse on longer lines
- Users must manually reposition cursor after clicking

### Expected Behavior
- Cursor should appear at or very near the clicked position
- Position should be accurate regardless of line length or formatting complexity

## Technical Approach

Use egui's **Galley** text layout system to accurately map click positions to character indices.

### Key egui APIs (from Context7 research)

```rust
// Create a Galley from displayed text
let galley = ui.painter().layout_no_wrap(displayed_text, font_id, color);

// Convert click position to character cursor
let cursor = galley.cursor_from_pos(click_pos - text_rect.min);
let displayed_char_index = cursor.ccursor.index;
```

### Algorithm

1. **On Click (Display Mode)**:
   - Capture click position in screen coordinates
   - Get the displayed text (without formatting markers)
   - Create a Galley from the displayed text using the same font
   - Use `galley.cursor_from_pos()` to get exact character index in displayed text
   - Build a mapping from displayed text position to raw text position
   - Store the raw text cursor position in `FormattedItemEditState.pending_cursor_pos`

2. **On Edit Mode Enter**:
   - Apply the pending cursor position to the TextEdit (existing code)

### Position Mapping: Displayed → Raw

The displayed text and raw text differ due to formatting markers. We need to map positions:

| Raw Text | Displayed Text | Raw Index | Displayed Index |
|----------|----------------|-----------|-----------------|
| `C` | `C` | 0 | 0 |
| `l` | `l` | 1 | 1 |
| ... | ... | ... | ... |
| `*` | (hidden) | 6 | - |
| `*` | (hidden) | 7 | - |
| `h` | `h` | 8 | 6 |
| `e` | `e` | 9 | 7 |
| ... | ... | ... | ... |

**Mapping Function**: Walk through both texts in parallel, tracking formatting markers in raw text, to build the correspondence.

## Affected Code

- `src/markdown/editor.rs`:
  - `FormattedItemEditState` struct
  - `render_paragraph_sk()` function (2 locations)
  - `render_list_item()` function (2 locations)

## Success Criteria

1. Clicking on formatted text positions cursor within 1-2 characters of click location
2. Works correctly for all inline formatting: bold, italic, code, links, strikethrough
3. Works correctly for varying line lengths
4. No performance regression (Galley creation is lightweight)

## Out of Scope

- Multi-line paragraph cursor positioning (complex, separate issue)
- Nested formatting edge cases (defer to 0.3.0 if problematic)

## Testing

1. Create test file with various formatted list items:
   - `- This is **bold** text`
   - `- Click *here* to edit`
   - `- See `code` and **more**`
   - `- Link to [example](url) here`
2. Click at various positions in each item
3. Verify cursor appears near click location
