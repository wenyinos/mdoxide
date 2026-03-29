# Uniform Height Mode for Large Files

**Task:** 29  
**Version:** v0.2.8  
**Status:** Done

## Overview

Files with more than 100,000 lines activate **uniform height mode** in `ViewState`. This avoids O(N) memory allocation for per-line vectors (`wrap_info`, `cumulative_heights`, `cumulative_visual_rows`), keeping large file performance smooth.

## How It Works

### Threshold

The constant `LARGE_FILE_THRESHOLD` in `view.rs` is set to 100,000 lines. `configure_for_file_size(total_lines)` is called every frame in `FerriteEditor::ui()`.

### When Activated

When `total_lines >= 100,000`:

- `use_uniform_heights` is set to `true`
- All O(N) per-line vectors are cleared and shrunk to free memory
- `set_line_wrap_info()` becomes a no-op (no per-line height data stored)
- `rebuild_height_cache()` returns immediately
- `get_line_height()` returns the single `line_height` value for all lines
- `get_line_y_offset(line)` returns `line * line_height` (no cumulative array)
- `y_offset_to_line(y)` returns `y / line_height` (no binary search)
- `total_content_height()` returns `total_lines * line_height`

### Word Wrap Override

The previous line overlap regression was caused by word-wrapped galleys being taller than the uniform `line_height`. A line wrapping to 2 visual rows would produce a 40px galley, but uniform spacing allocated only 20px.

**Fix:** When uniform heights are active, word wrap is force-disabled for the frame via `effective_wrap_enabled = self.wrap_enabled && !self.view.uses_uniform_heights()`. This local variable replaces `self.wrap_enabled` in:

- View wrap configuration (`enable_wrap`/`disable_wrap`)
- The render loop's wrap vs non-wrap branch
- Height cache rebuild calls
- Horizontal scrollbar visibility

The user's wrap preference (`self.wrap_enabled`) is preserved — wrap re-enables automatically if the file drops below the threshold.

### Cursor Positioning

`calculate_cursor_x()` uses `self.view.is_wrap_enabled()` (the actual runtime state) rather than `self.wrap_enabled` (the user preference), so cursor positioning automatically respects the uniform-height override.

## Performance Characteristics

| Metric | Without Uniform | With Uniform (100K+ lines) |
|--------|----------------|---------------------------|
| Memory for height vectors | O(N) (~2.4 MB for 100K lines) | O(1) (0 bytes) |
| `get_line_y_offset` | O(1) array lookup | O(1) multiplication |
| `y_offset_to_line` | O(log N) binary search | O(1) division |
| `rebuild_height_cache` | O(N) or O(delta) | No-op |
| Word wrap | Available | Force-disabled |

## Key Files

| File | Changes |
|------|---------|
| `src/editor/ferrite/view.rs` | `configure_for_file_size()` re-enabled with threshold + vector cleanup; `rebuild_height_cache()` early-returns in uniform mode |
| `src/editor/ferrite/editor.rs` | `effective_wrap_enabled` computed after `configure_for_file_size()`; used in wrap setup, render loop, scrollbar, and cursor positioning |

## Tests

12 new tests in `view.rs` covering:
- Activation/deactivation at threshold
- Vector cleanup on activation
- Uniform `get_line_height`, `get_line_y_offset`, `total_content_height`
- `set_line_wrap_info` is a no-op
- `rebuild_height_cache` is a no-op
- `y_offset_to_line` correctness
- `scroll_by` with uniform heights
- Deactivation when file shrinks below threshold
