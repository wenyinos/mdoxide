# Block Element Alignment

## Overview

Fixed visual alignment inconsistency where block-level elements (tables, code blocks, blockquotes, etc.) rendered at 0px left margin while paragraphs and headers had 4px indent.

## Problem

Some block elements rendered flush against the left edge while text content had a 4px base indent, creating visual misalignment in the rendered view.

**Affected elements:**
- Tables (`EditableTable`)
- Code blocks (`EditableCodeBlock`)
- Mermaid diagrams (`MermaidBlock`)
- Blockquotes
- Thematic breaks (horizontal rules)
- Front matter blocks

**Not affected (already correct):**
- Paragraphs (4px base indent)
- Headers (4px base indent)

## Solution

Added consistent `BASE_INDENT` (4.0px) to all block-level rendering functions by wrapping content in `ui.horizontal()` with `ui.add_space(BASE_INDENT)`.

## Key Files

- `src/markdown/editor.rs` - All rendering functions

## Implementation Details

### Pattern Used

```rust
const BASE_INDENT: f32 = 4.0;

ui.horizontal(|ui| {
    ui.add_space(BASE_INDENT);
    // ... render widget
}).inner;
```

### Functions Modified

| Function | Change |
|----------|--------|
| `render_table` | Wrapped `EditableTable::show()` in horizontal with indent |
| `render_code_block` | Wrapped `EditableCodeBlock::show()` in horizontal with indent |
| `render_mermaid_block` | Wrapped `MermaidBlock::show()` in horizontal with indent |
| `render_blockquote` | Added `ui.add_space(BASE_INDENT)` before quote border |
| `render_blockquote_with_structural_keys` | Same as above |
| `render_thematic_break` | Wrapped horizontal rule in horizontal with indent |
| `render_front_matter` | Wrapped frame in horizontal with indent |

## Testing

Visual verification:
1. Open a markdown file with tables, code blocks, blockquotes
2. All elements should align at the same left margin
3. Headers, paragraphs, and block elements share consistent 4px indent

## Related

- [CJK Paragraph Indentation](./cjk-paragraph-indentation.md) - First-line indent for CJK text
- [Click-to-Edit Formatting](./click-to-edit-formatting.md) - Paragraph rendering modes
