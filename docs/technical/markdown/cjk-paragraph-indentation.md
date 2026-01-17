# CJK Paragraph Indentation

Implements paragraph indentation for Chinese/Japanese writing conventions.

Reference: GitHub Issue #20

## Overview

In CJK (Chinese, Japanese, Korean) typography, paragraphs traditionally begin with first-line indentation:

- **Chinese**: 2 full-width spaces (2em)
- **Japanese**: 1 full-width space (1em)

This feature adds configurable paragraph indentation that applies to:
1. Rendered/Preview mode in the editor
2. HTML export (via CSS `text-indent`)

## Implementation

### Settings (`src/config/settings.rs`)

```rust
pub enum ParagraphIndent {
    Off,           // No indentation (default)
    Chinese,       // 2em indent
    Japanese,      // 1em indent  
    Custom(u8),    // Custom value in tenths of em
}
```

Methods:
- `to_em()` - Returns indentation in em units
- `to_pixels(font_size)` - Returns indentation in pixels
- `to_css()` - Returns CSS `text-indent` value for HTML export

### Rendered View (`src/markdown/editor.rs`)

Indentation is applied in `render_paragraph()`:

1. Calculate indentation: `paragraph_indent.to_pixels(font_size)`
2. Only apply to top-level paragraphs (`indent_level == 0`)
3. For **formatted paragraphs** (with bold/italic/links):
   - Display mode: `ui.add_space(cjk_indent)` inside `horizontal_wrapped` (first-line only)
   - Edit mode: `ui.add_space(cjk_indent)` before TextEdit (all lines - egui limitation)
4. For **simple paragraphs** (plain text):
   - Uses TextEdit directly, indentation applies to all lines

### HTML Export (`src/export/html.rs`)

CSS `text-indent` is applied to paragraph styles when indentation is enabled:
```css
p { text-indent: 2em; } /* Chinese */
p { text-indent: 1em; } /* Japanese */
```

## Behavior Summary

| Mode | First-line indent | All-lines indent | Notes |
|------|-------------------|------------------|-------|
| Display (formatted) | ✓ | - | Uses `horizontal_wrapped` |
| Edit (formatted) | - | ✓ | egui TextEdit limitation |
| Simple paragraph | - | ✓ | egui TextEdit limitation |
| HTML export | ✓ | - | CSS `text-indent` |

## Testing

1. Open Settings > Editor > Paragraph Indentation
2. Select "Chinese (2em)" or "Japanese (1em)"
3. Open a markdown file with CJK content
4. Verify:
   - Formatted paragraphs show first-line indent in display mode
   - Paragraphs inside blockquotes have no indent (indent_level > 0)
   - HTML export includes `text-indent` CSS

Test file: `test_md/test_korean.md`

## Known Limitations

1. **TextEdit all-lines indent**: Due to egui's TextEdit widget design, when editing text (either in edit mode for formatted paragraphs or for simple paragraphs), the indentation applies to ALL lines, not just the first line.

2. **Simple paragraphs**: Paragraphs without any inline formatting (bold, italic, links) always use TextEdit and cannot have first-line-only indentation.

## Configuration

Settings stored in: `~/.config/ferrite/settings.json` (Linux/macOS) or `%APPDATA%\ferrite\settings.json` (Windows)

```json
{
  "paragraph_indent": "chinese"  // or "japanese", "off", {"custom": 15}
}
```
