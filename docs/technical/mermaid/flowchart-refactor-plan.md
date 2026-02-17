# Flowchart Module Refactoring Plan

> **Status: COMPLETED** (Feb 2026, Task 17). See [Flowchart Modular Refactor](./flowchart-modular-refactor.md) for the implementation results.

## Overview

This document provides a comprehensive analysis of `src/markdown/mermaid/flowchart.rs` (~3,500 lines) and proposes a detailed refactoring strategy to split it into focused, maintainable modules.

**Goal**: Improve maintainability, reduce cognitive load, enable parallel development, and preserve all recent bug fixes (Tasks 43, 44, 46, 57).

## Current Structure Analysis

### File Statistics

| Section | Lines | % of File | Description |
|---------|-------|-----------|-------------|
| AST Types | 1-135 | 4% | Core data structures |
| Parser | 136-1034 | 26% | Parsing logic |
| Layout Engine | 1036-2497 | 42% | Sugiyama algorithm, subgraph layout |
| Renderer | 2499-3496 | 28% | Drawing functions |
| **Total** | 3,496 | 100% | |

### Section Breakdown

#### 1. AST Types (Lines 1-135, ~135 lines)

Core data structures representing the flowchart model:

| Type | Purpose | Dependencies |
|------|---------|--------------|
| `FlowDirection` | Enum: TopDown, BottomUp, LeftRight, RightLeft | None |
| `NodeShape` | Enum: Rectangle, RoundRect, Stadium, Diamond, etc. | None |
| `FlowNode` | Struct: id, label, shape | `NodeShape` |
| `EdgeStyle` | Enum: Solid, Dotted, Thick | None |
| `ArrowHead` | Enum: Arrow, Circle, Cross, None | None |
| `FlowEdge` | Struct: from, to, label, style, arrows | `EdgeStyle`, `ArrowHead` |
| `FlowSubgraph` | Struct: id, title, node_ids, children | `FlowDirection` |
| `NodeStyle` | Struct: fill, stroke, stroke_width | `egui::Color32` |
| `Flowchart` | Main container: nodes, edges, subgraphs, classes | All above |

**Issues**:
- `egui::Color32` import creates renderer dependency in types
- `NodeStyle` is styling concern mixed with AST

#### 2. Parser (Lines 136-1034, ~900 lines)

Parsing functions for converting Mermaid source text to AST:

| Function | Lines ~ | Purpose | Recent Fixes |
|----------|---------|---------|--------------|
| `parse_flowchart` | 141-334 | Main entry point | - |
| `SubgraphBuilder` | 338-344 | Helper struct for parsing | - |
| `parse_subgraph_header` | 348-402 | Extract subgraph id/title | - |
| `parse_direction` | 404-421 | Parse TD/LR/BT/RL | Task 43 (semicolon) |
| `parse_class_def` | 425-475 | Parse classDef directive | - |
| `parse_class_assignment` | 479-511 | Parse class assignments | - |
| `parse_css_color` | 515-550 | Parse hex colors | - |
| `parse_stroke_width` | 554-558 | Parse stroke-width values | - |
| `ARROW_PATTERNS` | 562-577 | Edge pattern definitions | Task 44 |
| `find_arrow_pattern` | 581-599 | Find arrow in text | Task 44 |
| `parse_edge_label` | 603-617 | Extract pipe-style labels | - |
| `extract_dash_label` | 631-686 | Extract dash-style labels | Task 44 |
| `strip_trailing_semicolon` | 688-691 | Remove trailing ; | Task 43 |
| `split_by_ampersand` | 695-740 | Handle A & B syntax | Task 43 |
| `parse_edge_line_full` | 745-850 | Main edge parser | Task 43, 44 |
| `parse_node_from_text` | 852-1009 | Parse node shapes | Task 54 (asymmetric) |
| `extract_id` | 1012-1021 | Extract node ID | - |
| `clean_label` | 1025-1030 | Clean HTML in labels | - |
| `parse_node_definition` | 1032-1034 | Wrapper for FlowNode | - |

**Issues**:
- Large function count makes navigation difficult
- `parse_edge_line_full` is complex (~100 lines) with multiple concerns
- Helper functions scattered throughout

#### 3. Layout Engine (Lines 1036-2497, ~1,460 lines)

The largest section, implementing Sugiyama-style layered graph layout:

| Component | Lines ~ | Purpose | Recent Fixes |
|-----------|---------|---------|--------------|
| `NodeLayout` | 1042-1045 | Position and size for a node | - |
| `SubgraphLayout` | 1048-1056 | Subgraph bounding box | - |
| `FlowchartLayout` | 1059-1066 | Complete layout result | - |
| `layout_flowchart` | 1079-1113 | Main entry point | - |
| `compute_subgraph_layouts` | 1120-1284 | Compute subgraph bounds | Task 57 |
| `FlowLayoutConfig` | 1287-1301 | Layout parameters | - |
| `FlowGraph` | 1304-1422 | Internal graph representation | - |
| `SugiyamaLayout` | 1425-2497 | Core layout algorithm | Task 46 |
| `SubgraphInternalLayout` | 1437-1445 | Internal subgraph positions | - |
| `SubgraphLayoutEngine` | 1448-1833 | Subgraph content layout | Task 56 |

**SugiyamaLayout Methods**:

| Method | Lines ~ | Purpose | Recent Fixes |
|--------|---------|---------|--------------|
| `new` | 1836-1850 | Constructor | - |
| `compute` | 1854-1880 | Main algorithm | - |
| `layout_subgraphs_inside_out` | 1884-1909 | Subgraph ordering | - |
| `get_subgraph_processing_order` | 1912-1944 | Topological sort | - |
| `detect_cycles_and_mark_back_edges` | 1948-1961 | Cycle detection | - |
| `dfs_find_back_edges` | 1963-1983 | DFS helper | - |
| `assign_layers` | 1991-2058 | Layer assignment | - |
| `cluster_subgraph_layers` | 2066-2117 | Group subgraph nodes | - |
| `compute_subgraph_relative_layers` | 2121-2186 | Relative layering | - |
| `ensure_layer_constraints` | 2190-2213 | Constraint propagation | - |
| `build_layers` | 2216-2241 | Build layer structure | - |
| `get_min_incoming_edge_position` | 2245-2257 | Edge position helper | - |
| `reduce_crossings` | 2261-2275 | Barycenter heuristic | - |
| `order_layer_by_barycenter` | 2279-2371 | Single layer ordering | - |
| `assign_coordinates_with_subgraphs` | 2378-2496 | Final positioning | Task 46 |

**Issues**:
- `SugiyamaLayout` is monolithic (~660 lines)
- `SubgraphLayoutEngine` is tightly coupled
- Direction handling duplicated in multiple places

#### 4. Renderer (Lines 2499-3496, ~1,000 lines)

Drawing functions for rendering the flowchart:

| Function | Lines ~ | Purpose |
|----------|---------|---------|
| `FlowchartColors` struct | 2505-2556 | Color configuration |
| `EdgeLabelInfo` struct | 2559-2563 | Pre-computed label sizes |
| `render_flowchart` | 2565-2675 | Main render entry point |
| `compute_subgraph_depths` | 2679-2705 | Calculate nesting depth |
| `draw_subgraph` | 2707-2743 | Draw subgraph box/title |
| `draw_node` | 2745-2901 | Draw node shapes |
| `draw_edge` | 2903-3222 | Draw edges with routing |
| `draw_dashed_line` | 3224-3243 | Helper for dotted edges |
| `bezier_point` | 3246-3257 | Bezier math |
| `draw_bezier_curve` | 3260-3277 | Draw curved edges |
| `draw_arrow_head` | 3279-3325 | Draw arrow heads |
| `find_node_subgraph` | 3329-3341 | Find containing subgraph |
| `line_rect_intersection` | 3345-3408 | Intersection math |
| `SubgraphCrossingInfo` | 3411-3417 | Edge crossing data |
| `get_subgraph_crossing_info` | 3421-3495 | Compute boundary crossings |

**Issues**:
- `draw_edge` is very complex (~320 lines)
- Geometry helpers mixed with rendering
- Color definitions embedded in renderer

---

## Proposed Module Structure

```
src/markdown/mermaid/flowchart/
├── mod.rs              # Re-exports, public API (~80 lines)
├── types.rs            # AST types, enums (~150 lines)
├── parser.rs           # All parsing functions (~900 lines)
├── layout/
│   ├── mod.rs          # Layout entry point (~100 lines)
│   ├── graph.rs        # FlowGraph, adjacency (~200 lines)
│   ├── sugiyama.rs     # SugiyamaLayout (~500 lines)
│   ├── subgraph.rs     # SubgraphLayoutEngine (~400 lines)
│   └── config.rs       # FlowLayoutConfig (~50 lines)
├── render/
│   ├── mod.rs          # Render entry point (~100 lines)
│   ├── colors.rs       # FlowchartColors (~70 lines)
│   ├── nodes.rs        # draw_node (~200 lines)
│   ├── edges.rs        # draw_edge, arrows (~400 lines)
│   └── subgraphs.rs    # draw_subgraph (~100 lines)
└── utils.rs            # Shared utilities (~100 lines)
```

### Module Responsibilities

#### `mod.rs` (~80 lines)
- Re-export public API
- Backward compatibility aliases

```rust
pub use types::*;
pub use parser::parse_flowchart;
pub use layout::layout_flowchart;
pub use render::{render_flowchart, FlowchartColors};

// For tests and internal use
pub(crate) use parser::{parse_direction, parse_edge_line_full, parse_node_from_text};
```

#### `types.rs` (~150 lines)
- All AST types: `FlowDirection`, `NodeShape`, `FlowNode`, etc.
- Layout result types: `NodeLayout`, `SubgraphLayout`, `FlowchartLayout`
- No dependencies on egui (use generic color type or Option<[u8; 4]>)

```rust
// types.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlowDirection { ... }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeShape { ... }

// Color can be converted to/from egui::Color32 in render module
#[derive(Debug, Clone, Default)]
pub struct NodeStyle {
    pub fill: Option<[u8; 4]>,    // RGBA
    pub stroke: Option<[u8; 4]>,
    pub stroke_width: Option<f32>,
}
```

#### `parser.rs` (~900 lines)
- `parse_flowchart()` main entry point
- All parsing helpers
- Internal struct `SubgraphBuilder`

#### `layout/mod.rs` (~100 lines)
- `layout_flowchart()` entry point
- Coordinate `compute_subgraph_layouts()`
- Re-export layout types

#### `layout/graph.rs` (~200 lines)
- `FlowGraph` struct and implementation
- Graph construction from flowchart
- Adjacency list management

#### `layout/sugiyama.rs` (~500 lines)
- `SugiyamaLayout` struct
- Cycle detection
- Layer assignment
- Crossing reduction
- Coordinate assignment

#### `layout/subgraph.rs` (~400 lines)
- `SubgraphLayoutEngine`
- `SubgraphInternalLayout`
- Hierarchical subgraph positioning

#### `layout/config.rs` (~50 lines)
- `FlowLayoutConfig` struct
- Default configuration values

#### `render/mod.rs` (~100 lines)
- `render_flowchart()` entry point
- Pre-computation of edge labels
- Drawing orchestration

#### `render/colors.rs` (~70 lines)
- `FlowchartColors` struct
- `dark()` and `light()` presets

#### `render/nodes.rs` (~200 lines)
- `draw_node()` function
- Shape drawing (rectangle, diamond, circle, etc.)

#### `render/edges.rs` (~400 lines)
- `draw_edge()` function
- `draw_arrow_head()`
- `draw_bezier_curve()`
- Subgraph crossing detection

#### `render/subgraphs.rs` (~100 lines)
- `draw_subgraph()`
- `compute_subgraph_depths()`

#### `utils.rs` (~100 lines)
- `draw_dashed_line()`
- `bezier_point()`
- `line_rect_intersection()`
- Geometry helpers

---

## Refactoring Strategy

### Approach: Parallel Development with `_legacy` Suffix

To minimize risk, use a parallel development approach:

1. **Create new module structure** alongside existing file
2. **Copy and adapt code** into new modules
3. **Run tests against both** implementations
4. **Switch imports** in `mod.rs` when ready
5. **Remove legacy file** after verification

### Phase 1: Extract Types (Low Risk)

**Goal**: Create `flowchart/types.rs` with all type definitions.

1. Create `src/markdown/mermaid/flowchart/` directory
2. Create `flowchart/types.rs` with all enums and structs
3. Create `flowchart/mod.rs` with re-exports
4. Update imports in `flowchart.rs` to use new types
5. Verify all tests pass

**Migration Path**:
```rust
// flowchart.rs (temporary)
mod types;
pub use types::*;
// ... rest of file unchanged
```

### Phase 2: Extract Parser (Medium Risk)

**Goal**: Move all parsing functions to `flowchart/parser.rs`.

**Critical: Preserve Task 43, 44 fixes**:
- `strip_trailing_semicolon()` - Task 43
- `split_by_ampersand()` - Task 43
- `parse_edge_line_full()` - Task 43, 44
- `extract_dash_label()` - Task 44
- `ARROW_PATTERNS` ordering - Task 44

1. Create `flowchart/parser.rs`
2. Move parsing functions (keep exact implementations)
3. Update `parse_flowchart()` to use moved functions
4. Run parsing tests specifically
5. Verify edge cases from Tasks 43, 44

**Test Cases to Verify**:
```rust
// Task 43: Semicolon handling
assert_parse("graph TD;\n  A-->B;");
assert_parse("A-->B;");

// Task 43: Ampersand handling  
assert_parse("A & B --> C");
assert_parse("A --> B & C");

// Task 44: Edge parsing
assert_parse("A -->|Yes| B");
assert_parse("A-- label -->B");
```

### Phase 3: Extract Layout (High Risk)

**Goal**: Create `flowchart/layout/` module hierarchy.

**Critical: Preserve Task 46, 57 fixes**:
- `assign_coordinates_with_subgraphs()` - Task 46 (direction handling)
- `compute_subgraph_layouts()` - Task 57 (viewport clipping)

1. Create layout module structure
2. Extract `FlowLayoutConfig` to `layout/config.rs`
3. Extract `FlowGraph` to `layout/graph.rs`
4. Extract `SubgraphLayoutEngine` to `layout/subgraph.rs`
5. Extract `SugiyamaLayout` to `layout/sugiyama.rs`
6. Create `layout/mod.rs` with entry point

**Dependency Order**:
```
config.rs (no deps)
    ↓
graph.rs (depends on config, types)
    ↓
subgraph.rs (depends on graph, config)
    ↓
sugiyama.rs (depends on all above)
    ↓
mod.rs (orchestrates)
```

**Test Cases to Verify**:
```rust
// Task 46: Direction handling
assert_layout_lr("flowchart LR\n  A --> B");
assert_layout_rl("flowchart RL\n  A --> B");
assert_layout_bt("flowchart BT\n  A --> B");

// Task 57: Viewport clipping
assert_no_negative_positions(layout);
assert_subgraph_contains_nodes(layout);
```

### Phase 4: Extract Renderer (Medium Risk)

**Goal**: Create `flowchart/render/` module hierarchy.

1. Create render module structure
2. Extract `FlowchartColors` to `render/colors.rs`
3. Extract `draw_node()` to `render/nodes.rs`
4. Extract `draw_edge()` and helpers to `render/edges.rs`
5. Extract `draw_subgraph()` to `render/subgraphs.rs`
6. Move utility functions to `utils.rs`

### Phase 5: Integration and Cleanup

1. Update `src/markdown/mermaid/mod.rs` imports
2. Run full test suite
3. Manual visual testing of all diagram types
4. Remove `flowchart.rs` legacy file
5. Update documentation

---

## Risk Assessment

### High-Risk Areas

| Area | Risk | Mitigation |
|------|------|------------|
| Task 46 (Direction) | Layout breaks for LR/RL/BT | Keep exact algorithm, comprehensive direction tests |
| Task 57 (Clipping) | Content clipped in viewport | Test nested subgraphs, verify positive coordinates |
| Edge parsing | Chained edges break | Test all ARROW_PATTERNS, verify edge declaration order |

### Recent Fix Locations

| Task | Fix Location | New Module |
|------|--------------|------------|
| 43 | `strip_trailing_semicolon`, `split_by_ampersand`, `parse_direction` | `parser.rs` |
| 44 | `ARROW_PATTERNS`, `find_arrow_pattern`, `extract_dash_label`, `parse_edge_line_full` | `parser.rs` |
| 46 | `assign_coordinates_with_subgraphs` | `layout/sugiyama.rs` |
| 54 | `parse_node_from_text` (asymmetric shape) | `parser.rs` |
| 55 | Title truncation | `layout/mod.rs` (compute_subgraph_layouts) |
| 56 | Nested subgraph layout | `layout/subgraph.rs` |
| 57 | Viewport clipping | `layout/mod.rs` (coordinate shifting) |

---

## Public API Stability

### Current Public Exports (from `mod.rs`)

```rust
// Must remain stable
pub use flowchart::{
    layout_flowchart,      // Main layout function
    parse_flowchart,       // Main parse function
    render_flowchart,      // Main render function
    FlowchartColors,       // Color configuration
    FlowDirection,         // Direction enum
    NodeShape,             // Shape enum
};

// Internal exports for tests
pub(crate) use flowchart::{
    parse_direction,       // Direction parsing
    parse_edge_line_full,  // Edge parsing
    parse_node_from_text,  // Node parsing
};
```

### Proposed API (unchanged signatures)

```rust
// flowchart/mod.rs
pub fn parse_flowchart(source: &str) -> Result<Flowchart, String>;

pub fn layout_flowchart(
    flowchart: &Flowchart,
    available_width: f32,
    font_size: f32,
    text_measurer: &impl TextMeasurer,
) -> FlowchartLayout;

pub fn render_flowchart(
    ui: &mut Ui,
    flowchart: &Flowchart,
    layout: &FlowchartLayout,
    colors: &FlowchartColors,
    font_size: f32,
);
```

---

## Implementation Checklist

### Pre-Refactor
- [ ] Create comprehensive test suite for all edge cases
- [ ] Document current behavior for regression testing
- [ ] Create backup branch

### Phase 1: Types
- [ ] Create `flowchart/` directory
- [ ] Create `types.rs` with all type definitions
- [ ] Create `mod.rs` with re-exports
- [ ] Verify compilation
- [ ] Run all tests

### Phase 2: Parser
- [ ] Create `parser.rs`
- [ ] Move all parsing functions
- [ ] Verify Task 43 semicolon tests pass
- [ ] Verify Task 44 edge parsing tests pass
- [ ] Verify Task 54 asymmetric shape tests pass

### Phase 3: Layout
- [ ] Create `layout/` directory structure
- [ ] Extract `config.rs`
- [ ] Extract `graph.rs`
- [ ] Extract `subgraph.rs`
- [ ] Extract `sugiyama.rs`
- [ ] Create `layout/mod.rs`
- [ ] Verify Task 46 direction tests pass
- [ ] Verify Task 55 title tests pass
- [ ] Verify Task 56 nested subgraph tests pass
- [ ] Verify Task 57 viewport tests pass

### Phase 4: Renderer
- [ ] Create `render/` directory structure
- [ ] Extract `colors.rs`
- [ ] Extract `nodes.rs`
- [ ] Extract `edges.rs`
- [ ] Extract `subgraphs.rs`
- [ ] Create `utils.rs`
- [ ] Visual testing

### Phase 5: Cleanup
- [ ] Update `mermaid/mod.rs` imports
- [ ] Remove legacy `flowchart.rs`
- [ ] Update documentation
- [ ] Final test run

---

## Success Criteria

1. **All existing tests pass** without modification
2. **Visual parity** with current rendering
3. **No performance regression** in layout computation
4. **Clean module boundaries** with minimal cross-dependencies
5. **Improved IDE navigation** and code completion
6. **Preserved git history** for bug tracking

---

## Related Documentation

- [Mermaid Modular Structure](./mermaid-modular-structure.md) - Current module organization
- [Flowchart Layout Algorithm](./flowchart-layout-algorithm.md) - Sugiyama algorithm details
- [Flowchart Subgraphs](./flowchart-subgraphs.md) - Subgraph parsing and layout
- [Flowchart Direction](./flowchart-direction.md) - Task 46 direction handling
- [Flowchart Viewport Clipping](./flowchart-viewport-clipping.md) - Task 57 clipping fix

---

## Appendix: Function Reference

### Parser Functions

| Function | Signature | Used By |
|----------|-----------|---------|
| `parse_flowchart` | `(source: &str) -> Result<Flowchart, String>` | Public API |
| `parse_direction` | `(header: &str) -> FlowDirection` | `parse_flowchart`, tests |
| `parse_edge_line_full` | `(line: &str) -> Option<(Vec<Node>, Vec<Edge>)>` | `parse_flowchart`, tests |
| `parse_node_from_text` | `(text: &str) -> Option<(String, String, NodeShape)>` | `parse_edge_line_full`, tests |
| `parse_subgraph_header` | `(line: &str, counter: &mut usize) -> (String, Option<String>)` | `parse_flowchart` |
| `parse_class_def` | `(line: &str) -> Option<(String, NodeStyle)>` | `parse_flowchart` |
| `parse_class_assignment` | `(line: &str, classes: &mut HashMap)` | `parse_flowchart` |

### Layout Functions

| Function | Signature | Used By |
|----------|-----------|---------|
| `layout_flowchart` | `(flowchart, width, font_size, measurer) -> FlowchartLayout` | Public API |
| `compute_subgraph_layouts` | `(layout, flowchart, config, font_size, measurer)` | `layout_flowchart` |
| `FlowGraph::from_flowchart` | `(flowchart, font_size, measurer, config) -> FlowGraph` | `SugiyamaLayout` |
| `SugiyamaLayout::compute` | `(self) -> FlowchartLayout` | `layout_flowchart` |

### Render Functions

| Function | Signature | Used By |
|----------|-----------|---------|
| `render_flowchart` | `(ui, flowchart, layout, colors, font_size)` | Public API |
| `draw_node` | `(painter, node, layout, offset, colors, font_size, style)` | `render_flowchart` |
| `draw_edge` | `(painter, edge, from, to, offset, colors, font_size, direction, ...)` | `render_flowchart` |
| `draw_subgraph` | `(painter, layout, offset, colors, font_size, depth)` | `render_flowchart` |
