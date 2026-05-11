# Flowchart shapes and per-node style

Native flowchart parsing and rendering support extra Mermaid node shapes and `style nodeId …` lines.

## Shapes

| Syntax | `NodeShape` | Notes |
|--------|-------------|--------|
| `[/text\]` | `Trapezoid` | Top edge shorter than base |
| `[\text/]` | `TrapezoidInv` | Inverted trapezoid |
| `(((text)))` | `DoubleCircle` | Parsed before `((` so triple parens are not misread as a circle |
| `[/text/]` | `Parallelogram` | Also parsed here (was missing from the text parser previously) |

Layout sizing uses the same text measurement for all shapes (no shape-specific footprint in `layout/graph.rs`).

## Styling

- **`classDef` / `class`:** unchanged; optional `color:#rrggbb` sets label text color via `NodeStyle::color`.
- **`style nodeId …`:** comma-separated `fill:`, `stroke:`, `stroke-width:`, `color:` (same lexer as `classDef`). Stored in `Flowchart::node_styles`.
- **Precedence:** For each field, a value from `style` overrides `classDef` when present (`NodeStyle::merge_class_and_inline`).

## Code map

- AST: `src/markdown/mermaid/flowchart/types.rs`
- Parser: `src/markdown/mermaid/flowchart/parser.rs` (`parse_node_from_text`, `parse_style_directive`)
- egui draw: `src/markdown/mermaid/flowchart/render/nodes.rs`, style merge in `render/mod.rs`
- HTML SVG: `src/export/flowchart_svg.rs` (polygons / double ellipse, merged colors)
