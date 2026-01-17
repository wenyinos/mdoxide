# Snippets/Abbreviation System

This document describes the snippets/abbreviation system implemented in Ferrite v0.2.5 (Task 23).

## Overview

The snippets system provides user-defined text expansions with built-in date/time snippets. Users can type a trigger word followed by space or tab, and it expands to the full text.

## Built-in Snippets

Ferrite includes four built-in date/time snippets:

| Trigger | Expansion | Example |
|---------|-----------|---------|
| `;date` | Current date (YYYY-MM-DD) | `2026-01-16` |
| `;time` | Current time (HH:MM) | `14:30` |
| `;datetime` | Current date and time | `2026-01-16 14:30` |
| `;now` | ISO 8601 timestamp | `2026-01-16T14:30:00+00:00` |

## Custom Snippets

Custom snippets can be defined in a JSON configuration file located at the platform-specific config directory:

- **Linux**: `~/.config/ferrite/snippets.json`
- **macOS**: `~/Library/Application Support/ferrite/snippets.json`
- **Windows**: `%APPDATA%\ferrite\snippets.json`

### Configuration File Format

```json
{
  "enabled": true,
  "snippets": {
    "sig": "Best regards,\nJohn Doe",
    "addr": "123 Main Street\nAnytown, USA",
    "mtg": "Meeting Notes\n\n**Date**: ;date\n**Attendees**:\n- "
  }
}
```

### Multi-line Expansions

Snippets can contain newlines using `\n`:

```json
{
  "snippets": {
    "header": "# Document Title\n\n**Author**: Name\n**Date**: ;date\n\n---\n"
  }
}
```

## How It Works

### Trigger Detection

1. User types a trigger word (e.g., `;date`)
2. User types a space or tab character
3. The system looks backwards from the cursor to find a word boundary (whitespace)
4. If the word matches a known trigger (built-in or custom), it's replaced with the expansion
5. The space/tab that triggered the expansion is preserved after the expansion

### Expansion Priority

1. Built-in snippets are checked first
2. Custom snippets are checked second
3. If no match is found, no expansion occurs

## Architecture

### Key Files

- `src/config/snippets.rs` - Core snippet configuration and trigger detection
- `src/config/mod.rs` - Module exports
- `src/app.rs` - Integration with the main application loop
- `src/ui/settings.rs` - Settings UI for enabling/disabling snippets

### Data Structures

```rust
// Snippet configuration
pub struct SnippetConfig {
    pub enabled: bool,
    pub snippets: HashMap<String, String>,
}

// Snippet manager (handles loading/saving)
pub struct SnippetManager {
    pub config: SnippetConfig,
    config_path: PathBuf,
    last_modified: Option<SystemTime>,
}

// Match result for expansion
pub struct SnippetMatch {
    pub start: usize,      // Start byte position
    pub end: usize,        // End byte position
    pub trigger: String,   // The trigger text
    pub expansion: String, // The expansion text
}
```

### Key Functions

- `SnippetConfig::builtin_expansion(trigger)` - Get expansion for built-in triggers
- `SnippetConfig::get_expansion(trigger)` - Get expansion for any trigger
- `find_trigger_at_cursor(text, cursor_pos, manager)` - Find trigger word at cursor
- `apply_snippet(text, snippet_match)` - Apply expansion to text
- `FerriteApp::try_expand_snippet(tab_index)` - Main expansion entry point

## Settings

The snippets feature can be enabled/disabled in Settings > Editor > Snippets:

- **Enable Snippet Expansion**: Toggle to enable/disable the feature
- When enabled, shows a list of built-in snippets for reference

## Implementation Notes

### Timing

Snippet expansion is checked at the end of each frame after the editor has processed user input. This ensures:
1. The space/tab character is already in the content
2. The cursor position is updated
3. We can safely modify content without conflicting with egui's TextEdit

### Undo/Redo

Snippet expansions are recorded in the undo stack, allowing users to undo an expansion if needed. The `record_edit` function is called with the old content before the expansion.

### Content Version

After expansion, `increment_content_version()` is called to signal to the editor widget that it needs to re-read the content. This ensures the UI reflects the expanded text immediately.

## Limitations

1. **No cursor placement markers**: The cursor is always placed at the end of the expansion. Future versions may support `$0` cursor markers.

2. **No variables/placeholders**: Current implementation only supports static text. Future versions may support variables like `$1`, `$2` for tabstops.

3. **No file watching**: Changes to `snippets.json` are not auto-reloaded. The application must be restarted to pick up changes (or the feature could be enhanced with `check_and_reload()`).

4. **No snippet editor UI**: Custom snippets must be edited manually in the JSON file. Future versions may add a UI for managing snippets.

## Dependencies

- `chrono` - For date/time formatting in built-in snippets
- `dirs` - For platform-specific config directory paths (already in project)
- `serde_json` - For config file parsing (already in project)
