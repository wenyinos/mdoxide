# LineCache - Galley Caching with LRU Eviction

The `LineCache` module provides efficient caching of egui `Galley` objects (text layouts) to avoid expensive galley recreation on each frame. It uses content-based hashing and LRU eviction to bound memory usage.

## Overview

`LineCache` is part of Phase 1 of the FerriteEditor custom widget. It addresses a key performance issue: egui's `Galley` (text layout) is expensive to create, and without caching, we'd recreate galleys for every visible line on every frame.

With caching:
- **Content-hash based keys** - Same content = cache hit
- **LRU eviction** - Max 200 entries to bound memory
- **Single-line galleys** - No wrapping (Phase 1)
- **Font/color aware** - Different styling = different cache entries

## Usage

```rust
use crate::editor::LineCache;
use egui::{Painter, FontId, Color32};

// Create a new cache
let mut cache = LineCache::new();

// Get or create a galley for a line
let galley = cache.get_galley(
    "Hello, World!",
    &painter,
    FontId::monospace(14.0),
    Color32::WHITE,
);

// Same content returns cached galley (cache hit)
let galley2 = cache.get_galley(
    "Hello, World!",
    &painter,
    FontId::monospace(14.0),
    Color32::WHITE,
);

// Use galley.size() to get dimensions
// Use painter.galley(pos, galley, color) to render
```

## Key Methods

### Cache Operations

| Method | Description |
|--------|-------------|
| `new()` | Create empty cache with capacity for 200 entries |
| `get_galley(content, painter, font_id, color)` | Get cached galley or create new one |
| `get_galley_with_job(content, layout_job, painter)` | Get galley with complex styling (LayoutJob) |
| `invalidate()` | Clear all cached galleys |
| `invalidate_line(content, font_id, color)` | Remove specific entry from cache |

### Cache Information

| Method | Description |
|--------|-------------|
| `len()` | Number of cached galleys |
| `is_empty()` | Check if cache is empty |
| `capacity()` | Maximum cache size (200) |
| `is_cached(content, font_id, color)` | Check if entry exists without modifying LRU |

## Cache Key Design

Each cache entry is keyed by a hash combining:
- **Content** - The line text content
- **Font family** - Monospace, Proportional, or custom
- **Font size** - Exact floating-point size (compared as bits)
- **Text color** - RGBA color values

This means:
- Same content with different fonts = different cache entries
- Same content with different colors = different cache entries
- Identical lines share the same cached galley

## LRU Eviction

When the cache reaches 200 entries:
1. The least recently used entry is evicted
2. New entry is added to the cache
3. Access order is tracked via a VecDeque

This bounds memory usage while keeping frequently-used galleys cached.

```
Cache hit → Move entry to back of LRU queue (most recent)
Cache miss → Create galley, evict oldest if at capacity, add to back
```

## Integration Points

`LineCache` will be used by:

- **Viewport rendering** (Task 7) - Efficiently render visible lines
- **FerriteEditor widget** (Task 6) - Line-by-line text rendering

## When to Invalidate

Call `invalidate()` when:
- Theme changes (colors change)
- Font changes (font family or size)
- Zoom level changes

Call `invalidate_line()` when:
- Specific line content changes (alternative to full invalidation)

## Performance

| Operation | Complexity |
|-----------|-----------|
| `get_galley()` (cache hit) | O(n) for LRU update |
| `get_galley()` (cache miss) | O(1) + galley creation |
| `invalidate()` | O(1) |
| `invalidate_line()` | O(n) |

Note: LRU update is O(n) due to VecDeque position search, but n is bounded at 200 entries which is fast in practice.

## Memory Usage

With 200 cached galleys:
- Each `Arc<Galley>` contains text layout data
- Typical memory usage: 2-5 MB for the cache
- Bounded by `MAX_CACHE_ENTRIES` (200)

## Testing

Comprehensive unit tests verify:

- Cache key equality and hashing
- LRU eviction at 200 entries
- Content-hash determinism
- Unicode and emoji support
- Whitespace sensitivity
- Font and color differentiation

Run tests with:

```bash
cargo test line_cache
```

## Design Decisions

### Why Content-Hash Keys?

Using content hashes instead of line indices allows:
- Identical lines to share cached galleys
- Efficient cache hits after text edits
- No need to invalidate on cursor movement

### Why 200 Entries?

The 200-entry limit balances:
- Memory usage (~2-5 MB)
- Cache hit rate (typical viewport shows ~50 lines)
- LRU overhead (O(n) search with n=200 is fast)

### Why No LayoutJob Caching by Default?

`LayoutJob` (for syntax highlighting) involves complex styling that's harder to hash efficiently. The `get_galley_with_job()` method uses content-only keys, which may cause cache misses if the same content has different styling. This is a known limitation for Phase 1.

## Related Documentation

- [ViewState](./view-state.md) - Visible line range calculation
- [TextBuffer](./text-buffer.md) - Rope-based text storage
- [EditHistory](./edit-history.md) - Undo/redo system
- [Custom Editor Widget Plan](../planning/custom-editor-widget-plan.md) - Overall architecture
