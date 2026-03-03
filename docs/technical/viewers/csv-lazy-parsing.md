# CSV Lazy Row Parsing

## Overview

For large CSV/TSV files (≥1MB), the CSV viewer uses byte-offset indexing to parse only visible rows on demand, rather than loading all rows into memory. This dramatically reduces memory usage and parsing time for large files while maintaining smooth scrolling.

## Key Files

| File | Purpose |
|------|---------|
| `src/markdown/csv_viewer.rs` | All lazy parsing logic: `CsvRowIndex`, `build_csv_row_index()`, `parse_csv_row_range()`, `show_table_view_lazy()`, `CachedVisibleRows` |

## Architecture

### Two rendering paths

The CSV viewer selects a path based on file size:

```
File opened
  ├── < 1MB (small) → Full parse into CsvData, cached
  │                    (was re-parsing every frame before this change)
  │
  └── ≥ 1MB (large) → Build CsvRowIndex (byte offsets only)
                       → Parse only visible rows on demand
                       → Cache parsed rows for smooth scrolling
```

### CsvRowIndex

Lightweight struct storing only the byte offset where each row begins:

```rust
pub struct CsvRowIndex {
    pub row_offsets: Vec<u64>,     // 8 bytes per row
    pub row_count: usize,
    pub num_columns: usize,
    pub column_widths: Vec<usize>, // Sampled from first 1000 rows
    pub first_row: Vec<String>,    // Kept for header detection/display
}
```

**Memory comparison for 1M-row CSV:**

| Approach | Additional memory (beyond raw file string) |
|----------|---------------------------------------------|
| Full parse (`Vec<Vec<String>>`) | ~100–200MB |
| Row index (`Vec<u64>`) | ~8MB |

### On-demand row parsing

`parse_csv_row_range(content, delimiter, index, start, end)` slices the raw content bytes at known offsets and runs the CSV parser on just that slice:

```
content: [row0_bytes][row1_bytes][row2_bytes]...[rowN_bytes]
                      ^                   ^
                      offsets[1]          offsets[3]
                      
parse_csv_row_range(content, delim, index, 1, 3)
  → slices content[offsets[1]..offsets[3]]
  → parses just those 2 rows
```

### Viewport caching

`CachedVisibleRows` stores a window of parsed rows around the current viewport. The cache is wider than the visible area (`LAZY_CACHE_BUFFER = 50` rows on each side) to avoid re-parsing during small scroll increments.

```
|---cache buffer (50 rows)---|
|---render buffer (5 rows)---|
|======visible rows==========|
|---render buffer (5 rows)---|
|---cache buffer (50 rows)---|
```

Re-parsing only occurs when the viewport scrolls beyond the cached range.

### Content hash for cache invalidation

`hash_content_bytes()` uses a fast sampling strategy for large files: hash the file length + first 4KB + last 4KB. This detects most content changes without hashing 100MB+.

## CsvViewerState changes

Two new fields were added:

```rust
pub struct CsvViewerState {
    // ... existing fields ...
    cached_index: Option<CsvRowIndex>,       // Large file row index
    cached_visible: Option<CachedVisibleRows>, // Parsed visible rows
}
```

All cache invalidation paths (`invalidate_cache`, `set_delimiter`, `clear_delimiter_override`) clear both new fields.

## Rendering flow (large file)

Per-frame in `CsvViewer::show()`:

1. **Hash check** — Compare content hash with cached hash (~µs, samples 8KB)
2. **Index** — If hash differs or no index, build `CsvRowIndex` (one-time O(N) scan)
3. **Header detection** — Run `detect_header_row()` on first 5 rows (one-time)
4. **Viewport callback** — `ScrollArea::show_viewport` provides exact visible rect
5. **Cache check** — If cached rows cover visible range, reuse them (most frames)
6. **Parse if needed** — `parse_csv_row_range()` for ~200 rows (~ms)
7. **Render** — `render_row_cells()` paints only visible rows

## Known limitations

- **Files >50MB**: The entire file is still loaded into `tab.content` as a `String`. For truly massive files (86MB+), this causes lag from I/O and memory allocation alone. Memory-mapped file access would be needed to address this (planned for v0.4+).
- **Initial scan**: `build_csv_row_index()` is O(N) on file size. For a 500k-row file (~50MB), this takes ~0.5–1 second on first load.
- **Quoted newlines**: The csv crate correctly handles multi-line quoted fields when building the index, so byte offsets are accurate.

## Tests

18 new tests cover:
- Index building (simple, TSV, empty, large, flexible columns)
- Row range parsing (full, partial, last row, empty, quoted fields, large dataset)
- Content hashing (consistency, large file sampling, change detection)
- State cache invalidation (invalidate_cache, delimiter change)
