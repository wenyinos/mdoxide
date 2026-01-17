# Subgraph Internal Layout and Positioning

## Overview

This document describes the subgraph internal layout system implemented in the flowchart layout engine. The system ensures subgraph contents are laid out as self-contained units, keeping nodes within a subgraph visually grouped together.

## Problem

When rendering flowcharts with subgraphs, nodes inside a subgraph should:
1. Be positioned together as a cohesive group
2. Have a bounding box that tightly fits the contents
3. Support nested subgraphs where inner subgraphs are contained within outer ones

## Solution

The implementation uses a multi-phase approach:

### Phase 1: Subgraph Internal Layout Computation

A `SubgraphLayoutEngine` computes the internal layout for each subgraph:

1. **Process subgraphs inside-out** (children before parents)
2. **For each subgraph:**
   - Extract nodes that directly belong to it
   - Find internal edges (edges with both endpoints in the subgraph)
   - Run a mini Sugiyama layer assignment on just those nodes
   - Compute positions relative to a (0, 0) origin
   - Calculate bounding box with padding and title height

### Phase 2: Global Layout

The global Sugiyama layout algorithm runs on all nodes:

1. **Subgraph-aware layer assignment** (Task 49) ensures nodes in the same subgraph are assigned to consecutive layers
2. **Crossing reduction** uses barycenter heuristic
3. **Coordinate assignment** positions all nodes globally

### Phase 3: Subgraph Bounds Calculation

After global positioning, `compute_subgraph_layouts()` computes the final bounding boxes:

1. **Process subgraphs in reverse order** (children before parents)
2. **For each subgraph:**
   - Find min/max coordinates of all contained nodes
   - Include nested subgraph bounds
   - Add padding around content
   - Add title height above content

## Implementation Details

### SubgraphInternalLayout

```rust
struct SubgraphInternalLayout {
    /// Positions of nodes relative to subgraph origin (0,0)
    node_positions: HashMap<usize, Pos2>,
    /// Bounding box size (including padding and title)
    bounding_size: Vec2,
    /// Content size (without padding)
    content_size: Vec2,
}
```

### SubgraphLayoutEngine

The engine provides methods to:
- `layout_subgraph()` - Layout a single subgraph's contents
- `layout_simple_subgraph()` - Layout subgraph with only direct nodes
- `layout_hierarchical_subgraph()` - Layout subgraph with nested children
- `assign_internal_layers()` - Layer assignment within subgraph
- `compute_internal_positions()` - Position nodes within subgraph

### Processing Order

Subgraphs are processed in dependency order:
1. **Innermost first** - Children are laid out before parents
2. **Child bounds included** - Parent subgraph bounds include child subgraph bounds

## Algorithm Flow

```
layout_flowchart()
├── Build FlowGraph with subgraph membership info
├── SugiyamaLayout::compute()
│   ├── layout_subgraphs_inside_out()  ← NEW
│   │   └── For each subgraph (children first):
│   │       └── SubgraphLayoutEngine::layout_subgraph()
│   ├── detect_cycles_and_mark_back_edges()
│   ├── assign_layers() (subgraph-aware from Task 49)
│   ├── build_layers()
│   ├── reduce_crossings()
│   └── assign_coordinates_with_subgraphs()
└── compute_subgraph_layouts()
    └── Calculate bounds from positioned nodes
```

## Future Enhancements

The `SubgraphInternalLayout` data structure stores internal positions that can be used for:
- **Edge routing within subgraphs** (Task 52) - Route edges to avoid crossing subgraph boundaries
- **Super-node positioning** - Treat subgraphs as single nodes in parent layout for tighter bounds
- **Independent subgraph directions** - Support different flow directions within subgraphs

## Configuration

Layout configuration for subgraphs is in `FlowLayoutConfig`:

```rust
subgraph_padding: 15.0,      // Padding around subgraph content
subgraph_title_height: 24.0, // Height reserved for title
```

## Testing

Unit tests verify:
- Nodes in a subgraph are positioned together
- Subgraph bounding boxes correctly encompass contents
- Nested subgraphs are properly contained
- External connections work correctly
- Multiple parallel subgraphs are handled

See tests in `src/markdown/mermaid/mod.rs`:
- `test_subgraph_nodes_clustered`
- `test_subgraph_with_external_connections`
- `test_multiple_subgraphs`
- `test_subgraph_internal_layout`
- `test_nested_subgraph_layout`

## Related Documentation

- [Subgraph Layer Clustering](subgraph-layer-clustering.md) - Task 49 implementation
- [Flowchart Layout Algorithm](flowchart-layout-algorithm.md) - Sugiyama algorithm overview

## Related Tasks

- Task 49: Subgraph-aware layer assignment ✅
- Task 50: Subgraph internal layout (this feature) ✅
- Task 51: Subgraph visual styling ✅
- Task 52: Edge routing for subgraphs (pending)
