# CJK Paragraph Indentation

Implements first-line paragraph indentation for Chinese/Japanese writing conventions.

Reference: GitHub Issue #20

## Overview

In CJK (Chinese, Japanese, Korean) typography, paragraphs traditionally begin with first-line indentation:

- **Chinese**: 2 full-width spaces (2em)
- **Japanese**: 1 full-width space (1em)

This feature adds configurable paragraph indentation that applies to:
1. Rendered/Preview mode in the editor
2. HTML export

## Setting Options

The setting is available in **Settings > Editor > Paragraph Indentation**:

| Option | Value | Description |
|--------|-------|-------------|
| Off | 0 | No paragraph indentation (default) |
| Chinese (2em) | 2em | Two full-width characters indent |
| Japanese (1em) | 1em | One full-width character indent |
| Custom | 0.5-5em | User-specified em value |

## Implementation

### Settings (`src/config/settings.rs`)

The `ParagraphIndent` enum stores the indentation setting:

```rust
pub enum ParagraphIndent {
    Off,           // No indentation
    Chinese,       // 2em
    Japanese,      // 1em
    Custom(u8),    // Custom em value (stored as tenths: 15 = 1.5em)
}
```

Key methods:
- `to_em()` → Returns indentation in em units
- `to_pixels(font_size)` → Returns indentation in pixels
- `to_css()` → Returns CSS value string (e.g., "2em")

### Rendered View (`src/markdown/editor.rs`)

Indentation is applied in the `render_paragraph` and `render_paragraph_with_structural_keys` functions:

1. Calculate indentation: `paragraph_indent.to_pixels(font_size)`
2. Only apply to top-level paragraphs (indent_level == 0)
3. Add spacing before paragraph text content

### HTML Export (`src/export/html.rs`)

The `generate_paragraph_indent_css` function generates CSS when exporting:

```css
/* CJK Paragraph Indentation */
.markdown-body > p {
    text-indent: 2em;
}
```

The CSS selector `.markdown-body > p` targets direct child paragraphs only, avoiding indentation of paragraphs inside blockquotes or list items.

### UI (`src/ui/settings.rs`)

A dropdown in the Editor section allows selecting preset indentation modes or a custom value with a numeric input.

## Translations

Translations added to `locales/en.yaml`:

```yaml
settings.editor:
  paragraph_indent: "Paragraph Indentation"
  paragraph_indent_hint: "First-line indentation for CJK typography conventions"
  paragraph_indent_off: "Off"
  paragraph_indent_chinese: "Chinese (2em)"
  paragraph_indent_japanese: "Japanese (1em)"
  paragraph_indent_custom: "Custom"
  paragraph_indent_custom_value: "Indent (em):"
```

## Testing

1. **Rendered view**: Open a markdown file with paragraphs, enable Chinese or Japanese indentation in Settings > Editor > Paragraph Indentation, and verify paragraphs in rendered view have proper indentation
2. **HTML export**: Export to HTML and verify the CSS includes `text-indent` with correct value
3. **Different modes**: Test with Off, Chinese (2em), Japanese (1em), and Custom values
4. **Nested content**: Verify paragraphs inside blockquotes or lists do NOT get indented (only top-level paragraphs)
5. **Mixed content**: Test documents with CJK and English text together

## Files Modified

- `src/config/settings.rs` - Added `ParagraphIndent` enum and setting
- `src/config/mod.rs` - (auto-exported via `pub use settings::*;`)
- `src/ui/settings.rs` - Added UI controls
- `src/markdown/editor.rs` - Applied indentation in rendered view
- `src/export/html.rs` - Added CSS generation for HTML export
- `locales/en.yaml` - Added translations
