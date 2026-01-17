# Windows Borderless Window Fixes

## Overview

Improvements to the Windows borderless window implementation, adding top edge resize capability and OS-level fullscreen toggle.

## Key Files

- `src/ui/window.rs` - Window resize detection logic with title bar exclusion
- `src/app.rs` - Title bar rendering, fullscreen button, keyboard shortcut handling
- `src/config/settings.rs` - `ToggleFullscreen` shortcut command

## Implementation Details

### Top Edge Resize

The original implementation disabled north edge resize entirely in the title bar area (35px) to prevent cursor conflicts with window control buttons. This was overly restrictive since buttons are only on the right side.

**Solution:** Added position-aware resize detection:
- `TITLE_BAR_BUTTON_AREA_WIDTH = 280.0` - defines the button area on the right
- North edge resize works on the LEFT and CENTER portions of the title bar
- North edge resize is disabled only in the button area (right 280px)
- NorthWest corner resize is enabled (no buttons on the left)
- NorthEast corner remains disabled (buttons are there)

```rust
// Check if pointer is in the button area (right side of title bar)
let in_button_area = pointer_pos.x > max.x - TITLE_BAR_BUTTON_AREA_WIDTH;

// North edge/corner resize is only disabled when BOTH in title bar AND in button area
let disable_north_resize = in_title_bar && in_button_area;
```

### Fullscreen Toggle

Added OS-level fullscreen mode (different from Zen Mode which hides UI elements but keeps window decorations).

**Features:**
- **Keyboard shortcut:** F10 to toggle fullscreen
- **Escape key:** Exits fullscreen mode (priority over other Escape behaviors)
- **Title bar button:** Visual icon next to minimize button with expand/contract arrows
- **Toast notifications:** Feedback when entering/exiting fullscreen

```rust
// Toggle fullscreen via ViewportCommand
ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!is_fullscreen));
```

**Escape Key Priority:**
1. Exit fullscreen if in fullscreen mode
2. Exit multi-cursor mode if active
3. Close find/replace panel

## Dependencies Used

- `egui::ViewportCommand::Fullscreen` - egui viewport control for fullscreen
- `egui::ViewportCommand::BeginResize` - egui viewport control for resize operations

## Testing

Unit tests added for title bar exclusion behavior:
- `test_title_bar_north_edge_left_side` - North edge works outside button area
- `test_title_bar_north_edge_button_area_blocked` - North edge blocked in button area
- `test_title_bar_northwest_corner` - NorthWest corner works
- `test_title_bar_northeast_corner_blocked` - NorthEast corner blocked
- `test_title_bar_south_corners_always_work` - South corners unaffected
- `test_title_bar_east_west_edges_work_in_title_bar` - Side edges work in title bar

## Related Issues

- [#15](https://github.com/OlaProeis/Ferrite/issues/15) - Windows borderless window issues
