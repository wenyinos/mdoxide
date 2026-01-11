# Default View Mode Configuration

## Overview

Ferrite v0.2.2 introduces a configurable default view mode, allowing users to choose which view mode (Raw, Rendered, or Split) new tabs open in. This addresses [GitHub Issue #3](https://github.com/OlaProeis/Ferrite/issues/3).

## Key Files

| File | Purpose |
|------|---------|
| `src/config/settings.rs` | `ViewMode` enum enhancements and `default_view_mode` field in Settings |
| `src/state.rs` | Tab constructors modified to accept default view mode |
| `src/ui/settings.rs` | Settings UI dropdown for default_view_mode selection |

## Implementation Details

### ViewMode Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ViewMode {
    #[default]
    Raw,      // Plain markdown text editing
    Rendered, // WYSIWYG rendered editing
    Split,    // Raw editor + rendered preview side by side
}
```

### New Helper Methods

- `all()` - Returns all available view modes
- `description()` - Returns a description of each mode

### Settings Field

```rust
pub struct Settings {
    // ...
    /// Default view mode for new tabs.
    /// Controls whether new tabs open in Raw, Rendered, or Split view.
    /// Existing tabs retain their stored view mode.
    pub default_view_mode: ViewMode,
}
```

### Tab Construction Flow

When creating a new tab, the default view mode is applied:

```rust
// AppState::new_tab()
pub fn new_tab(&mut self) -> usize {
    let auto_save_default = self.settings.auto_save_enabled_default;
    let default_view_mode = self.settings.default_view_mode;
    let tab = Tab::new_with_settings(self.next_tab_id, auto_save_default, default_view_mode);
    // ...
}
```

### Important Behavior

- **New tabs** open with the configured default view mode
- **Existing tabs** retain their saved view mode (from session restore)
- **Individual tab changes** do NOT affect the global setting

## Usage

### Settings UI

1. Open Settings (⚙ button or Ctrl+,)
2. Navigate to "Appearance" section
3. Find "Default View Mode" at the bottom
4. Select: Raw, Rendered, or Split

### Config File

In `config.json`:

```json
{
  "default_view_mode": "split"
}
```

Valid values: `raw`, `rendered`, `split`

## Backward Compatibility

- Uses `#[serde(default)]` so existing config files without `default_view_mode` default to `raw`
- Existing tabs with saved view modes are unaffected
- Session restore continues to work as before (tabs restore with their saved view mode)

## Tests

Run default view mode tests:

```bash
cargo test settings::tests::test_view_mode
cargo test settings::tests::test_settings_default_view_mode
```

Tests cover:
- Default value (`Raw`)
- Serialization/deserialization
- All enum variants with `ViewMode::all()`
- Backward compatibility with old config files
- Description method for all modes

## Test Strategy

| Scenario | Expected Behavior |
|----------|-------------------|
| Delete config.json, start app | New tabs default to Raw |
| Set default_view_mode to Split in settings | New tabs open in Split view |
| Change one tab to Rendered manually | Only that tab changes, global setting unchanged |
| Session restore | Existing tabs retain their saved view modes |

## Related

- [Settings & Config](./settings-config.md) - Settings system overview
- [GitHub Issue #3](https://github.com/OlaProeis/Ferrite/issues/3) - Original feature request
