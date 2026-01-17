# Auto-Save Feature

## Overview

Ferrite implements a configurable auto-save feature that saves documents after a configurable idle delay. The feature uses a **temp-file based strategy** to prevent data loss without overwriting the main file prematurely.

## Key Features

1. **Per-document toggle** - Each tab can have auto-save enabled/disabled independently via toolbar button
2. **Settings-based defaults** - New documents inherit auto-save state from settings
3. **Idle-based triggering** - Saves after configurable idle delay (not interval-based)
4. **Temp file strategy** - Writes to `.autosave/` in config directory, not the main file
5. **Recovery on open** - Detects newer temp files and offers restore/discard
6. **Atomic writes** - Uses temp file + rename for crash safety

## Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `auto_save_enabled_default` | bool | false | Default auto-save state for new tabs |
| `auto_save_delay_ms` | u32 | 15000 | Delay in ms before auto-save triggers |

Settings are configurable in the Settings panel under "Files" section.

## Architecture

### State Management

```rust
// Tab struct fields for auto-save
pub struct Tab {
    pub auto_save_enabled: bool,              // Per-tab toggle
    pub last_edit_time: Option<Instant>,      // For idle detection
    last_auto_save_content_hash: Option<u64>, // Change detection
}
```

### Key Methods

- `Tab::toggle_auto_save()` - Toggle auto-save for this tab
- `Tab::mark_content_edited()` - Called when content changes (updates last_edit_time)
- `Tab::should_auto_save(delay_ms)` - Check if auto-save should trigger
- `Tab::mark_auto_saved()` - Mark content as auto-saved (updates hash)

### Temp File Storage

Auto-save files are stored in:
```
~/.config/ferrite/autosave/
├── filename_<hash>.md.autosave     # For saved files
└── untitled_<tab_id>.md.autosave   # For unsaved documents
```

Each auto-save file contains:
1. JSON metadata line (tab ID, original path, timestamp, content hash)
2. Blank line separator
3. Document content

### Auto-Save Flow

1. **Content edited** → `set_content()` calls `mark_content_edited()`
2. **Main loop** → `process_auto_saves()` checks each tab
3. **If should_auto_save** → Write to temp file, `mark_auto_saved()`
4. **Manual save** → `cleanup_auto_save_for_tab()` deletes temp file

### Recovery Flow

1. **File opened** → `check_auto_save_recovery()` checks for newer temp file
2. **If found** → Store `AutoSaveRecoveryInfo` for dialog
3. **Update loop** → `show_auto_save_recovery_dialog()` renders modal
4. **User chooses** → Restore content or discard temp file

## UI Components

### Toolbar Button

Located in the File group of the ribbon:
- **Icon**: ⏱ (enabled) / ⏸ (disabled)
- **Color**: Green background when active
- **Action**: `RibbonAction::ToggleAutoSave`

### Settings Panel

Under "Files" section:
- Checkbox: "Enable Auto-Save by Default"
- Slider: Auto-save delay (5-300 seconds)
- Presets: 15s, 30s, 1m buttons

### Recovery Dialog

Modal dialog shown when opening file with newer auto-save:
- Shows file path and time since auto-save
- Buttons: "✅ Restore" and "🗑 Discard"

## Cleanup

Auto-save temp files are cleaned up:
- After manual save (via Save or Save As)
- After user discards recovery
- After user restores from recovery

## Related Files

- `src/config/settings.rs` - Settings definitions
- `src/config/session.rs` - Temp file functions
- `src/state.rs` - Tab auto-save state
- `src/app.rs` - Auto-save processing, recovery dialog
- `src/ui/ribbon.rs` - Toolbar toggle button

## Testing

### Manual Test Cases

1. **Enable auto-save, edit, wait > delay** → Verify temp file created in config dir
2. **Manual save** → Verify temp file cleared
3. **Crash simulation** → Kill process, restart, open file → Verify recovery prompt
4. **Per-editor toggle** → Verify independent auto-save state per tab
5. **Large file** → Verify no UI lag during auto-save (atomic write)

### Verify Temp File Location

Windows: `%APPDATA%\ferrite\autosave\`
Linux: `~/.config/ferrite/autosave/`
macOS: `~/Library/Application Support/ferrite/autosave/`
