# HarfRust text shaping

## Overview

Ferrite integrates **[harfrust](https://crates.io/crates/harfrust) 0.5.2** (pure-Rust HarfBuzz) for OpenType shaping: GSUB/GPOS, Arabic contextual forms, Indic clusters, etc. The pipeline produces glyph runs with cluster mapping and advances in points.

**egui 0.34:** Text is rasterized via epaint's **skrifa** / **vello_cpu** backend. HarfRust does **not** replace that stack — it supplies **horizontal advances and cluster boundaries** for complex-script lines while egui still builds per-cluster mini-galleys for drawing.

## Key Files

| File | Role |
|------|------|
| `src/editor/ferrite/shaping.rs` | `shape_text`, `shape_line_clusters`, `validate_cluster_byte_ranges`, `ShapedCluster`, position-mapping helpers |
| `src/fonts.rs` | `ttf_bytes_for_font_id_shaping`, `row_height_for_font`, `needs_complex_script_fonts` |
| `src/editor/ferrite/line_cache.rs` | `get_shaped_line` — shaped-line LRU cache; `ShapedLine`, `ClusterGalley` |
| `src/editor/ferrite/editor.rs` | Rendering: `get_shaped_line` before `render_line`; IME cursor x via shaped advances |
| `src/editor/ferrite/rendering/cursor.rs` | Unwrapped cursor x for complex scripts |
| `src/editor/ferrite/mouse.rs` | Click-to-column via `shaped_x_to_column` (non-wrapped) |
| `src/editor/ferrite/selection.rs` | Selection rects via `column_to_x_offset` (non-wrapped) |
| `src/editor/ferrite/mod.rs` | `mod shaping`; re-exports `ClusterGalley`, `ShapedLine` |

## Architecture

### Rendering pipeline (complex-script lines, word wrap **off**)

```
line text
  → fonts::needs_complex_script_fonts()
  → shaping::shape_line_clusters()          [harfrust → ShapedCluster vec]
  → per-cluster mini-galley                 [egui/skrifa layout_no_wrap per substring]
  → paint at cumulative HarfRust advance    [painter.galley at x_offset]
```

**Index conventions:** HarfRust `cluster` = UTF-8 **byte** offset. egui `CCursor::index` = **character** index. Never pass byte indices to `cursor_from_pos` / `pos_from_cursor`.

### Caching

`LineCache` maintains `shaped_cache` (LRU, min 100 entries). Keys: content + font + color. Invalidated with standard galley cache on content/font changes.

### Fallback

If shaping is skipped or fails, the standard `get_galley` / `render_line` path runs. Log at `debug` (`ferrite::shaping`).

## Implementation Details

- **`shape_line_clusters(text, font_bytes, font_size_pt)`** — `shape_text` + `group_clusters`.
- **`validate_cluster_byte_ranges`** — test/debug helper; asserts clusters tile the UTF-8 buffer.
- **Per-cluster galleys:** Left-aligned at HarfRust `x_offset`. Cursor/selection use **advances**, not galley widths. If skrifa lays out wider than the shaped advance, a `trace` log is emitted (ligature/cluster mismatch indicator).
- **Script/direction:** `UnicodeBuffer::guess_segment_properties()` + first strong char hint when script is unknown. Clusters are in **visual order** (LTR on screen).

## Dependencies

- **harfrust** 0.5.2 — OTL shaping
- **unicode-script** — script hints

## Usage

```bash
cargo test shaping::          # 31+ unit tests (shape, clusters, roundtrips, validate)
RUST_LOG=ferrite::shaping=trace cargo run
```

## Current scope (post egui 0.34 / task 89.6)

| Aspect | Status |
|--------|--------|
| Cluster-level positioning (HarfRust advances) | ✅ |
| Cursor / click / selection (non-wrapped) | ✅ HarfRust advances |
| IME cursor x (non-wrapped complex script) | ✅ |
| Horizontal scrollbar width | ✅ `shaped.total_width` |
| Line height | ✅ `row_height_for_font` (89.5) |
| OTL glyph forms in atlas | ⚠️ skrifa rasterizes substring codepoints, not HarfRust glyph IDs |
| Word wrap + complex script | ❌ egui wrapped galley only (cursor/selection differ from shaped path) |
| Syntax-highlighted complex script | ❌ plain `get_shaped_line` path only |

## Known limitations

1. **Wrap + complex script:** With word wrap on, cursor/selection use egui's wrapped galley, not HarfRust — expect weaker Arabic/Indic caret accuracy until wrap integration lands.
2. **Draw vs cursor:** Mini-galley width can differ slightly from HarfRust advance; cursor remains advance-based (correct for OTL).
3. **RTL:** Visual-order clusters are painted LTR; full bidi parity is not implemented.

## Follow-up

- Glyph-ID rendering through egui's atlas (true contextual forms without substring fallback).
- Shaped path for wrapped and syntax-highlighted lines.
