# Consecutive Fenced Code Blocks — auto_shrink Fix (issue #129)

**Symptom:** In rendered and split view, when a markdown document contains
two or more fenced code blocks back-to-back (mixed languages or all the same),
only the first block is visible. Subsequent blocks appear blank or are pushed
far below the viewport. Cursor movement or edits sometimes seem to "briefly
refresh" them, but they revert on the next layout pass.

Visible from v0.2.8 onward because the new viewport-culling pipeline (Task 5)
and the per-block height cache (Task 6) latch onto the wrong heights produced
by this layout bug and use them as the canonical layout for subsequent frames.

## Root cause

`EditableCodeBlock`, `MermaidBlock`, and the markdown blockquote / callout
renderers each wrap their content in a horizontal-only `egui::ScrollArea` so
that wide content scrolls horizontally instead of breaking `max_line_width`
(see commit `70699c5`, "fix: horizontal scrolling for code blocks, mermaid
diagrams, and blockquotes"). All four spots used:

```rust
egui::ScrollArea::horizontal()
    .id_salt(...)
    .auto_shrink([false, false]) // ← bug
    .show(ui, |ui| { ... });
```

egui sizes a scroll area's perpendicular axis according to the
`(scroll_enabled, auto_shrink)` truth table in
`egui::containers::scroll_area::Prepared::end`:

```rust
inner_size[d] = match (scroll_enabled[d], auto_shrink[d]) {
    (true, true)   => inner_size[d].min(content_size[d]), // shrink to fit
    (true, false)  => inner_size[d],                      // fill available
    (false, true)  => content_size[d],                    // size to content
    (false, false) => inner_size[d].max(content_size[d]), // expand to max
};
```

For `ScrollArea::horizontal()` the y-axis has `scroll_enabled[1] = false`, so
`auto_shrink[1] = false` selects the last branch: `max(available_height,
content_height)`. When `available_height` exceeds the block's natural height
(true on every fresh layout pass), the inner scroll area is stretched to fill
the rest of the viewport. The first code block consumes almost the entire
visible area; the next block starts after that gap and lands below the
viewport.

`show_rendered_editor` in `src/markdown/editor.rs` then captures
`y_after - y_before` as the measured block height and writes it into the
block-height cache and the `ViewportCullingState`. Subsequent frames take the
fast culling path, see the (still-wrong) cached height, and continue to
position later blocks far off-screen — hence the "invisible until interaction"
behaviour in the report.

## Fix

Switch the perpendicular axis to `auto_shrink_y = true` for every
horizontal-only scroll area in the rendered pipeline. The horizontal axis
keeps `false` so wide content still triggers a horizontal scrollbar instead of
expanding the layout.

| File | Wrapper | Change |
|------|---------|--------|
| `src/markdown/widgets.rs` (~3133) | code-block content (`EditableCodeBlock`) | `auto_shrink([false, false])` → `auto_shrink([false, true])` |
| `src/markdown/widgets.rs` (~4504) | mermaid block content (`MermaidBlock`) | same |
| `src/markdown/editor.rs` (~4361) | blockquote (`render_blockquote`) | same |
| `src/markdown/editor.rs` (~4434) | callout (`render_callout`) | same |

The run-output panel's nested horizontal scroll area (~3406) is intentionally
left at `[false, false]` because it sits inside a vertical `ScrollArea` with a
hard `max_height(220.0)` cap; the perpendicular sizing is bounded there and
the panel is meant to look like a fixed-height pane.

## Regression tests (`src/markdown/editor.rs::tests`)

The visible failure is a layout one and cannot be reproduced in a `cargo
test`-only environment, so the regression tests target the data that the
viewport culling and height cache rely on:

| Test | Verifies |
|------|----------|
| `consecutive_fenced_blocks_parse_as_separate_ast_nodes` | Three back-to-back fenced blocks parse as three top-level `CodeBlock` nodes with valid 1-indexed line ranges. |
| `block_source_slice_extracts_each_consecutive_block_independently` | `block_source_slice` returns each block's own source slice without bleeding into the next block's text. |
| `estimate_block_height_is_finite_and_positive_for_each_block` | `estimate_block_height` produces a finite, positive value scaled by the block's line count. |
| `block_height_cache_keys_distinguish_consecutive_blocks` | The blake3-keyed `BlockHeightCache` returns each block's own height even when several distinct blocks are inserted in sequence. |

If any of these regress, the rendered view's layout pipeline cannot trust the
inputs it relies on and the bug can re-emerge through a different code path.

## Manual verification

`test_md/test_consecutive_code_blocks.md` ships a 5-block reproducer (plain,
C#, Python, Rust, plain). Open in rendered or split view, scroll, and confirm
all five blocks remain visible across viewport changes. Do the same with
`test_md/test_horizontal_scroll.md` (existing) to confirm wide-content
horizontal scrolling still works after the auto_shrink change.

## Related docs

- [Rendered View Viewport Culling](./rendered-view-viewport-culling.md)
- [Block-Level Height Cache](./block-level-height-cache.md)
- [Lazy Block Height Estimation](./lazy-block-height-estimation.md)
