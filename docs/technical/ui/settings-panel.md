# Settings Panel

The Settings Panel provides an interface for configuring application preferences with live preview. All changes are applied immediately and saved automatically.

Since v0.2.7, the Settings Panel opens as a **special tab** in the main tab bar (like Cursor/VS Code), replacing the previous modal window approach. See [Special Tabs](./special-tabs.md) for the underlying system.

## Features

- **Tab-based UI** - Opens as a tab alongside documents, using full editor space
- **Single instance** - Only one Settings tab can exist at a time; re-opening focuses it
- **Section navigation** - Sidebar with Appearance, Editor, Files, Keyboard, Terminal, About
- **Live preview** - Changes apply immediately without requiring a save action
- **Auto-save** - Settings are persisted automatically when modified
- **Reset to defaults** - One-click option to restore all settings to defaults

## Access Methods

1. **Title bar**: Click the gear icon (⚙) in the title bar
2. Settings tab appears in the tab bar and can be closed like any tab

## Sections

### Appearance

Configure visual preferences:

| Setting | Description | Range/Options |
|---------|-------------|---------------|
| Theme | Color scheme | Light, Dark, System |
| Font Size | Editor text size | 8-72px (Small/Medium/Large presets) |

### Editor

Configure editing behavior:

| Setting | Description | Default |
|---------|-------------|---------|
| Word Wrap | Wrap long lines | Enabled |
| Show Line Numbers | Display line numbers | Enabled |
| Use Spaces | Spaces instead of tabs | Enabled |
| Tab Size | Indentation width | 4 spaces (2-8 range) |

### Files

Configure file handling:

| Setting | Description | Default |
|---------|-------------|---------|
| Auto-Save | Save files automatically | Disabled |
| Auto-Save Interval | Seconds between saves | 60 (5-300 range) |
| Recent Files | Number to remember | 10 (0-20 range) |
| Clear Recent Files | Remove all recent entries | Button |

## Architecture

### Components

```
src/ui/settings.rs
├── SettingsSection    - Enum for navigation tabs
├── SettingsPanelOutput - Result of showing the panel
└── SettingsPanel      - Main panel component
```

### State Flow

```
User Action
    ↓
SettingsPanel::show()
    ↓
Modifies &mut Settings directly
    ↓
Returns SettingsPanelOutput { changed, close_requested, reset_requested }
    ↓
App handles:
  - changed → Apply theme, mark dirty
  - reset_requested → Restore defaults, apply theme
  - close_requested → Hide panel
```

### Integration Points

1. **AppState::open_settings_tab()** - Opens or focuses the Settings special tab
2. **TabKind::Special(SpecialTabKind::Settings)** - Tab kind identifier
3. **SettingsPanel::show_inline()** - Renders settings directly in a `Ui` (tab content area)
4. **AppState.settings** - Direct mutation for live preview
5. **ThemeManager** - Theme changes applied immediately via `set_theme()` and `apply()`
6. **mark_settings_dirty()** - Triggers persistence on next save interval

## Implementation Details

### Tab-Based Rendering (v0.2.7+)

Settings is rendered as a special tab via `render_special_tab_content()` in `central_panel.rs`:

```rust
// In central_panel.rs
if let TabKind::Special(special_kind) = active_tab_kind {
    self.render_special_tab_content(ui, special_kind);
} else {
    // Normal editor rendering...
}
```

The `show_inline()` method renders a sidebar + content layout that fills the entire tab area.

### Live Preview

Changes modify settings directly, enabling immediate visual feedback:

```rust
if ui.selectable_value(&mut settings.theme, Theme::Dark, "Dark").changed() {
    changed = true;  // Signal for theme application
}
```

The app then applies theme changes:

```rust
if output.changed {
    self.theme_manager.set_theme(self.state.settings.theme);
    self.theme_manager.apply(ctx);
    self.state.mark_settings_dirty();
}
```

### Persistence

Settings are marked dirty on change and saved automatically via the existing config persistence system:

1. `mark_settings_dirty()` sets the dirty flag
2. `eframe::App::save()` calls `save_settings_if_dirty()`
3. Settings serialize to `~/.config/sleek-markdown-editor/config.json`

## Testing

Unit tests cover:
- Panel initialization (`test_settings_panel_new`, `test_settings_panel_default`)
- Section enumerations (`test_settings_section_label`, `test_settings_section_icon`)
- Output struct defaults (`test_settings_panel_output_default`)

The panel integrates with existing settings tests for validation and serialization.

## Future Enhancements

Potential additions for future iterations:
- Accent color picker for UI customization
- Default save location preference
- Font family selection
- Keyboard shortcut customization
- Import/export settings
