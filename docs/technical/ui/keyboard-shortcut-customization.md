# Keyboard Shortcut Customization

## Overview

Task 25 implements a settings panel for customizing keyboard shortcuts with conflict detection and persistence.

## Features

- **Customizable Shortcuts**: All keyboard shortcuts can be rebound to different key combinations
- **Conflict Detection**: Warns when a new binding conflicts with an existing shortcut
- **Reset Functionality**: Individual shortcuts or all shortcuts can be reset to defaults
- **Search/Filter**: Filter shortcuts by name or category
- **Persistence**: Custom bindings are saved to config.json and restored on startup
- **Categories**: Shortcuts are organized by category (File, Edit, View, Search, Format, etc.)

## Architecture

### Data Structures (src/config/settings.rs)

```rust
/// Modifier keys for keyboard shortcuts
pub struct KeyModifiers {
    pub ctrl: bool,   // Ctrl/Cmd
    pub shift: bool,
    pub alt: bool,    // Alt/Option
}

/// Key codes (serializable wrapper for egui::Key)
pub enum KeyCode {
    A, B, C, ..., Z,
    Num0, Num1, ..., Num9,
    F1, F2, ..., F12,
    Tab, Escape, Space, Enter,
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
    // ... and more
}

/// A keyboard binding with modifiers and key
pub struct KeyBinding {
    pub modifiers: KeyModifiers,
    pub key: KeyCode,
}

/// Command identifier for shortcuts
pub enum ShortcutCommand {
    Save, SaveAs, Open, New, NewTab, CloseTab,
    NextTab, PrevTab, GoToLine, QuickOpen,
    ToggleViewMode, ToggleZenMode, ToggleOutline,
    Find, FindReplace, FindNext, FindPrev,
    FormatBold, FormatItalic, FormatLink,
    // ... etc.
}

/// Keyboard shortcuts configuration
pub struct KeyboardShortcuts {
    bindings: HashMap<ShortcutCommand, KeyBinding>,
}
```

### Settings Panel (src/ui/settings.rs)

New "Keyboard" section added with:
- Search/filter box for finding shortcuts
- Scrollable list grouped by category
- Click-to-capture interface for rebinding
- Conflict warnings
- Reset buttons (individual and all)

### Integration (src/app.rs)

The `handle_keyboard_shortcuts()` method now uses:
```rust
let shortcuts = self.state.settings.keyboard_shortcuts.clone();
// ...
if shortcuts.get(ShortcutCommand::Save).matches(input) {
    return Some(KeyboardAction::Save);
}
```

## Key Binding UI Flow

1. User clicks on a shortcut's current binding button
2. Panel enters "capture mode" for that shortcut
3. Current modifiers and pressed key are displayed in real-time
4. User can Apply or Cancel
5. On Apply, conflict detection runs
6. If conflict found, warning is displayed
7. If no conflict, binding is saved

## Serialization

Custom bindings are stored in config.json:
```json
{
  "keyboard_shortcuts": {
    "bindings": {
      "save": {
        "modifiers": {"ctrl": true, "shift": false, "alt": false},
        "key": "s"
      }
    }
  }
}
```

Only non-default bindings are stored. Commands without custom bindings use their defaults.

## Default Shortcuts

| Command | Default Binding |
|---------|----------------|
| Save | Ctrl+S |
| Save As | Ctrl+Shift+S |
| Open | Ctrl+O |
| New Tab | Ctrl+T |
| Close Tab | Ctrl+W |
| Find | Ctrl+F |
| Find & Replace | Ctrl+H |
| Go to Line | Ctrl+G |
| Bold | Ctrl+B |
| Italic | Ctrl+I |
| Toggle Zen Mode | F11 |
| Open Settings | Ctrl+, |
| ... | ... |

## Localization

Keyboard section strings in `locales/en.yaml`:
- `settings.keyboard.title`
- `settings.keyboard.search_hint`
- `settings.keyboard.reset_all`
- `settings.keyboard.press_key`
- `settings.keyboard.waiting`
- `settings.keyboard.cancel`
- `settings.keyboard.apply`
- `settings.keyboard.conflict_with`
- `settings.keyboard.click_to_change`

## Testing

Comprehensive tests in `src/config/settings.rs`:
- Serialization/deserialization of KeyModifiers, KeyCode, KeyBinding
- Default bindings for all commands
- Custom binding storage and retrieval
- Reset functionality
- Conflict detection
- Backward compatibility with old config files

## Notes

- Escape key is always hardcoded for exiting multi-cursor/closing panels
- Undo/Redo (Ctrl+Z/Y) are handled separately in `consume_undo_redo_keys()`
- Move Line Up/Down (Alt+Up/Down) are handled separately in `consume_move_line_keys()`
