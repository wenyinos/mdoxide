# Windows Borderless Window Fixes

## Overview

Improvements to the Windows borderless window implementation: top edge resize, OS-level fullscreen, and window control button redesign.

## Key Files

- `src/ui/window.rs` - Window resize detection logic with title bar exclusion
- `src/app/title_bar.rs` - Title bar rendering, window control buttons
- `src/config/settings.rs` - `ToggleFullscreen` shortcut command

## Implementation Details

### Top Edge Resize

The original implementation disabled north edge resize entirely in the title bar area (35 px) to prevent cursor conflicts with window control buttons. This was overly restrictive since buttons are only on the right side.

**Solution:** Added position-aware resize detection:
- `TITLE_BAR_BUTTON_AREA_WIDTH = 280.0` - defines the drag-exclusion zone on the right
- North edge resize works on the LEFT and CENTER portions of the title bar
- North edge resize is disabled only in the button area (right 280 px)
- NorthWest corner resize is enabled (no buttons on the left)

```rust
// Check if pointer is in the button area (right side of title bar)
let in_button_area = pointer_pos.x > max.x - TITLE_BAR_BUTTON_AREA_WIDTH;

// North edge/corner resize is only disabled when BOTH in title bar AND in button area
let disable_north_resize = in_title_bar && in_button_area;
```

### NorthEast Corner Resize (re-enabled)

Previously, the NE corner was permanently disabled because clicking the Close button (located in that corner) would accidentally start a resize. The fix is geometric: the window control buttons now have a **12 px right margin** (`TITLE_BAR_BUTTON_RIGHT_MARGIN`), which is larger than the corner grab zone (`CORNER_GRAB_SIZE = 10 px`). Any cursor position in the NE corner zone is therefore always button-free.

```rust
/// Right margin gap between window control buttons and the window edge.
/// Must be larger than CORNER_GRAB_SIZE so the NE corner grab zone stays button-free.
const TITLE_BAR_BUTTON_RIGHT_MARGIN: f32 = 12.0;

// NE corner: enabled because the 12 px margin > 10 px corner zone
if (near_right || in_right_zone)
    && pointer_pos.x > max.x - CORNER_GRAB_SIZE
    && pointer_pos.y < min.y + CORNER_GRAB_SIZE
    && (!in_title_bar || pointer_pos.x > max.x - TITLE_BAR_BUTTON_RIGHT_MARGIN)
{
    return Some(ResizeDirection::NorthEast);
}
```

### Window Control Button Redesign

All four window control buttons (Close, Minimize, Maximize/Restore, Fullscreen) were redesigned for a more polished look:

| Property | Before | After |
|----------|--------|-------|
| Size | 46 × 28 px | 36 × 22 px |
| Hover shape | Sharp rectangle | Rounded rect (4 px radius) |
| Close icon | Font glyph `×` | Two diagonal line segments |
| Maximize icon | Plain square | Square with 2 px thick top edge |
| Fullscreen icon | Broken (rendered as ×) | Corner brackets ⌜⌝⌞⌟ |
| Right margin | 4 px | 12 px (enables NE corner resize) |

**Icon rendering** — all icons are drawn directly with `Painter::line_segment` calls, never font glyphs. This ensures pixel-accurate, font-independent rendering:

```rust
// Close button: two diagonal lines forming ×
let d = 5.5_f32;
painter.line_segment([pos2(c.x-d, c.y-d), pos2(c.x+d, c.y+d)], stroke);
painter.line_segment([pos2(c.x+d, c.y-d), pos2(c.x-d, c.y+d)], stroke);

// Fullscreen expand: 4 corner L-brackets, vertex at outer corners, arms pointing in
// TL corner example:
painter.line_segment([pos2(cx-d, cy-d), pos2(cx-d+a, cy-d)], stroke); // → right arm
painter.line_segment([pos2(cx-d, cy-d), pos2(cx-d,   cy-d+a)], stroke); // ↓ down arm
// (TR, BL, BR follow same pattern)
```

**Close button hover** — white icon on red background (`Color32::from_rgb(232, 17, 35)`).  
**Min/Max/Fullscreen hover** — icon unchanged, rounded background fill.

### Fullscreen Toggle

OS-level fullscreen mode (distinct from Zen Mode, which hides UI but keeps window decorations).

**Features:**
- **Keyboard shortcut:** F10 to toggle fullscreen
- **Escape key:** Exits fullscreen mode (highest priority)
- **Title bar button:** Corner-bracket icon (expand ⌜⌝⌞⌟ / compress)
- **Active state highlight:** Button background stays lit when in fullscreen

```rust
ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!is_fullscreen));
```

**Escape Key Priority:**
1. Exit fullscreen if in fullscreen mode
2. Exit multi-cursor mode if active
3. Close find/replace panel

## Dependencies Used

- `egui::ViewportCommand::Fullscreen` - viewport control for fullscreen
- `egui::ViewportCommand::BeginResize` - viewport control for resize operations
- `egui::Painter::line_segment` - direct icon drawing

## Testing

Unit tests in `src/ui/window.rs`:
- `test_title_bar_north_edge_left_side` - North edge works outside button area
- `test_title_bar_north_edge_button_area_blocked` - North edge blocked in button area
- `test_title_bar_northwest_corner` - NorthWest corner works
- `test_title_bar_northeast_corner_enabled` - NorthEast corner now works (12 px margin)
- `test_title_bar_south_corners_always_work` - South corners unaffected
- `test_title_bar_east_west_edges_work_in_title_bar` - Side edges work in title bar

## Related Issues

- [#15](https://github.com/OlaProeis/Ferrite/issues/15) - Windows borderless window issues
