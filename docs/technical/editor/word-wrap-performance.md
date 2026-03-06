# Word Wrap Scroll Performance Optimizations

Addresses O(N) hot paths in the word wrap scroll pipeline that cause micro-stuttering on wrapped documents.

## Changes

### 1. Incremental `rebuild_height_cache()` (view.rs)

**Before:** Every call when `wrap_info_dirty` was true iterated all `total_lines` to rebuild `cumulative_heights` from scratch — O(total_lines).

**After:** Tracks `dirty_from_line` (the earliest line whose wrap info changed). Reuses the cached prefix `cumulative_heights[0..=dirty_from_line]` and only recomputes from `dirty_from_line` forward. Common case (scrolling reveals a few new lines) is now O(newly_visible_lines) instead of O(total_lines).

**Key field:** `dirty_from_line: usize` (initialized to `usize::MAX` meaning "clean"). Set to `min(current, changed_line)` on each `set_line_wrap_info()` call.

### 2. O(1) LRU in `LineCache` (line_cache.rs)

**Before:** `VecDeque<CacheKey>` tracked access order. Every cache hit called `update_lru_order()` which did an O(N) linear scan + remove (up to 200 entries, 30–50 hits per frame).

**After:** Monotonic `access_counter: u64` stamps each access. Each `CacheEntry` stores its `last_access` timestamp. Cache hits are now O(1) (HashMap lookup + counter increment). Eviction on cache miss scans all entries for the minimum counter — O(cache_size) but only on misses, which are rare after warm-up.

**Data structure:**
```rust
struct CacheEntry {
    galley: Arc<Galley>,
    last_access: u64,
}

struct LineCache {
    cache: HashMap<CacheKey, CacheEntry>,
    access_counter: u64,
}
```

### 3. O(log N) Visual Row Mapping (view.rs)

**Before:** `logical_to_visual_row()` summed visual rows for all preceding lines — O(N). `visual_row_to_logical()` linearly scanned wrap_info — O(N).

**After:** `cumulative_visual_rows: Vec<usize>` is built alongside `cumulative_heights` during `rebuild_height_cache()`. Structure mirrors `cumulative_heights`:
- `cumulative_visual_rows[i]` = total visual rows for lines `0..i`
- `logical_to_visual_row()`: direct index lookup — O(1)
- `visual_row_to_logical()`: binary search — O(log N)

### 4. Fixed `get_galley_with_job` Cache Key (line_cache.rs)

**Before:** `CacheKey::from_content_hash()` only hashed text content, ignoring font, color, and wrap width. Different `LayoutJob`s for the same text could collide.

**After:** `CacheKey::from_layout_job()` hashes text, wrap width, and each section's byte range, leading space, font family, font size, and color. This prevents incorrect galley returns when the same content is styled differently.

## Performance Impact

| Operation | Before | After |
|-----------|--------|-------|
| `rebuild_height_cache` (scroll) | O(total_lines) | O(changed_lines) |
| Cache hit (per frame, 30–50×) | O(cache_size) | O(1) |
| `logical_to_visual_row` | O(N) | O(1) |
| `visual_row_to_logical` | O(N) | O(log N) |
| Eviction (cache miss) | O(1) | O(cache_size) |

The eviction trade-off is favorable: cache misses are rare after initial warm-up (scrolling reuses galleys), while cache hits happen 30–50 times per frame.

## Files Changed

| File | Changes |
|------|---------|
| `src/editor/ferrite/view.rs` | `dirty_from_line` tracking, `cumulative_visual_rows`, incremental rebuild |
| `src/editor/ferrite/line_cache.rs` | Counter-based LRU, `CacheEntry` struct, `CacheKey::from_layout_job()` |

## Task 41: Verification & Scroll Consistency Fix

### Issue Found: Scroll Sensitivity Problem

**Initial incorrect fix (2026-03-06):**
Added `line_height` multiplication to `editor.rs` scroll calculation to match `mouse.rs`:
```rust
// WRONG - caused 20x overscroll!
let scroll_amount = -scroll_delta.y * scroll_lines * line_height;
```

**Root cause analysis:**
- `smooth_scroll_delta` from egui is already in **points** (pixel-equivalent units)
- Multiplying by `line_height` (20px) caused scroll amounts to be ~20× too large
- User reported: "scrolling just a tiny bit, we scroll down hundreds of lines"

**Correct fix (2026-03-06):**
```rust
// CORRECT - smooth_scroll_delta is already in points/pixels
let scroll_lines = 3.0;
let scroll_amount = -scroll_delta.y * scroll_lines;
```

**Why `mouse.rs` uses `line_height`:**
The `input/mouse.rs` path handles raw `Event::MouseWheel` which has different units. It correctly converts to pixels by multiplying by `line_height`. The `smooth_scroll_delta` in `editor.rs` is already processed/smoothed and in pixel-equivalent units.

### Verification Results

| Test | Result |
|------|--------|
| Build | ✅ Compiles (release mode) |
| Unit tests (47 scroll-related) | ✅ All passed |
| Scroll feel | ✅ Now matches expected sensitivity (~3 lines per notch) |
| Test document | ✅ `test_scroll_perf.md` created (1204 lines with wrapping content) |

### Test Document

Created `test_scroll_perf.md` with 1200 lines of wrapping text for profiling:
```bash
# Lines contain ~150 chars each, designed to wrap to multiple visual rows
Line N with some text content that will wrap when word wrap is enabled...
```

## Related

- [Word Wrap](./word-wrap.md) — Phase 2 word wrap support
- [Word Wrap Scroll Fixes](./word-wrap-scroll-fixes.md) — Task 34 correctness fixes (prerequisite)
- [LineCache](./line-cache.md) — Galley caching documentation
- [ViewState](./view-state.md) — Viewport tracking documentation
