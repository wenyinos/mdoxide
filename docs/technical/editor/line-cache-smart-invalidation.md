# LineCache: Smart Invalidation & Dynamic Sizing

**Task 30** — Improve `LineCache` to support targeted invalidation for changed line ranges and dynamic sizing based on visible lines.

## Problem

Before this change, every content edit (typing a character, pasting text, cutting) called `line_cache.invalidate()` which cleared **all** cached galleys. For a 10 000-line file with 200 cached galleys, a single keystroke destroyed the entire cache, forcing every visible line to be re-laid-out on the next frame.

## Solution

### 1. Content-hash keys already handle correctness

`CacheKey` is computed from `(content_hash, font, color)` — not from line indices. When a line's content changes, the old hash produces a miss and a new entry is created. Lines whose content is unchanged always hit, regardless of whether their line index shifted (e.g. after an insertion above them).

This means **clearing the entire cache on content edits was never necessary for correctness** — only for reclaiming space occupied by stale entries.

### 2. Targeted invalidation (`invalidate_range`)

A reverse index `line_keys: HashMap<usize, CacheKey>` maps each rendered line to the `CacheKey` used for it. During the rendering loop, `register_line(line_idx, content, font_id, color)` populates this index for every visible line.

When content changes, `invalidate_range(start_line, end_line)` looks up the affected lines in `line_keys`, evicts their entries from both the standard and shaped caches, and removes the stale index entries. Lines outside the range are untouched.

```
Edit line 5000 → invalidate_range(5000, 5000)
  ↓
line_keys[5000] → CacheKey(0xABCD)
  ↓
cache.remove(0xABCD)  +  shaped_cache.remove(0xABCD)
  ↓
Lines 0–4999, 5001+ retain their cached galleys
```

### 3. Dirty range tracking in the editor

`FerriteEditor` accumulates a `dirty_range: Option<(usize, usize)>` across edits within a frame:

| Edit path | Tracking |
|-----------|----------|
| Normal typing, IME, Vim input | `mark_lines_dirty(cursor_line, cursor_line)` |
| Cut (selection) | `mark_lines_dirty(sel_start_line, sel_end_line)` |
| Undo / Redo | `mark_dirty()` (full invalidation — arbitrary range) |
| Font / theme / zoom / wrap changes | `invalidate()` (full — all galleys are stale) |

On the next `ui()` frame:

```rust
if self.content_dirty {
    if let Some((start, end)) = self.dirty_range.take() {
        self.line_cache.invalidate_range(start, end);
    } else {
        self.line_cache.invalidate(); // full fallback
    }
    self.content_dirty = false;
}
```

### 4. Dynamic cache capacity

The fixed `const MAX_CACHE_ENTRIES: usize = 200` is replaced by a per-instance limit recomputed every frame:

```rust
max_entries = max(200, visible_lines × 3)
```

| Viewport | Visible lines | Cache capacity |
|----------|---------------|----------------|
| Small (400 px) | ~20 | 200 (floor) |
| Normal (800 px) | ~40 | 200 (floor) |
| Large (1600 px) | ~80 | 240 |
| 4K ultrawide | ~200 | 600 |

The 3× multiplier provides headroom for:
- Overscan lines rendered above/below the viewport
- Temporarily stale entries awaiting LRU eviction
- Lines just scrolled out of view (likely to scroll back)

The shaped-line cache scales similarly at `max(100, visible_lines)`.

## Files Changed

| File | Changes |
|------|---------|
| `src/editor/ferrite/line_cache.rs` | Dynamic `max_cache_entries` / `max_shaped_entries` fields; `update_capacity()`, `register_line()`, `register_line_highlighted()`, `invalidate_range()`, `clear_line_keys()`, `tracked_lines()`; 11 new tests |
| `src/editor/ferrite/editor.rs` | `dirty_range` field; `mark_lines_dirty()` helper; targeted invalidation in `ui()`; `update_capacity()` call after visible range computed; `register_line()` call during rendering; removed redundant double-invalidation from undo/redo |

## Performance Impact

| Scenario | Before | After |
|----------|--------|-------|
| Type character in 10K-line file | ~200 cache entries destroyed, all visible lines re-laid-out | 1 entry evicted, ~39 lines hit cache |
| Scroll after editing | All galleys miss (cache was cleared) | Unchanged lines hit immediately |
| Resize window | Cache stays at 200 | Cache grows to match viewport |
| Font/theme change | Full clear | Full clear (unchanged — correct behavior) |

## Considerations

- **Multi-line edits** (paste spanning lines): `mark_lines_dirty` accumulates the full range via `min/max` merging across the frame.
- **Undo/redo**: Falls back to full invalidation since the affected range is not easily known.
- **Uniform-height mode** (100K+ line files, Task 29): Dynamic sizing still applies; the cache can be smaller since there is no wrap info to track.
- **Thread safety**: `LineCache` remains single-threaded (UI thread only).
- **Memory bound**: With dynamic sizing, a 1M-line file viewed at typical zoom would have a cache capacity of ~600 entries — well under the 50 MB budget.
