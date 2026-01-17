# Zen Mode (Distraction-Free Writing)

## Overview

Zen Mode provides a distraction-free writing experience by hiding all non-essential UI elements and centering the text content. This feature is ideal for focused writing sessions where visual distractions should be minimized.

## Features

### Chrome Hidden in Zen Mode

When Zen Mode is enabled, the following UI elements are hidden:
- **Ribbon toolbar** - All formatting buttons and controls
- **Tab bar** - Document tabs and new tab button
- **Status bar** - File path, cursor position, encoding info
- **File tree panel** - Workspace file browser (when in workspace mode)
- **Outline panel** - Document structure navigation
- **Line numbers** - Hidden for cleaner appearance

### Retained in Zen Mode

- **Title bar** - Window controls (minimize, maximize, close) remain accessible
- **Editor content** - The main editing area remains fully functional
- **Keyboard shortcuts** - All editing shortcuts continue to work

### Centered Text Column

In Zen Mode, the text content is horizontally centered with a maximum column width:
- Default maximum width: 80 characters
- Configurable via `zen_max_column_width` setting (50-120 chars)
- Editor background fills the full window
- Text content centered within that space
- Adaptive margins that adjust to window size

## Activation

### Keyboard Shortcut

- **F11** - Toggle Zen Mode on/off

### Ribbon Button

The View group in the ribbon includes a Zen Mode toggle button:
- 🧘 (yoga/meditation icon) when Zen Mode is enabled
- 🔲 (square icon) when Zen Mode is disabled

### Toast Notification

When toggling Zen Mode, a brief toast message confirms the state change:
- "Zen Mode enabled"
- "Zen Mode disabled"

## Configuration

### Settings (`src/config/settings.rs`)

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `zen_max_column_width` | f32 | 80.0 | Maximum text column width in characters |
| `zen_mode_enabled` | bool | false | Session state for persistence |

Validation constants:
- `MIN_ZEN_COLUMN_WIDTH`: 50.0
- `MAX_ZEN_COLUMN_WIDTH`: 120.0

### Session Persistence

Zen Mode state is automatically persisted:
- Saved when the application closes
- Restored on next launch
- Included in crash recovery snapshots

## Implementation Details

### State Management

| Location | Field | Purpose |
|----------|-------|---------|
| `UiState` | `zen_mode: bool` | Runtime state flag |
| `SessionState` | `zen_mode: bool` | Persisted session state |
| `Settings` | `zen_max_column_width: f32` | User preference |
| `AppState` | `toggle_zen_mode()` | State toggle method |
| `AppState` | `is_zen_mode()` | State query method |

### Key Files

| File | Purpose |
|------|---------|
| `src/state.rs` | `UiState.zen_mode` flag, toggle/query methods, session capture/restore |
| `src/config/settings.rs` | `zen_max_column_width` setting with validation |
| `src/config/session.rs` | `zen_mode` field for session persistence |
| `src/app.rs` | Layout conditional hiding, keyboard shortcut (F11) |
| `src/editor/widget.rs` | `zen_mode()` builder method, centered text rendering |
| `src/ui/ribbon.rs` | `RibbonAction::ToggleZenMode`, toggle button UI |

### Editor Widget Integration

The `EditorWidget` supports Zen Mode via a builder method:

```rust
EditorWidget::new(tab)
    .font_size(font_size)
    .zen_mode(true, 80.0)  // Enable with 80-char column width
    .show(ui);
```

When enabled:
1. Calculates left margin based on available width and max column width
2. Adds horizontal spacing before the text content
3. Constrains `TextEdit` desired width to max column width

### Column Width Calculation

```rust
let char_width = font_size * 0.6;  // Approximate character width
let max_content_width = char_width * zen_max_column_width;
let zen_margin = (available_width - max_content_width) / 2.0;
```

## Test Strategy

1. **Toggle Test**: Press F11 to toggle Zen Mode
   - Verify all chrome hides/shows correctly
   - Verify toast message appears
   - Verify editor remains functional

2. **Centered Text**: With Zen Mode enabled
   - Text content should be horizontally centered
   - Margins should be equal on left and right
   - Text should wrap within the column width

3. **Window Resize**: Resize window while in Zen Mode
   - Margins should adapt dynamically
   - On narrow windows, margins reduce gracefully

4. **Session Persistence**: Enable Zen Mode, close and reopen app
   - Zen Mode should be restored on launch

5. **Keyboard Navigation**: All shortcuts should work in Zen Mode
   - Ctrl+Z/Y (undo/redo)
   - Ctrl+S (save)
   - Ctrl+F (find)
   - F11 (exit Zen Mode)

## Future Enhancements

- **Typewriter scrolling** - Keep active line vertically centered
- **Smooth transitions** - Animated chrome hide/show
- **Settings UI** - In-app adjustment of column width
- **Theme integration** - Subtle background dimming options
- **Rendered mode support** - Zen Mode for WYSIWYG view
