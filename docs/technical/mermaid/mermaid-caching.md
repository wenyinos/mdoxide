# Mermaid Diagram Caching

This document describes the AST and layout caching implementation for Mermaid diagrams, which significantly improves rendering performance by avoiding redundant parsing and layout calculations.

## Overview

Mermaid diagrams (particularly flowcharts) require multiple expensive operations:

1. **Parsing**: Converting source text to an Abstract Syntax Tree (AST)
2. **Layout**: Computing node positions using the Sugiyama algorithm
3. **Rendering**: Drawing the diagram to the screen

Without caching, these operations run on every frame, which is wasteful for unchanged diagrams.

## Cache Architecture

### Cache Key

The cache key is a combination of:

```rust
struct CacheKey {
    /// Blake3 hash of the source code
    source_hash: [u8; 32],
    /// Font size rounded to nearest 0.5
    font_size_x2: u32,
    /// Available width rounded to nearest 10 pixels
    width_div10: u32,
}
```

**Why these components?**

- **Source hash**: Changes when diagram content changes
- **Font size**: Affects text measurement and node sizing
- **Available width**: Affects layout spacing and centering

Font size and width are rounded to prevent cache thrashing from minor floating-point variations.

### Cached Data

For flowcharts, the cache stores:

```rust
struct CachedFlowchart {
    flowchart: Flowchart,      // Parsed AST
    layout: FlowchartLayout,   // Computed positions
    last_access: Instant,      // For LRU eviction
}
```

### Cache Manager

The `MermaidCacheManager` provides:

- O(1) lookup by cache key
- LRU (Least Recently Used) eviction when full
- Statistics tracking (hits, misses, evictions)
- Thread-safe access via a global Mutex

## Implementation

### Global Cache

A single global cache is used, initialized lazily:

```rust
static DIAGRAM_CACHE: Mutex<Option<MermaidCacheManager>> = Mutex::new(None);

fn with_cache<F, R>(f: F) -> R
where
    F: FnOnce(&mut MermaidCacheManager) -> R,
{
    let mut guard = DIAGRAM_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let cache = guard.get_or_insert_with(MermaidCacheManager::new);
    f(cache)
}
```

### Rendering Flow

```
Source Code → Compute Cache Key → Check Cache
                                      │
                    ┌─────────────────┴─────────────────┐
                    ↓                                   ↓
              Cache Hit                           Cache Miss
                    │                                   │
                    ↓                                   ↓
           Use cached AST                     Parse → Layout
           and layout                               │
                    │                               ↓
                    │                         Store in cache
                    │                               │
                    └───────────────┬───────────────┘
                                    ↓
                                 Render
```

### Cache Invalidation

Entries are automatically invalidated when:

1. **Source changes**: Different blake3 hash
2. **Font size changes**: Exceeds rounding threshold (0.5)
3. **Width changes**: Exceeds rounding threshold (10px)
4. **LRU eviction**: Cache is full (default: 50 entries)

Manual invalidation via `clear_diagram_cache()` for:

- Theme changes
- Global font settings changes

## Performance Characteristics

### Targets

| Metric | Target |
|--------|--------|
| Cache hit render | <10ms for 50+ node diagrams |
| Cache hit rate | >90% during normal use |
| Memory per entry | ~5-50KB depending on complexity |

### Trade-offs

**Pros:**
- Eliminates redundant parsing (O(n) per line)
- Eliminates redundant layout (O(V+E) graph operations)
- Fast blake3 hashing (~1GB/s)

**Cons:**
- Memory overhead for cached entries
- Mutex contention (minimal in single-threaded UI)
- Clone operations for cached data

## Usage

The cache is transparent to callers. Simply use `render_mermaid_diagram()`:

```rust
// Automatic caching - no changes needed
match render_mermaid_diagram(ui, source, dark_mode, font_size) {
    RenderResult::Success => {},
    RenderResult::ParseError(e) => log::error!("{}", e),
    _ => {}
}
```

### Monitoring

Access cache statistics:

```rust
if let Some(stats) = get_cache_stats() {
    log::info!(
        "Cache: {} hits, {} misses, {:.1}% hit rate",
        stats.hits,
        stats.misses,
        stats.hit_rate()
    );
}
```

### Manual Cache Control

```rust
// Clear cache when theme changes
clear_diagram_cache();
```

## File Structure

```
src/markdown/mermaid/
├── mod.rs           # Global cache, render_mermaid_diagram integration
├── cache.rs         # CacheKey, MermaidCacheManager, CachedFlowchart
├── flowchart.rs     # Flowchart, FlowchartLayout (cached types)
└── ...
```

## Future Improvements

1. **Per-diagram-type caching**: Extend beyond flowcharts to sequence, state, etc.
2. **Size-based eviction**: Consider memory usage, not just entry count
3. **Warm-up**: Pre-parse visible diagrams on file load
4. **Persistence**: Save cache to disk for faster startup (likely overkill)

## Dependencies

- `blake3`: Fast cryptographic hashing for source fingerprinting
