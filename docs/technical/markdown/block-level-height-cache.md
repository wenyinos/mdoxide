# Block-Level Height Cache

Persistent per-block height cache for the rendered view's viewport culling system. Eliminates the O(all-blocks) measurement pass on content edits by caching previously-measured block heights keyed by blake3 content hash.

## Problem

Task 5 introduced viewport culling with a two-phase approach:

1. **Measurement pass** (first frame / content change): renders all blocks, records heights into `ViewportCullingState`.
2. **Culled pass** (subsequent frames): binary-searches cached positions, renders only visible blocks.

The measurement pass is O(N) in block count because every block must be rendered to measure its height. When the user edits a single paragraph in a 500-block document, all 500 blocks are re-rendered just to re-measure heights — even though 499 of them haven't changed.

## Solution

A global LRU cache in `src/markdown/cache.rs` maps `(blake3(block_source), render_params_hash)` → `height: f32`. During the measurement pass, off-screen blocks with a cached height skip rendering entirely — only `ui.allocate_space()` is called to advance the layout cursor.

### Cache Key

| Component | Type | Purpose |
|-----------|------|---------|
| `content_hash` | `blake3([u8; 32])` | Blake3 hash of the block's source markdown text |
| `render_params_hash` | `u64` | Hash of `available_width` (as u32) and `font_size * 100` (as u32) |

Width and font size are included because they directly affect line wrapping and thus block height.

### Cache Parameters

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Max entries | 512 | Covers large documents (~500 blocks) with headroom |
| Eviction | LRU (least recently accessed) | Bounded memory; old entries expire naturally |

## Data Flow

```
Content edit detected
        │
        ▼
ViewportCullingState invalidated (content_hash changed)
        │
        ▼
Measurement pass begins
        │
        ▼
For each block:
  ┌─────────────────────────────┐
  │ Extract block source lines  │
  │ Check BlockHeightCache      │
  └──────────┬──────────────────┘
             │
     ┌───────┴────────┐
     │                 │
  Cache HIT         Cache MISS
  + off-screen      (or visible)
     │                 │
     ▼                 ▼
  allocate_space()   render_node()
  (skip render)      insert into cache
     │                 │
     └───────┬─────────┘
             ▼
  Record height in measured_heights[]
        │
        ▼
ViewportCullingState repopulated
        │
        ▼
Subsequent frames: fast culled path (Task 5)
```

## Key Files

| File | Changes |
|------|---------|
| `src/markdown/cache.rs` | `BlockHeightCache`, `BlockHeightKey`, `get_block_height()`, `insert_block_height()`, `render_params_hash()`, `clear_block_height_cache()` |
| `src/markdown/editor.rs` | `line_start_byte_offsets()`, `block_source_slice()`, modified measurement pass in `show_rendered_editor()` |

## Block Source Extraction

To hash individual blocks, the measurement pass needs each block's source markdown. `MarkdownNode` provides `start_line` and `end_line` (1-indexed). Two helper functions extract the corresponding byte slice:

- `line_start_byte_offsets(content)` — precomputes byte offset of each line start (O(N) once per pass)
- `block_source_slice(content, offsets, start_line, end_line)` — returns `&str` slice for the block

## Visibility Check

During the measurement pass, blocks are classified as:

- **Visible** (in/near viewport + overscan): always rendered for visual correctness
- **Off-screen + cache hit**: `allocate_space()` placeholder (no render)
- **Off-screen + cache miss**: rendered to measure (first-time blocks)

This ensures the user never sees blank space for visible blocks, while off-screen blocks benefit from the cache.

## Limitations

- **Font family and paragraph indent** are not currently included in the render params hash. Changing these settings may produce slightly inaccurate cached heights for one frame until re-measured. This is acceptable because these settings change rarely.
- **In-place edits** during the measurement pass (rendered-mode WYSIWYG editing) may cause line offsets to shift. The re-extraction after `render_node` returns uses the current content, so the cache insert is correct for the post-edit state.

## Tests

Unit tests in `src/markdown/cache.rs`:

| Test | Verifies |
|------|----------|
| `block_height_cache_hit` | Same content + params → cached height returned |
| `block_height_cache_miss_different_content` | Different block content → cache miss |
| `block_height_cache_miss_different_width` | Different available width → cache miss |
| `block_height_cache_miss_different_font_size` | Different font size → cache miss |
| `block_height_lru_eviction` | Cache stays bounded at 512 entries |
| `clear_block_height_cache_works` | Manual cache clear removes all entries |
