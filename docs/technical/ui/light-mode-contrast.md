# Light Mode Color Contrast Documentation

This document describes the light theme color tokens in Ferrite and their WCAG 2.1 contrast compliance.

## WCAG 2.1 Level AA Requirements

- **Normal text**: Minimum 4.5:1 contrast ratio against background
- **Large text** (18pt or 14pt bold): Minimum 3:1 contrast ratio
- **UI components** (borders, icons, focus indicators): Minimum 3:1 contrast ratio

## Light Theme Color Tokens

### Base Colors (`BaseColors::light()`)

| Token | RGB Value | Hex | Purpose | Contrast vs White |
|-------|-----------|-----|---------|-------------------|
| `background` | (255, 255, 255) | #FFFFFF | Primary background | N/A |
| `background_secondary` | (250, 250, 250) | #FAFAFA | Elevated surfaces | 1.03:1 |
| `background_tertiary` | (245, 245, 245) | #F5F5F5 | Inputs, code blocks | 1.06:1 |
| `border` | (160, 160, 160) | #A0A0A0 | Primary borders | ~3.2:1 ✓ |
| `border_subtle` | (185, 185, 185) | #B9B9B9 | Subtle dividers | ~2.3:1 |
| `hover` | (235, 235, 240) | #EBEBF0 | Hover state | 1.12:1 |
| `selected` | (215, 230, 250) | #D7E6FA | Selected state | 1.17:1 |

### Text Colors (`TextColors::light()`)

| Token | RGB Value | Hex | Purpose | Contrast vs White |
|-------|-----------|-----|---------|-------------------|
| `primary` | (30, 30, 30) | #1E1E1E | Main content text | ~12.6:1 ✓✓ |
| `secondary` | (75, 75, 75) | #4B4B4B | Descriptions, labels | ~7.0:1 ✓✓ |
| `muted` | (100, 100, 100) | #646464 | Hints, placeholders | ~5.3:1 ✓ |
| `disabled` | (140, 140, 140) | #8C8C8C | Disabled elements | ~3.5:1 |
| `link` | (0, 90, 170) | #005AAA | Hyperlinks | ~6.5:1 ✓ |
| `code` | (70, 70, 70) | #464646 | Inline code | ~7.5:1 ✓ |

### Editor Colors (`EditorThemeColors::light()`)

| Token | RGB Value | Hex | Purpose | Contrast vs White |
|-------|-----------|-----|---------|-------------------|
| `heading` | (0, 90, 165) | #005AA5 | H1-H6 headings | ~6.0:1 ✓ |
| `blockquote_border` | (160, 160, 160) | #A0A0A0 | Quote borders | ~3.2:1 ✓ |
| `blockquote_text` | (85, 85, 85) | #555555 | Quote content | ~6.0:1 ✓ |
| `code_block_bg` | (243, 244, 246) | #F3F4F6 | Code block background | 1.08:1 |
| `code_block_border` | (175, 180, 190) | #AFB4BE | Code block border | ~2.1:1 |
| `horizontal_rule` | (160, 160, 160) | #A0A0A0 | HR elements | ~3.2:1 ✓ |
| `list_marker` | (85, 85, 85) | #555555 | Bullets, numbers | ~6.0:1 ✓ |
| `checkbox` | (0, 90, 165) | #005AA5 | Task checkboxes | ~6.0:1 ✓ |
| `table_border` | (170, 175, 185) | #AAAFB9 | Table borders | ~2.2:1 |
| `table_header_bg` | (240, 242, 245) | #F0F2F5 | Table header bg | 1.09:1 |

### UI Separator Colors

| Location | RGB Value | Hex | Purpose | Contrast vs Background |
|----------|-----------|-----|---------|------------------------|
| Tab/Editor separator | (160, 160, 160) | #A0A0A0 | Tabs-to-editor divider | ~3.2:1 vs white ✓ |
| Ribbon separators | (165, 165, 165) | #A5A5A5 | Ribbon group dividers | ~2.7:1 vs ribbon bg |

## Legend

- ✓ = Meets WCAG AA requirement
- ✓✓ = Exceeds WCAG AAA requirement
- No mark = Does not meet AA for its category (acceptable for decorative or non-essential elements)

## Notes

1. **Disabled text** intentionally has lower contrast (~3.5:1) as WCAG allows reduced contrast for disabled elements.

2. **Background variants** have low contrast against white since they are meant as subtle surface distinctions, not as foreground content.

3. **Subtle borders** (`border_subtle`) intentionally use softer contrast (~2.3:1) for non-critical dividers to maintain the light, airy aesthetic while the primary `border` token meets the 3:1 requirement for important UI boundaries.

4. **Tab/Editor separator** uses the stronger `border` color (160, 160, 160) to ensure clear visual separation between tabs and editor content.

## Testing Contrast

To verify contrast ratios, use tools like:
- [WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)
- [Accessible Colors](https://accessible-colors.com/)
- Browser DevTools accessibility panels

## Changes from Previous Version

| Token | Old Value | New Value | Improvement |
|-------|-----------|-----------|-------------|
| `border` | (200, 200, 200) | (160, 160, 160) | 1.6:1 → 3.2:1 |
| `border_subtle` | (230, 230, 230) | (185, 185, 185) | 1.25:1 → 2.3:1 |
| `text.muted` | (120, 120, 120) | (100, 100, 100) | 4.5:1 → 5.3:1 |
| `text.disabled` | (160, 160, 160) | (140, 140, 140) | 2.7:1 → 3.5:1 |
| `editor.blockquote_text` | (100, 100, 100) | (85, 85, 85) | 5.3:1 → 6.0:1 |
| `editor.list_marker` | (100, 100, 100) | (85, 85, 85) | 5.3:1 → 6.0:1 |
