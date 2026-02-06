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

Indentation is applied in `render_paragraph()` and `render_paragraph_with_structural_keys()`:

1. Calculate indentation: `paragraph_indent.to_pixels(font_size)`
2. Only apply to top-level paragraphs (`indent_level == 0`)
3. A custom **LayoutJob layouter** is used for all paragraph TextEdits:
   - `LayoutJob::append(text, leading_space, format)` — the `leading_space` parameter adds space before the first character only; subsequent wrapped lines start flush left
   - This gives **true first-line-only indentation** directly inside `egui::TextEdit`
   - When `cjk_indent` is `0.0`, the leading space is a no-op (identical to default behavior)
4. For **formatted paragraphs** (with bold/italic/links):
   - Display mode: `ui.add_space(cjk_indent)` inside `horizontal_wrapped` (first-line only via multiple inline widgets)
   - Edit mode: Custom layouter with `leading_space` on TextEdit (first-line only)
5. For **simple paragraphs** (plain text):
   - Custom layouter with `leading_space` on TextEdit (first-line only, always editable)

### HTML Export (`src/export/html.rs`)

CSS `text-indent` is applied to paragraph styles when indentation is enabled:
```css
p { text-indent: 2em; } /* Chinese */
p { text-indent: 1em; } /* Japanese */
```

## Behavior Summary

| Mode | First-line indent | All-lines indent | Notes |
|------|-------------------|------------------|-------|
| Display (formatted) | ✓ | - | `horizontal_wrapped` + spacer |
| Edit (formatted) | ✓ | - | LayoutJob `leading_space` in TextEdit |
| Simple paragraph | ✓ | - | LayoutJob `leading_space` in TextEdit |
| No CJK indent | N/A | N/A | `leading_space: 0.0` — no-op (unchanged) |
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

1. **LayoutJob overrides default TextEdit layout**: The custom layouter replaces egui's default text layout for paragraphs. This is transparent when `cjk_indent` is `0.0` (the `leading_space` is a no-op), but means paragraph TextEdits always go through a custom LayoutJob rather than the built-in layouter.

2. **Formatted paragraph display mode** still uses `horizontal_wrapped` with multiple inline widgets — this works correctly for first-line indent because the spacer widget only occupies the first row.

## Configuration

Settings stored in: `~/.config/ferrite/settings.json` (Linux/macOS) or `%APPDATA%\ferrite\settings.json` (Windows)

```json
{
  "paragraph_indent": "chinese"  // or "japanese", "off", {"custom": 15}
}
```
