# HarfRust text shaping

## Overview

Ferrite integrates **[harfrust](https://crates.io/crates/harfrust) 0.5.2** (pure-Rust HarfBuzz) for OpenType shaping: GSUB/GPOS, Arabic contextual forms, Indic clusters, etc. The pipeline produces glyph runs with cluster mapping and advances in points. For lines containing complex-script characters, shaped cluster positions replace egui's default per-character `ab_glyph` layout as the authoritative text positioning.

## Key Files

| File | Role |
|------|------|
| `src/editor/ferrite/shaping.rs` | `shape_text`, `ShapedGlyph`, `ShapedCluster`, `group_clusters`, `shaped_width`, `ShapeError`; position-mapping: `column_to_x_offset`, `x_to_column`, `shaped_column_to_x`, `shaped_x_to_column` |
| `src/fonts.rs` | `ttf_bytes_for_font_id_shaping`, `needs_complex_script_fonts` |
| `src/editor/ferrite/line_cache.rs` | `get_shaped_line` — shaped-line LRU cache; `ShapedLine`, `ClusterGalley` types; legacy `preshape_complex_script_line` for non-shaped paths |
| `src/editor/ferrite/editor.rs` | Rendering loop: tries `get_shaped_line` before standard `render_line`; `calculate_cursor_x` uses shaped advances for complex scripts |
| `src/editor/ferrite/rendering/cursor.rs` | `calculate_unwrapped_cursor_position` uses shaped advances for complex-script cursor placement |
| `src/editor/ferrite/mouse.rs` | `calculate_column_from_pos` uses `shaped_x_to_column` for complex-script click-to-cursor |
| `src/editor/ferrite/selection.rs` | `render_line_selection` uses `column_to_x_offset` for complex-script selection rectangles |
| `src/editor/ferrite/mod.rs` | `mod shaping`; re-exports `ClusterGalley`, `ShapedLine` |

## Architecture

### Rendering pipeline (complex-script lines)

```
line text
  → fonts::needs_complex_script_fonts()   [guard: skip shaping for Latin/CJK]
  → shaping::shape_text()                 [harfrust: glyphs + cluster byte offsets + advances]
  → shaping::group_clusters()             [merge same-cluster glyphs → ShapedCluster vec]
  → per-cluster mini-galley               [painter.layout_no_wrap per cluster]
  → paint at cumulative x_offset          [painter.galley per cluster]
```

### Caching

`LineCache` maintains a separate `shaped_cache: HashMap<CacheKey, ShapedCacheEntry>` with LRU eviction (max 100 entries). Cache key is content + font + color. Both `invalidate()` (full clear) and `invalidate_line()` (per-line) clear shaped entries alongside standard galley entries. `shaped_len()` exposes the current shaped cache size for debugging.

### Fallback

If any step fails — `needs_complex_script_fonts` returns false, `shape_text` errors, or glyph list is empty — the standard `get_galley` / `render_line` path runs unchanged. Failures are logged at `debug` level (`ferrite::shaping` target).

## Implementation Details

- **`shape_text(text, font_bytes, font_size_pt)`** — `FontRef::new` → `ShaperData` → `shape`. **Cluster** = UTF-8 **byte** index into `text`; advances/offsets scaled by `font_size_pt / units_per_em`.
- **Script/direction:** `UnicodeBuffer::guess_segment_properties()`; if script stays unknown, first strong character via `unicode_script::UnicodeScript` + `harfrust::Script::from_str(short_name)`.
- **`group_clusters(glyphs, text_byte_len)`** — Merges consecutive glyphs sharing the same cluster value. Byte-range boundaries derived from sorted unique cluster offsets. Returns clusters in visual order (left-to-right regardless of text direction).
- **Per-cluster galleys:** Each `ShapedCluster` becomes a mini-galley (`painter.layout_no_wrap`) painted at the cumulative shaped advance. This positions clusters according to OTL-shaped advances rather than `ab_glyph`'s per-codepoint widths.

## Dependencies Used

- **harfrust** — OTL shaping engine (pinned 0.5.2).
- **unicode-script** — Script hints when HarfRust reports unknown script.

## Usage

- **From code:** `crate::editor::ferrite::shaping::shape_text(...)` with bytes from `crate::fonts::ttf_bytes_for_font_id_shaping(&font_id)`.
- **Tests:** `cargo test shaping::` (14 tests including cluster grouping)
- **Debug:** `RUST_LOG=ferrite::shaping=trace` (or `debug`).

## Position-Mapping Helpers (Task 22)

`shaping.rs` provides four helpers that translate between character columns and pixel x-offsets using shaped cluster advances:

- **`column_to_x_offset(text, clusters, char_column)`** — walks clusters, accumulating advances; linearly interpolates within multi-character clusters (ligatures).
- **`x_to_column(text, clusters, x)`** — inverse: snaps to nearest character boundary (midpoint for single-glyph, interpolation for multi-char).
- **`shaped_column_to_x(text, font_bytes, font_size, col)`** — convenience wrapper: shapes + groups + maps.
- **`shaped_x_to_column(text, font_bytes, font_size, x)`** — convenience wrapper for the inverse.

These are used by cursor rendering, IME positioning, mouse click-to-cursor, and selection rendering for complex-script lines. For Latin-only text, the standard egui galley measurement path remains unchanged.

## Current scope and trade-offs

| Aspect | Status |
|--------|--------|
| Cluster-level positioning | ✅ Shaped advances used for x-offset |
| Cursor alignment | ✅ Cursor x uses shaped advances for complex scripts (non-wrapped) |
| Click-to-cursor | ✅ Mouse click maps to correct grapheme using shaped x-to-column |
| Selection rendering | ✅ Selection rectangles use shaped widths for complex scripts |
| IME positioning | ✅ IME candidate window uses shaped cursor x |
| Horizontal scrollbar | ✅ `shaped.total_width` already used for content width |
| Intra-cluster glyph forms | ⚠️ ab_glyph still rasterizes individual codepoints (no OTL glyph ID rendering) |
| Wrapped lines | ❌ Shaped path only active for non-wrapped mode |
| Syntax-highlighted lines | ❌ Shaped path only active for plain text (no syntax highlighting) |

## Risks and follow-up

- Pin version; malformed fonts return `ShapeError`.
- **Follow-up — glyph ID rendering:** Bridge HarfRust glyph IDs into egui's font atlas (or custom mesh) for true OTL glyph forms (contextual Arabic, Indic ligatures). Requires custom rasterization or ab_glyph GID access.
- **Follow-up — broader integration:** Extend shaped path to wrapped lines and syntax-highlighted lines.
