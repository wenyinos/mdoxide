# Terminal CJK / Double-Width Character Rendering

## Overview

CJK (Chinese, Japanese, Korean) and other East Asian characters occupy two columns in a terminal grid instead of one. The terminal emulator uses the `unicode-width` crate to detect these characters and adjusts the screen buffer, cursor advancement, and rendering accordingly.

## Implementation

### Screen Buffer (`src/terminal/screen.rs`)

**Cell flags:**
- `Cell::wide` — the cell holds the leading half of a double-width character.
- `Cell::wide_continuation` — the cell is the trailing (placeholder) half; it stores no visible character.

**`put_char()` logic:**
1. `UnicodeWidthChar::width(ch)` determines if a character is double-width (width == 2).
2. If the cursor is at the last column and the character needs two cells, auto-wrap triggers early (or truncation if auto-wrap is off).
3. Overwriting either half of an existing wide pair clears the orphaned partner cell.
4. For width-2 characters: the primary cell gets `wide = true`, the next cell becomes a `wide_continuation` placeholder.
5. Cursor advances by 2 columns instead of 1.

**Text extraction methods** (`get_row_text`, `get_selected_text`, `get_visible_text`, `screen_contains`, `export_html`) all skip continuation cells so copied/exported text contains the original Unicode characters without phantom spaces.

### Renderer (`src/terminal/widget.rs`)

**`render_screen()`:**
- Continuation cells are skipped entirely (no double-draw).
- Wide character cells render with `cell_width = char_size.x * 2.0`, spanning two grid columns.
- Selection highlight covers the full double-width rect.
- Underline and strikethrough decorations span the full cell width.

**`render_cursor()`:**
- If the cursor lands on a continuation cell, it snaps back to the leading cell.
- The cursor rect spans two columns when on a wide character.

**Hit-testing / selection:**
- `snap_wide_char()` adjusts click coordinates so clicking on a continuation cell selects the leading cell.
- Drag-selection respects wide character boundaries.

## Dependencies

- `unicode-width = "0.1"` — Unicode Standard Annex #11 East Asian Width classification.

## Edge Cases

| Scenario | Behavior |
|----------|----------|
| Wide char at last column | Auto-wrap to next line; last cell cleared |
| Overwrite leading half of wide char | Trailing half cleared to space |
| Overwrite trailing half of wide char | Leading half cleared to space |
| Zero-width / combining characters | Treated as width 1 (occupies one cell) |
| Ambiguous-width characters | Treated as narrow (width 1) by default |

## Test Cases

```bash
echo 'Hello 世界'           # Wide chars span 2 cells, no overlap
echo 'A半角全角ABC'          # Mixed narrow + wide widths
printf '%-20s|\n' '你好世界'  # Column alignment preserved
```
