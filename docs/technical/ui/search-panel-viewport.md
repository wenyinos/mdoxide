# Search Panel Viewport Constraints

## Overview

The Search in Files panel (Ctrl+Shift+F) implements robust viewport constraints to ensure it always renders fully within the main application window. This prevents clipping issues on small screens, high DPI displays, or when the window is resized to be smaller than the panel.

## Problem Statement

Before this fix, the Search panel used a center anchor but no viewport constraints, which caused:
- Panel clipping when window height was smaller than panel height
- Panel extending beyond screen edges on small resolutions
- Inconsistent behavior across different DPI settings
- Panel not repositioning when Split View reduced available space

## Solution Architecture

### Viewport Constraint Utilities

New helper functions in `src/ui/window.rs`:

```rust
/// Constraints for a floating panel or window.
pub struct PanelConstraints {
    pub min_width: f32,   // Minimum usable width
    pub max_width: f32,   // Maximum allowed width
    pub min_height: f32,  // Minimum usable height
    pub max_height: f32,  // Maximum allowed height
    pub margin: f32,      // Margin from viewport edges
}

/// Result of constraining a panel to viewport bounds.
pub struct ConstrainedPanel {
    pub pos: Pos2,             // Constrained position
    pub size: Vec2,            // Constrained size
    pub was_resized: bool,     // Whether size was adjusted
    pub was_repositioned: bool, // Whether position was adjusted
}

/// Core constraint function
pub fn constrain_rect_to_viewport(
    desired_rect: Rect,
    viewport: Rect,
    constraints: &PanelConstraints,
) -> ConstrainedPanel;

/// Convenience function for centering panels
pub fn center_panel_in_viewport(
    viewport: Rect,
    panel_size: Vec2,
    constraints: &PanelConstraints,
) -> ConstrainedPanel;

/// Pre-configured constraints for the Search panel
pub fn search_panel_constraints() -> PanelConstraints;
```

### Search Panel Constraints

Default constraints for the Search in Files panel:
- **Min Width:** 350px - Enough for search field + buttons
- **Min Height:** 200px - Show search field + a few results
- **Max Width:** 700px - Don't get too wide
- **Max Height:** 600px - Don't take entire screen
- **Margin:** 16px - Keep padding from edges

### Panel State Management

The `SearchPanel` struct now tracks:
- `panel_size: Vec2` - Last known panel size
- `panel_pos: Option<Pos2>` - Last known position (None = recalculate)
- `constraints: PanelConstraints` - Size/position limits
- `last_viewport_size: Vec2` - Detect viewport changes

### Viewport Change Detection

On each frame, the panel checks if the viewport size changed:
```rust
let viewport = ctx.screen_rect();
let viewport_size = viewport.size();

// Detect resize (with 1px tolerance for floating point)
let viewport_changed = (viewport_size - self.last_viewport_size).length() > 1.0;
if viewport_changed {
    self.last_viewport_size = viewport_size;
    self.panel_pos = None; // Force recalculation
}
```

### Position/Size Calculation

When showing the panel:
1. If `panel_pos` is `None`, center in viewport
2. If position exists, validate it still fits
3. Apply min/max size constraints
4. Clamp position to keep panel within available area
5. Update stored position/size

```rust
let constrained = if let Some(pos) = self.panel_pos {
    // Validate existing position
    let desired_rect = Rect::from_min_size(pos, self.panel_size);
    constrain_rect_to_viewport(desired_rect, viewport, &self.constraints)
} else {
    // Center on first show or after viewport change
    center_panel_in_viewport(viewport, self.panel_size, &self.constraints)
};
```

### Window Integration

The egui Window is configured with dynamic constraints:
```rust
window = window
    .min_width(constraints.min_width)
    .min_height(constraints.min_height)
    .max_width((viewport.width() - margin * 2.0).max(min_width))
    .max_height((viewport.height() - margin * 2.0).max(min_height));
```

This ensures:
- User can't resize smaller than minimum usable size
- User can't resize larger than available viewport
- Constraints adapt to current window size

## Behavior

### On Panel Open
1. Calculates centered position within viewport
2. Applies size constraints
3. Ensures panel fits with margin from edges

### On Window Resize
1. Detects viewport size change
2. Resets stored position to force recalculation
3. Next frame re-centers panel in new viewport
4. Applies new size constraints

### On User Drag/Resize
1. egui handles drag/resize within Window
2. Window respects configured min/max constraints
3. Position is clamped to valid area

### With Split View
- Panel uses the full window viewport (not just editor pane)
- Viewport changes when Split View is toggled
- Panel automatically repositions to stay visible

## Internal Scrolling

When the viewport is too small to fit minimum panel size, the panel:
1. Clamps to available space (may be smaller than minimum)
2. Uses internal `ScrollArea` for content
3. Results list scrolls when content exceeds visible area

## Testing

### Unit Tests

Located in `src/ui/window.rs`:
- `test_constrain_rect_centered` - Panel fits easily
- `test_constrain_rect_too_large` - Panel shrunk to fit
- `test_constrain_rect_off_screen_right` - Repositioned from right edge
- `test_constrain_rect_off_screen_bottom` - Repositioned from bottom
- `test_constrain_rect_respects_min_size` - Small panel enlarged
- `test_center_panel_in_viewport` - Centering works correctly
- `test_search_panel_constraints` - Default values are valid

Located in `src/ui/search.rs`:
- `test_search_panel_size_constraints` - Size clamping works

### Manual Testing

1. **Basic visibility:** Open Search panel at normal window size
2. **Vertical resize:** Shrink window height, verify panel repositions
3. **Horizontal resize:** Shrink window width, verify panel resizes
4. **DPI changes:** Test at 100%, 150%, 200% scaling
5. **Split View:** Enable split view, verify panel stays visible
6. **Multi-monitor:** Move window between different resolution monitors
7. **Keyboard:** Tab through controls, verify all focusable

## Files Changed

- `src/ui/window.rs` - Added constraint utilities and tests
- `src/ui/search.rs` - Updated to use viewport constraints
- `src/ui/mod.rs` - Export new public functions

## Future Improvements

1. **Persistence:** Save panel size/position to settings
2. **Dock modes:** Allow docking to side of window
3. **Animation:** Smooth transition when repositioning
4. **Other panels:** Apply same pattern to About, Settings, etc.
