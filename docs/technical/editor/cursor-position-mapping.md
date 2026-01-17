# Cursor Position Mapping

## Overview

Maps cursor positions between displayed text (rendered without markdown formatting) and raw text (containing markdown markers like `**`, `*`, etc.). This enables accurate cursor placement when clicking on formatted content to enter edit mode.

## Key Files

- `src/markdown/editor.rs` - Contains `map_displayed_to_raw()` function (~line 2467)

## Implementation Details

### The Problem

When a user clicks on formatted markdown content like `**bold**` text, the rendered view shows just `bold`. The click position is captured relative to the displayed text, but the cursor needs to be placed in the raw text which includes the `**` markers.

Example:
- Raw text: `*h**e**re*` (nested italic + bold)
- Displayed: `here`
- Click on 'r' (displayed index 2) → needs raw index 7

### Solution: `map_displayed_to_raw()`

```rust
fn map_displayed_to_raw(displayed_idx: usize, raw_text: &str) -> usize
```

The function walks through the raw text character by character, skipping formatting markers while counting displayed characters. When the displayed count reaches the target index, it returns the raw position.

### Formatting Markers Handled

| Marker | Pattern | Description |
|--------|---------|-------------|
| Bold | `**` or `__` | Skips 2 chars |
| Italic | `*` or `_` | Skips 1 char (when not part of bold) |
| Code | `` ` `` | Skips backticks |
| Strikethrough | `~~` | Skips 2 chars |
| Links | `[text](url)` | Skips `[`, `](url)`, keeps `text` |

### Algorithm

1. Walk through raw text character by character
2. Check for formatting marker patterns in priority order (double-char first)
3. Skip marker characters without advancing displayed position
4. For regular content, advance both raw and displayed positions
5. Return raw position when displayed count reaches target

### Link Handling

Links require special handling since the structure is `[visible text](hidden url)`:
- Skip opening `[`
- Count text characters normally (they're displayed)
- Skip `](url)` entirely, including nested parentheses

## Integration Points

The mapping is used in three click handlers in `src/markdown/editor.rs`:

1. **Formatted paragraph (first location)** - ~line 1554
2. **Formatted paragraph (second location)** - ~line 2268  
3. **List items** - ~line 3228

Each handler:
1. Computes `displayed_idx` using `compute_displayed_cursor_index()` (Galley-based)
2. Maps to `raw_idx` using `map_displayed_to_raw()`
3. Sets `pending_cursor_pos` for the edit state

## Related Documentation

- [Galley Cursor Positioning](./galley-cursor-positioning.md) - Pixel-accurate displayed position via egui Galley
- [Click-to-Edit Formatting](./click-to-edit-formatting.md) - Hybrid editing for formatted content

## Testing

Test with formatted content:
- `- This is **bold** text` - Click on "bold" word
- `- Click *here* to edit` - Click on "here"
- `- Link [example](https://example.com) text` - Click on "example"
- `- Nested ***bold italic*** test` - Click in middle

Verify cursor appears within 1-2 characters of click position in edit mode.
