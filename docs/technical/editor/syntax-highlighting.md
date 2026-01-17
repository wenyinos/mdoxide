# Syntax Highlighting

Full-file syntax highlighting for source code files in Ferrite's raw editor mode.

## Overview

Ferrite now supports syntax highlighting for source code files (Rust, Python, JavaScript, etc.) in the raw editor mode. This feature uses the existing `syntect` library that was previously only used for code blocks in markdown preview.

## Supported Languages

The following file extensions are supported:

| Category | Extensions |
|----------|------------|
| **Systems** | `.rs` (Rust), `.c`, `.cpp`, `.h`, `.hpp`, `.go`, `.swift` |
| **Web** | `.js`, `.ts`, `.tsx`, `.jsx`, `.html`, `.css`, `.scss`, `.sass`, `.less` |
| **Scripting** | `.py`, `.rb`, `.php`, `.lua`, `.pl`, `.sh`, `.bash`, `.ps1` |
| **JVM** | `.java`, `.kt`, `.scala`, `.clj` |
| **Data** | `.json`, `.yaml`, `.yml`, `.toml`, `.xml` |
| **Functional** | `.hs`, `.ex`, `.exs`, `.erl` |
| **Other** | `.sql`, `.vim`, `.diff`, `.ini`, `.cmake`, `.dockerfile`, `.makefile` |

## Usage

### Enabling/Disabling

1. Open Settings (Ctrl+,)
2. Navigate to the **Editor** section
3. Toggle **Syntax Highlighting** checkbox

The setting is enabled by default.

### How It Works

- When you open a source code file, Ferrite detects the language from the file extension
- If syntax highlighting is enabled and the language is recognized, the editor applies colored text formatting
- Syntax colors automatically adapt to light/dark theme (using `base16-ocean.dark` for dark mode and `InspiredGitHub` for light mode)
- Markdown files are handled separately by the rendered/WYSIWYG editor, so syntax highlighting only affects other file types in raw mode

## Implementation Details

### Files Modified

| File | Changes |
|------|---------|
| `src/config/settings.rs` | Added `syntax_highlighting_enabled: bool` setting |
| `src/ui/settings.rs` | Added checkbox in Editor section |
| `src/markdown/syntax.rs` | Added `language_from_path()` and `can_highlight_file()` helpers |
| `src/editor/widget.rs` | Modified layouter to use syntax highlighting |
| `src/app.rs` | Passed syntax highlighting config to EditorWidget |

### Key Components

1. **Language Detection**: `language_from_path()` in `syntax.rs` maps file extensions to syntect language identifiers

2. **Highlighter Integration**: The `EditorWidget` layouter creates a colored `LayoutJob` using `highlight_code()` when a language is detected

3. **Theme Selection**: Dark mode uses `base16-ocean.dark`, light mode uses `InspiredGitHub`

### Performance Considerations

- **Caching**: Highlighted output is cached in egui's memory and only regenerated when content changes (detected via content hash). This makes static viewing essentially zero-cost.
- The syntect `SyntaxSet` and `ThemeSet` are loaded once globally and reused
- Cache is keyed by (editor_id, content_hash, language, dark_mode) for correctness

## Testing

1. Open a `.rs`, `.py`, `.js`, or `.json` file
2. Verify syntax colors appear (keywords, strings, comments should have distinct colors)
3. Toggle setting off in Settings → Editor → Syntax Highlighting
4. Colors should disappear (plain text)
5. Switch between light/dark theme - colors should adapt

## Known Limitations

- Bold/italic font styles from syntect are not applied (egui TextFormat limitation)
- No per-file-type theme customization (uses global dark/light themes)
- Initial highlighting of very large files (>10,000 lines) may take a moment on first open
