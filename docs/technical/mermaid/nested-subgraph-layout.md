# Nested Subgraph Layout

## Overview

This document describes the implementation of nested subgraph support in Ferrite's flowchart renderer. Mermaid supports subgraphs that can contain other subgraphs, creating a hierarchical structure.

## Key Changes (Task 56)

### 1. Nested Margin Support

Added `nested_subgraph_margin` configuration (10.0 pixels by default) to create proper visual separation between parent and child subgraph boundaries.

When computing subgraph bounds in `compute_subgraph_layouts()`:
- Child subgraph bounds include extra margin around them
- Parent subgraphs with nested children use increased effective padding
- This prevents parent and child borders from appearing too close together

### 2. Accurate Depth Calculation

The `compute_subgraph_depths()` function computes actual nesting depth:
- Depth 0 = top-level subgraph (no parent)
- Depth 1 = child of top-level
- Depth 2 = grandchild, etc.

This enables proper alternating fill colors for visual distinction between nesting levels.

### 3. Subgraph Direction Overrides

Infrastructure added to support per-subgraph direction overrides:
- `FlowGraph.subgraph_directions` stores direction overrides from parsing
- `SubgraphLayoutEngine` uses effective direction for each subgraph
- Example: A TD flowchart can have a subgraph with `direction LR`

## File Structure

```
src/markdown/mermaid/flowchart.rs
├── FlowLayoutConfig
│   └── nested_subgraph_margin: f32  // Extra margin between nested boundaries
├── FlowGraph
│   └── subgraph_directions: HashMap<String, FlowDirection>  // Direction overrides
├── SubgraphLayoutEngine
│   ├── layout_subgraph() - Uses effective direction per subgraph
│   ├── layout_simple_subgraph() - Accepts direction parameter
│   └── layout_hierarchical_subgraph() - Accepts direction parameter
├── compute_subgraph_layouts() - Adds nested margins to bounds
└── compute_subgraph_depths() - Calculates actual nesting depth
```

## Configuration Values

| Parameter | Value | Description |
|-----------|-------|-------------|
| `subgraph_padding` | 15.0 | Standard padding around subgraph content |
| `subgraph_title_height` | 24.0 | Height reserved for title |
| `nested_subgraph_margin` | 10.0 | Extra margin between nested boundaries |

## Test Cases

See `test_md/test_flowcharts.md` for nested subgraph test cases:

1. **True Nested Subgraphs** - Parent-child relationship
2. **Deeply Nested Subgraphs** - 3 levels deep
3. **Nested with Direction Override** - Child uses different direction

## Known Limitations

1. **Internal Layouts Not Applied**: The `SubgraphLayoutEngine` computes internal layouts for subgraphs, but these aren't currently applied to actual node positions. Nodes are positioned by the global Sugiyama algorithm.

2. **Direction Override Visual Effect**: While direction overrides are parsed and stored, the visual effect is limited because internal layouts aren't applied to final positions.

## Future Improvements

- Apply internal subgraph layouts to node positions
- Support treating subgraphs as "super-nodes" in the global layout
- Improve edge routing through multiple nested levels

## Related Documentation

- `flowchart-subgraph-title.md` - Title width expansion (Task 55)
- `subgraph-internal-layout.md` - Internal positioning
