# Linux Cursor Flicker Fix

## Overview

This document describes the fix for cursor flicker near the window control buttons (close, maximize, minimize) on Linux when using Ferrite's custom borderless window frame.

## Problem

On Linux systems with compositing window managers (like Linux Mint, Ubuntu with GNOME, etc.), users experienced cursor flickering when moving the mouse near the close button in the top-right corner. The cursor would rapidly toggle between:
- Default/pointer cursor (for buttons)
- Resize cursor (NorthEast resize zone)

This was caused by the overlap between the window resize detection zones and the title bar button areas.

## Root Cause

The `handle_window_resize` function in `src/ui/window.rs` ran early in each frame to detect mouse proximity to window edges and set appropriate resize cursors. However:

1. The **NorthEast resize zone** (top-right corner, ~10px) overlapped with the **close button** (46x28px at top-right)
2. As the mouse moved within this overlap area, resize detection would toggle on/off based on exact pixel position
3. Multiple cursor icon changes per frame caused visible flickering on Linux (more sensitive than Windows/macOS)

## Solution

Two complementary fixes were implemented:

### 1. Title Bar Exclusion Zone

Added a `TITLE_BAR_EXCLUSION_HEIGHT` constant (35px) that prevents north-edge and north-corner resize detection within the title bar area:

```rust
const TITLE_BAR_EXCLUSION_HEIGHT: f32 = 35.0;
```

The `detect_resize_direction_with_exclusion` function now checks if the pointer is within the title bar before allowing north-edge detection:

```rust
let in_title_bar = pointer_pos.y < min.y + title_bar_height;
let near_top = !in_title_bar && pointer_pos.y < min.y + RESIZE_BORDER_WIDTH;
let in_top_zone = !in_title_bar && pointer_pos.y < min.y + CORNER_GRAB_SIZE;
```

This means:
- **North, NorthEast, NorthWest** resize zones are disabled when cursor is in title bar
- **East and West** edges still work for side resizing (useful for edge cases)
- **South, SouthEast, SouthWest** zones are unaffected

### 2. Cursor Caching

Added cursor state caching to `WindowResizeState` to avoid redundant cursor updates:

```rust
pub struct WindowResizeState {
    current_direction: Option<ResizeDirection>,
    is_resizing: bool,
    last_cursor: Option<CursorIcon>, // NEW: cached cursor
}
```

The cursor is only updated when it actually changes:

```rust
if state.last_cursor != Some(desired_cursor) {
    ctx.set_cursor_icon(desired_cursor);
    state.last_cursor = Some(desired_cursor);
}
```

## Files Modified

- `src/ui/window.rs`:
  - Added `TITLE_BAR_EXCLUSION_HEIGHT` constant
  - Added `last_cursor` field to `WindowResizeState`
  - Created `detect_resize_direction_with_exclusion` function
  - Updated `handle_window_resize` to use exclusion and caching

## Testing

### Manual Test Cases

| Scenario | Expected Behavior |
|----------|-------------------|
| Move mouse near close button (Linux) | Cursor remains stable, no rapid switching |
| Click close button | Window closes as expected |
| Hover over maximize/minimize buttons | Default cursor, buttons highlight on hover |
| Resize from left/right edges | Resize cursors appear, resizing works |
| Resize from bottom/corners | Resize cursors appear, resizing works |
| Drag window from title bar | Window moves smoothly |
| Test on Windows/macOS | No regression, cursor behavior unchanged |

### Platform Notes

- **Linux**: Primary target for this fix. Compositing WMs are more sensitive to cursor changes.
- **Windows/macOS**: Should see no regression. Native window managers handle cursor state more gracefully.

## Trade-offs

1. **North-edge resizing**: Users cannot resize from the top edge while in the title bar area. This is an acceptable trade-off because:
   - The title bar is primarily for window controls and dragging
   - Side and bottom resizing still work
   - Corners (except north corners in title bar) still work

2. **Title bar height assumption**: The 35px exclusion height is hardcoded based on the current title bar design (28px height + 5px padding + margin). If title bar height changes, this constant should be updated.

## Related Issues

- Original behavior caused cursor flicker on Linux Mint and similar distributions
- No GitHub issue was filed for this bug (discovered during development)
