# ViewState - Viewport Tracking for Virtual Scrolling

The `ViewState` module provides viewport tracking and visible line range calculation for efficient rendering in the custom editor widget. It enables virtual scrolling by only rendering lines that are actually visible (plus overscan for smooth scrolling).

## Overview

`ViewState` is part of Phase 1 of the FerriteEditor custom widget, which focuses on non-wrapping horizontal scroll. It tracks:

- **Viewport dimensions** - Height of the visible area in pixels
- **Scroll position** - First visible line and sub-line offset
- **Line height** - Pixels per line (from font metrics)
- **Horizontal scroll** - Offset for long lines without wrapping

## Usage

```rust
use crate::editor::ViewState;

// Create a new ViewState
let mut view = ViewState::new();

// Configure viewport (called on window resize)
view.update_viewport(600.0);  // 600px tall viewport
view.set_line_height(18.0);   // 18px per line (from galley metrics)

// Get visible line range for rendering
let total_lines = buffer.line_count();
let (start, end) = view.get_visible_line_range(total_lines);

// Render only lines start..end for efficiency
for line_idx in start..end {
    let line_content = buffer.line(line_idx);
    // render line...
}
```

## Key Methods

### Viewport Management

| Method | Description |
|--------|-------------|
| `new()` | Create with default values |
| `update_viewport(height)` | Set viewport height in pixels |
| `set_line_height(height)` | Set line height from font metrics |

### Line Range Calculation

| Method | Description |
|--------|-------------|
| `get_visible_line_range(total_lines)` | Returns `(start, end)` with 5-line overscan |
| `is_line_visible(line, total_lines)` | Check if line is in visible area |

### Scrolling

| Method | Description |
|--------|-------------|
| `scroll_to_line(line)` | Jump to show line at viewport top |
| `scroll_to_center_line(line, total_lines)` | Center a line in viewport |
| `scroll_by(delta_y, total_lines)` | Smooth pixel-based scrolling (word wrap aware) |
| `clamp_scroll_position(total_lines)` | Ensure scroll bounds are valid (word wrap aware) |
| `ensure_line_visible(line, total_lines)` | Scroll only if line is off-screen |

### Coordinate Conversion

| Method | Description |
|--------|-------------|
| `pixel_to_line(pixel_y)` | Convert y-coordinate to line number |
| `line_to_pixel(line)` | Convert line number to y-coordinate |

### Horizontal Scroll

| Method | Description |
|--------|-------------|
| `set_horizontal_scroll(offset)` | Set horizontal scroll offset |
| `horizontal_scroll()` | Get current horizontal scroll |

## Overscan

The visible line range includes 5 extra lines above and below the viewport. This **overscan** prevents visual glitches during rapid scrolling by pre-rendering lines just outside the visible area.

```
visible_lines = viewport_height / line_height
start_line = max(0, first_visible_line - 5)
end_line = min(total_lines, first_visible_line + visible_lines + 5)
```

## Integration Points

`ViewState` will be used by:

- **FerriteEditor widget** (Task 6) - Viewport-aware text rendering
- **Viewport rendering** (Task 7) - Culling non-visible lines
- **Cursor management** (Task 8) - Ensuring cursor stays visible

## Design Decisions

### Line Height Source

Line height should ideally come from egui galley metrics (`galley.rows[0].height()`), not hardcoded. The default of 20.0px is a fallback only.

### Word Wrap Mode (Phase 2)

Word wrap is now supported. When enabled:
- Long lines wrap to multiple visual rows
- Each line can have a different height
- Scroll calculations use cumulative height cache for accuracy
- Horizontal scroll is disabled (wrap handles long lines)

### 0-Indexed Lines

All line numbers are 0-indexed internally. Be explicit about conversion when dealing with 1-indexed user-facing line numbers.

## Word Wrap Support

When word wrap is enabled, lines may have different heights. `ViewState` tracks wrapped line heights via:

| Method | Description |
|--------|-------------|
| `set_line_wrap_info(line, visual_rows, height)` | Update wrap info for a line |
| `rebuild_height_cache(total_lines)` | Rebuild cumulative height cache |
| `total_content_height(total_lines)` | Get actual content height (wrap-aware) |
| `get_line_y_offset(line)` | Get y-position of line (wrap-aware) |
| `y_offset_to_line(y, total_lines)` | Find line at y-position (binary search) |

### Scroll Calculation with Word Wrap

The `scroll_by()` and `clamp_scroll_position()` methods use the cumulative height cache to:

1. Convert `first_visible_line` to absolute pixel position via `get_line_y_offset()`
2. Apply scroll delta and clamp to `[0, total_content_height - viewport_height]`
3. Convert back to line number via `y_offset_to_line()` (binary search)

This ensures scrolling works correctly even when wrapped lines have different heights, allowing users to scroll to view all content including the last line.

## Performance

| Operation | Complexity |
|-----------|-----------|
| `get_visible_line_range()` | O(1) or O(visible) with wrap |
| `pixel_to_line()` | O(1) |
| `line_to_pixel()` | O(1) |
| `scroll_by()` | O(log N) with wrap (binary search), O(1) without |
| `y_offset_to_line()` | O(log N) binary search |
| `rebuild_height_cache()` | O(N) - called after wrap changes |

All non-wrap operations are constant-time. With word wrap, scroll calculations use binary search on cumulative heights.

## Testing

Comprehensive unit tests verify:

- Viewport heights from 100-2000px
- Visible range calculation with overscan
- Edge cases (empty documents, small documents)
- Scroll clamping at document bounds
- Coordinate conversions
- Smooth scrolling behavior

Run tests with:

```bash
cargo test editor::view --bin ferrite
```

## Related Documentation

- [TextBuffer](./text-buffer.md) - Rope-based text storage
- [EditHistory](./edit-history.md) - Undo/redo system
- [Custom Editor Widget Plan](../planning/custom-editor-widget-plan.md) - Overall architecture
