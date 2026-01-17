# Custom Font Selection

This document describes the custom font selection feature in Ferrite, which allows users to choose their preferred fonts for the editor and configure CJK regional glyph preferences.

## Overview

Ferrite supports three types of font configurations:

1. **Built-in Fonts**: Inter (proportional) and JetBrains Mono (monospace) are bundled with the application
2. **Custom System Fonts**: Users can select any font installed on their system
3. **CJK Regional Preferences**: Users can prioritize specific regional variants for CJK characters

## Feature Details

### Built-in Fonts

- **Inter**: A modern, clean proportional font used for general text
- **JetBrains Mono**: A monospace font optimized for code and technical documents

Both fonts include bold and italic variants for proper styling.

### Custom System Font Selection

Users can select any system font from Settings → Appearance → Font.

**How it works:**

1. The system font list is enumerated using `font-kit` crate on application startup
2. Fonts are cached to avoid re-enumeration on each settings panel open
3. When a custom font is selected, it's loaded dynamically and added to the font fallback chain
4. The custom font name is stored in `config.json`

**Limitations:**

- Custom fonts don't have separate bold/italic variants loaded
- The OS may synthesize bold/italic styles depending on the font and platform
- Not all system fonts may be suitable for text editing

### CJK Regional Glyph Preferences

CJK (Chinese, Japanese, Korean) fonts render the same Unicode code points differently based on regional conventions. This setting controls which regional font takes priority.

**Available options:**

| Option | Priority Order | Description |
|--------|----------------|-------------|
| Auto | KR → SC → TC → JP | Use system locale to determine |
| Korean | KR → SC → TC → JP | Prioritize Korean glyph variants |
| Simplified Chinese | SC → TC → KR → JP | Prioritize Simplified Chinese |
| Traditional Chinese | TC → SC → KR → JP | Prioritize Traditional Chinese |
| Japanese | JP → KR → SC → TC | Prioritize Japanese glyph variants |

**System fonts used:**

| Region | macOS | Windows | Linux |
|--------|-------|---------|-------|
| Korean | Apple SD Gothic Neo | Malgun Gothic | Noto Sans CJK KR, NanumGothic |
| Simplified Chinese | PingFang SC | Microsoft YaHei | Noto Sans CJK SC |
| Traditional Chinese | PingFang TC | Microsoft JhengHei | Noto Sans CJK TC |
| Japanese | Hiragino Sans | Yu Gothic, Meiryo | Noto Sans CJK JP |

## Implementation

### Key Files

| File | Purpose |
|------|---------|
| `src/fonts.rs` | Font loading, system font enumeration, and runtime font reload |
| `src/config/settings.rs` | `EditorFont` enum with `Custom` variant, `CjkFontPreference` enum |
| `src/ui/settings.rs` | Settings UI for font picker and CJK preference dropdown |
| `src/app.rs` | Font reload on settings change, initial font setup with saved settings |

### EditorFont Enum

```rust
pub enum EditorFont {
    Inter,           // Built-in proportional font
    JetBrainsMono,   // Built-in monospace font
    Custom(String),  // Custom system font name
}
```

### CjkFontPreference Enum

```rust
pub enum CjkFontPreference {
    Auto,              // Use system locale
    Korean,            // Prioritize Korean glyphs
    SimplifiedChinese, // Prioritize SC glyphs
    TraditionalChinese,// Prioritize TC glyphs
    Japanese,          // Prioritize Japanese glyphs
}
```

### Font Reload

Fonts are reloaded at runtime when settings change:

```rust
// In app.rs, when settings change:
if font_changed {
    fonts::reload_fonts(
        ctx,
        custom_font.as_deref(),
        settings.cjk_font_preference,
    );
}
```

## Configuration

Font settings are stored in `config.json`:

```json
{
  "font_family": "custom",           // or "inter" or "jetbrainsmono"
  "font_family": { "custom": "Arial" }, // when using custom font
  "cjk_font_preference": "auto"      // or "korean", "simplifiedchinese", etc.
}
```

## Testing

- [ ] Test font picker shows system fonts
- [ ] Test selecting a custom font applies to editor
- [ ] Test CJK fonts render correct regional glyphs
- [ ] Test font settings persist across restart
- [ ] Test invalid custom font falls back gracefully

## Related

- GitHub Issue: [#15](https://github.com/OlaProeis/Ferrite/issues/15) - CJK regional glyph preferences
- Dependencies: `font-kit` 0.14.3 for system font enumeration
