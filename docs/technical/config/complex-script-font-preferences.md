# Complex Script Font Preferences

This document describes the per-script font preference settings for complex scripts (Arabic, Bengali, Devanagari, Thai, Hebrew, Tamil, Georgian, Armenian, Ethiopic, Other Indic, Southeast Asian).

## Overview

Ferrite supports lazy loading of complex script fonts when files containing those scripts are opened. Users can pre-select preferred system fonts for each script family. When a script is detected in text, the user's preferred font (if set) is tried first before falling back to platform-specific defaults.

## Feature Details

### Supported Scripts

| Script ID | Display Name | Default Candidates (platform-specific) |
|-----------|--------------|----------------------------------------|
| arabic | Arabic | Geeza Pro, Al Nile, Segoe UI, Noto Sans Arabic |
| bengali | Bengali | Bangla MN, Nirmala UI, Vrinda, Noto Sans Bengali |
| devanagari | Devanagari | Devanagari MT, Kohinoor Devanagari, Mangal, Noto Sans Devanagari |
| thai | Thai | Thonburi, Leelawadee UI, Noto Sans Thai |
| hebrew | Hebrew | Arial Hebrew, David, Segoe UI, Noto Sans Hebrew |
| tamil | Tamil | Tamil MN, Nirmala UI, Latha, Noto Sans Tamil |
| georgian | Georgian | Segoe UI, Noto Sans Georgian |
| armenian | Armenian | Segoe UI, Noto Sans Armenian |
| ethiopic | Ethiopic | Kefa, Nyala, Noto Sans Ethiopic |
| other_indic | Other Indic | Nirmala UI, Noto Sans Gujarati/Gurmukhi/Kannada/Malayalam/Telugu |
| southeast_asian | Southeast Asian | Myanmar MN, Noto Sans Myanmar/Khmer/Sinhala |

### Settings UI

- **Location**: Settings → Appearance → Additional Scripts (below CJK Regional Preference)
- **Behavior**: Dropdown per script; "Default (System)" uses platform candidates; selecting a font stores it in `complex_script_font_preferences`
- **Persistence**: Stored in `config.json` under `complex_script_font_preferences` (BTreeMap)

### Lazy Loading

Fonts load on-demand when:
1. A file containing complex script characters is opened
2. IME input produces complex script text
3. Fonts are rebuilt (e.g. after settings change, CJK load)

If the user's preferred font is not found on the system, a warning is logged and the default candidates are tried.

## Implementation

### Key Files

| File | Purpose |
|------|---------|
| `src/config/settings.rs` | `complex_script_font_preferences: BTreeMap<String, String>` in Settings |
| `src/ui/settings.rs` | Additional Scripts section in Appearance, dropdown per script |
| `src/fonts.rs` | `load_system_font_with_preference()`, `ComplexScriptFontPreferences`, per-script loaders |

### Data Flow

1. User selects font in Settings → stored in `settings.complex_script_font_preferences`
2. On file open / IME: `load_complex_script_fonts_for_text()` receives preferences
3. `load_complex_script_fonts_selective()` passes preference to each `load_*_font(preference)`
4. `load_system_font_with_preference(preference, candidates)` tries preference first, then candidates

### Backward Compatibility

- `#[serde(default)]` on `complex_script_font_preferences` ensures empty map for existing configs
- When preferences are empty or key missing, default platform candidates are used
