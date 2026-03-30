# Lazy Block Height Estimation

**Task 32** — Reduce first-frame cost for large rendered markdown documents by using heuristic heights for blocks that have never been measured.

## Problem

When opening a 10K+ block markdown file in Rendered view, the first frame had to measure (render via egui) every block to build the `ViewportCullingState` layout. For very large files this exceeded 100 ms and caused visible stutter.

## Solution

A two-phase lazy estimation system that builds the viewport culling state immediately using a mix of real measurements and heuristic estimates, then progressively refines the estimates over subsequent frames.

### Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `MAX_NEW_MEASUREMENTS_PER_FRAME` | 20 | Cap on blocks newly measured (full egui render) per frame |
| `ESTIMATED_LINE_HEIGHT_PX` | 20.0 | Baseline px/line for heuristic estimates |

### Heuristic Height

For a block spanning `start_line..=end_line` (1-indexed from the parser):

```
height = max(line_count * 20.0, 20.0)
```

This is intentionally simple. It will overshoot for headings (which are taller but fewer lines) and undershoot for complex blocks, but the scrollbar self-corrects as blocks are measured.

### `ViewportCullingState` Changes

Added `block_measured: Vec<bool>` — per-block flag indicating whether the stored height came from a real egui render (`true`) or the heuristic (`false`).

### Bootstrap Pass (First Frame)

When no valid culling state exists:

1. **Phase 1** — For every block, look up the block-height cache. Cache hit → `measured = true`, use cached height. Cache miss → `measured = false`, use heuristic estimate.
2. **Phase 2** — Build `block_start_y` positions from the heights.
3. **Phase 3** — Render only the viewport + 500 px overscan blocks, capped by the render budget (20 new measurements). Blocks within the visible range that are already measured (cache hit) are rendered regardless of budget. Blocks exceeding the budget use `allocate_space` with their estimated height.
4. Persist the culling state with the mixed measured/estimated heights.
5. If any blocks remain unmeasured, `request_repaint()` to trigger progressive measurement on subsequent frames.

### Fast Path (Subsequent Frames)

When a valid culling state exists:

1. Binary-search for the visible block range (unchanged from Task 5).
2. Render all visible blocks. For each block that was previously unmeasured, its real height is recorded and the block-height cache is updated.
3. If any heights changed, rebuild `block_start_y` and `total_height`, persist the updated culling state.
4. If unmeasured blocks remain, `request_repaint()` to continue progressive refinement.

### Invalidation

Content changes are detected via `content_hash` comparison. When the document is edited, the culling state is invalidated entirely — the bootstrap pass re-runs, re-checking the block-height cache (which retains heights for unchanged blocks) and estimating heights for new/changed blocks.

## Files Changed

| File | Change |
|------|--------|
| `src/markdown/editor.rs` | `ViewportCullingState.block_measured`, `estimate_block_height()`, `MAX_NEW_MEASUREMENTS_PER_FRAME`, `ESTIMATED_LINE_HEIGHT_PX`, rewritten bootstrap/fast paths |

## Performance Characteristics

- **First frame**: O(N) to estimate heights + O(viewport) to render visible blocks. With 10K blocks this is ~1 ms for estimation + typical viewport render time.
- **Progressive refinement**: ~20 blocks measured per frame until all viewport-visible blocks are measured. Off-screen blocks remain estimated until scrolled into view.
- **Scrollbar accuracy**: Approximately correct from first frame. Becomes fully accurate after scrolling through the document.
- **Memory**: One extra `Vec<bool>` in the culling state (1 byte per block).
