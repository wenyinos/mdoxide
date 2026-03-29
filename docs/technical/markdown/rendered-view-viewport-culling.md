# Rendered View Viewport Culling

## Overview

The rendered (WYSIWYG) markdown view now renders only the blocks visible on screen plus an overscan buffer, instead of laying out every block in the document each frame. Combined with the AST cache (Task 4), this makes the rendered view practical for large documents (thousands of lines at 30+ fps).

## Problem

`show_rendered_editor()` in `src/markdown/editor.rs` used `ScrollArea::vertical().show()` and iterated over **every** top-level AST node each frame, calling `render_node()` for each. For a 6 000-line markdown file with ~200+ top-level blocks, this meant laying out hundreds of headings, paragraphs, code blocks, tables, etc. every frame — even though only ~10–20 blocks are visible at any time.

## Solution

### Two-phase rendering with `show_viewport()`

Replaced `scroll_area.show()` with `scroll_area.show_viewport()`, which passes a `viewport: Rect` describing the visible region in content coordinates.

**Phase 1 — Measurement pass** (first frame after content/width change):
All blocks are rendered normally. For each block the start-Y offset and rendered height are recorded and stored as `ViewportCullingState` in egui temp memory, keyed by a content hash + available width.

**Phase 2 — Culled pass** (subsequent frames with unchanged content):
1. `ui.set_min_height(total_height)` tells the scroll area the full content extent.
2. A binary search over `block_start_y` finds the first and last blocks overlapping `[viewport.top − overscan, viewport.bottom + overscan]`.
3. `ui.allocate_space()` reserves vertical space for blocks above the visible range.
4. Only the visible blocks are rendered via `render_node()`.
5. `ui.allocate_space()` reserves space for blocks below.
6. Off-screen `LineMapping` entries are populated from cached positions (for scroll sync).

### Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `VIEWPORT_OVERSCAN_PX` | 500 px | Pre-render buffer above/below the viewport |
| `BLOCK_ITEM_SPACING_Y` | 1.0 px | Matches `item_spacing.y` set during layout |

### Cache invalidation

`ViewportCullingState` is invalidated when:
- **Content changes** — content hash mismatch triggers a new measurement pass.
- **Available width changes** — width delta > 1 px triggers re-measurement (reflowing may change block heights).

## Key types

```rust
struct ViewportCullingState {
    content_hash: u64,
    available_width: f32,
    block_start_y: Vec<f32>,   // Y offset of each block (includes spacing)
    block_heights: Vec<f32>,   // Height of each block
    total_height: f32,         // Measured total content height
}
```

Stored in `ui.memory()` temp data, keyed by `id.with("viewport_culling")`.

## Files changed

| File | Change |
|------|--------|
| `src/markdown/editor.rs` | `ViewportCullingState` struct, `VIEWPORT_OVERSCAN_PX` / `BLOCK_ITEM_SPACING_Y` constants, `show_rendered_editor()` rewritten to use `show_viewport()` with two-phase culling |

## Performance characteristics

| Scenario | Before | After |
|----------|--------|-------|
| 200-block document, all rendered each frame | O(N) `render_node` calls | O(visible + 2×overscan/avg_height) calls |
| Scroll position change | Full re-layout | Only newly-visible blocks re-rendered |
| Content edit | Full re-parse + re-layout | Re-parse (cached unless hash changes) + one measurement frame, then culled |

## Relationship to Task 4 (AST Caching)

Task 4 eliminated re-**parsing** unchanged content each frame. Task 5 eliminates re-**rendering** off-screen blocks. Together they reduce per-frame cost from O(parse + render_all) to O(cache_lookup + render_visible).
