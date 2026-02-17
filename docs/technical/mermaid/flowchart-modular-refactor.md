# Flowchart Modular Refactoring

## Overview

The monolithic `flowchart.rs` (~3,600 lines) was refactored into 12 focused module files organized under `src/markdown/mermaid/flowchart/`. This improves maintainability, reduces cognitive load, and enables parallel development on different concerns (parsing, layout, rendering).

**Task**: 17 (Refactor flowchart into modular components)
**Date**: Feb 2026

## Module Structure

```
src/markdown/mermaid/flowchart/
├── mod.rs              # Public API re-exports
├── types.rs            # AST types (FlowNode, FlowEdge, Flowchart, etc.)
├── parser.rs           # Mermaid source text -> AST parsing
├── utils.rs            # Shared utilities (bezier curves, arrow heads, etc.)
├── layout/
│   ├── mod.rs          # layout_flowchart() + subgraph bounding boxes
│   ├── config.rs       # FlowLayoutConfig parameters
│   ├── graph.rs        # FlowGraph internal representation
│   ├── subgraph.rs     # SubgraphLayoutEngine for nested subgraphs
│   └── sugiyama.rs     # Core Sugiyama layered graph algorithm
└── render/
    ├── mod.rs          # render_flowchart() orchestration
    ├── colors.rs       # FlowchartColors (dark/light themes)
    ├── nodes.rs        # Node shape drawing (all 10 shapes)
    ├── edges.rs        # Edge routing with subgraph boundary crossing
    └── subgraphs.rs    # Subgraph background + nesting depth
```

## Key Files

| File | Lines | Purpose |
|------|-------|---------|
| `types.rs` | ~150 | All AST types: `Flowchart`, `FlowNode`, `FlowEdge`, `FlowSubgraph`, `NodeShape`, `EdgeStyle`, `ArrowHead`, `NodeStyle`, `LinkStyle`, `NodeLayout`, `SubgraphLayout`, `FlowchartLayout` |
| `parser.rs` | ~950 | Full parser: `parse_flowchart()`, direction, node shapes, edge patterns, chained edges, ampersand syntax, subgraphs, classDef/class, linkStyle |
| `layout/sugiyama.rs` | ~660 | Core Sugiyama layout: cycle detection, layer assignment, crossing reduction (barycenter), coordinate assignment |
| `layout/subgraph.rs` | ~410 | `SubgraphLayoutEngine`: internal layout for simple and hierarchical subgraphs |
| `layout/graph.rs` | ~120 | `FlowGraph`: internal graph representation built from `Flowchart` AST |
| `layout/config.rs` | ~20 | `FlowLayoutConfig`: padding, spacing, max width parameters |
| `layout/mod.rs` | ~170 | `layout_flowchart()` entry point + subgraph bounding box computation |
| `render/edges.rs` | ~400 | Edge drawing: normal edges, back-edges (bezier), subgraph boundary crossing |
| `render/nodes.rs` | ~170 | All node shapes: Rectangle, RoundRect, Stadium, Diamond, Circle, Hexagon, Cylinder, Parallelogram, Asymmetric, Subroutine |
| `render/subgraphs.rs` | ~80 | Subgraph background rendering with nesting depth alternation |
| `render/colors.rs` | ~65 | `FlowchartColors` struct with `dark()` and `light()` constructors |
| `render/mod.rs` | ~120 | `render_flowchart()` orchestration: label pre-computation, subgraph/edge/node draw ordering |
| `utils.rs` | ~270 | Shared: `draw_dashed_line`, `bezier_point`, `draw_bezier_curve`, `draw_arrow_head`, `find_node_subgraph`, `line_rect_intersection` |

## Public API

The public API is unchanged. All external consumers use the same imports:

```rust
// From mermaid/mod.rs
pub use flowchart::{
    layout_flowchart, parse_flowchart, render_flowchart,
    FlowchartColors,
};

// From cache.rs
use super::flowchart::{Flowchart, FlowchartLayout};

// Test-only (pub(crate))
use crate::markdown::mermaid::flowchart::{
    parse_direction, parse_edge_line_full, parse_node_from_text,
    FlowDirection, NodeShape,
};
```

## Design Decisions

### Module Boundaries

- **types.rs** is dependency-free within the flowchart module (only imports `egui` and `std`)
- **parser.rs** depends only on `types`
- **layout/** depends on `types` and `text::TextMeasurer` (for node sizing)
- **render/** depends on `types`, `utils`, and `text::EguiTextMeasurer` (for label measurement)
- **utils.rs** depends on `types` (for `FlowSubgraph`, `Flowchart`)

### Visibility

- `types` are `pub` (used by cache module and tests)
- Parser helpers (`parse_direction`, `parse_edge_line_full`, `parse_node_from_text`) are `pub(crate)` (test access only)
- Layout and render internals are `pub(crate)` (crate-internal use)
- Render sub-modules (`colors`, `nodes`, `edges`, `subgraphs`) are `pub(crate)`

### Edge Routing Refactoring

The `draw_edge` function was decomposed into smaller focused functions:
- `draw_back_edge()` - bezier curve routing for cycle edges
- `draw_normal_edge()` - standard edge with optional subgraph boundary routing
- `compute_edge_endpoints()` - direction-aware start/end point calculation
- `compute_routed_path()` - orthogonal waypoint routing through subgraph boundaries

## Preserved Bug Fixes

All critical bug fixes from previous tasks are preserved in the new modules:

| Fix | Original Task | Location in New Structure |
|-----|---------------|--------------------------|
| Chained edge parsing (`A --> B --> C`) | Task 43 | `parser.rs` - `parse_edge_line_full()` |
| Ampersand edges (`A & B --> C`) | Task 44 | `parser.rs` - `split_by_ampersand()` |
| Subgraph viewport clipping | Task 46 | `layout/mod.rs` - negative coordinate shifting |
| Crash prevention (infinite loops) | Task 57 | `layout/sugiyama.rs` - iteration limits |
| Subgraph title width expansion | Task 56 | `layout/mod.rs` - `compute_subgraph_layouts()` |
| Back-edge detection and curved routing | Task 48 | `layout/sugiyama.rs` + `render/edges.rs` |
| Subgraph edge routing | Task 52 | `render/edges.rs` - `get_subgraph_crossing_info()` |

## Verification

- **Build**: `cargo build` compiles successfully
- **Tests**: All 83 mermaid-related tests pass unchanged
- **Behavior**: Exact rendering preservation - no visual changes

## Related Documents

- [Flowchart Refactor Plan](./flowchart-refactor-plan.md) - Original analysis and planning document
- [Flowchart Layout Algorithm](./flowchart-layout-algorithm.md) - Sugiyama algorithm details
- [Flowchart Subgraphs](./flowchart-subgraphs.md) - Subgraph support details
- [Mermaid Modular Structure](./mermaid-modular-structure.md) - Overall mermaid module organization
