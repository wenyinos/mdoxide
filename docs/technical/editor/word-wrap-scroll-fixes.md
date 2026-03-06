# Word Wrap Scroll Correctness Fixes

## Overview

Fixed multiple functions in the scroll/viewport pipeline that assumed uniform line heights, causing incorrect behavior when word wrap is enabled (where lines can have variable heights depending on how many visual rows they wrap into).

## Key Files

- `src/editor/ferrite/view.rs` — ViewState: scroll position, viewport, coordinate conversion
- `src/editor/widget.rs` — EditorWidget: scroll sync, viewport restoration

## Problem

When word wrap is enabled, a single logical line can span multiple visual rows with height greater than the base `line_height`. Several core functions ignored this and used `line * line_height` for all calculations, causing:

- Click-to-place-cursor mapped to wrong lines
- Go-to-line (Ctrl+G) didn't center correctly
- "Ensure cursor visible" after typing miscalculated viewport bounds
- Sync scrolling between split view panes was misaligned
- Minimap position indicator was wrong
- Scroll restoration after content sync used wrong positions

## Fixes Applied

### view.rs

| Function | Before | After |
|----------|--------|-------|
| `pixel_to_line()` | `pixel_y / line_height` | Uses `get_line_y_offset()` + `y_offset_to_line()` binary search when wrap active |
| `line_to_pixel()` | `line_diff * line_height` | Uses `get_line_y_offset(line) - viewport_top` when wrap active |
| `scroll_to_center_line()` | `viewport_height / line_height` for visible count | Uses actual pixel positions via `scroll_to_absolute()` |
| `is_line_visible()` | `first_visible + ceil(viewport/line_height)` | Compares `get_line_y_offset(line)` against viewport pixel bounds |
| `ensure_line_visible()` | Same uniform assumption | Uses `scroll_to_absolute()` with actual line bottom position |

All fixes are guarded by `self.is_wrap_enabled() && self.cumulative_heights.len() > 1`, so the uniform-height fast path is unchanged for non-wrapped documents.

### widget.rs

| Location | Before | After |
|----------|--------|-------|
| `tab.scroll_offset` | `first_visible * line_height` | `view.current_scroll_y()` (absolute pixel position) |
| `EditorOutput.scroll_offset` | `first_visible * line_height + scroll_offset_y` | Same absolute position |
| `pending_scroll_offset` | `offset / line_height` → `scroll_to_line()` | `scroll_to_absolute(offset, total_lines)` |
| `pending_sync_scroll_offset` | `offset / line_height` → `scroll_to_line()` | `scroll_to_absolute(offset, total_lines)` |
| Viewport restoration | `scroll_offset / line_height` → `scroll_to_line()` | `scroll_to_absolute(scroll_offset, total_lines)` |

## How It Works

`ViewState` maintains two parallel data structures for wrapped documents:

- **`wrap_info: Vec<WrapInfo>`** — Per-line visual row count and pixel height
- **`cumulative_heights: Vec<f32>`** — Prefix sum array where `cumulative_heights[i]` = total height of lines 0..i

These enable O(log N) binary search from y-offset to line number via `y_offset_to_line()`, and O(1) line-to-y-offset via `get_line_y_offset()`.

The fixed functions use these structures when wrap is active, falling back to the `line * line_height` fast path when wrap is off or height data isn't yet available.

## Related Tasks

- **Task 40** — Performance optimization of the wrap scroll pipeline (incremental cache rebuild, O(1) LRU, O(log N) visual row mapping)
- **Task 41** — Profiling and verification of 60fps scroll performance
