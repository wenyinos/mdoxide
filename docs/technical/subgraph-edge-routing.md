# Subgraph Edge Routing

This document describes the implementation of edge routing when edges cross subgraph boundaries in Mermaid flowcharts.

## Overview

When edges connect nodes that are in different subgraphs (or when one node is inside a subgraph and the other is outside), the edges need to route cleanly through the subgraph borders rather than crossing through the container arbitrarily.

## Implementation

### Key Functions

#### `find_node_subgraph(node_id, flowchart)`

Finds the innermost subgraph containing a given node.

- **Parameters:**
  - `node_id`: The ID of the node to look up
  - `flowchart`: Reference to the parsed flowchart
- **Returns:** `Option<&FlowSubgraph>` - The subgraph containing the node, or None

```rust
fn find_node_subgraph<'a>(
    node_id: &str,
    flowchart: &'a Flowchart,
) -> Option<&'a FlowSubgraph> {
    for subgraph in &flowchart.subgraphs {
        if subgraph.node_ids.contains(&node_id.to_string()) {
            return Some(subgraph);
        }
    }
    None
}
```

#### `line_rect_intersection(from, to, rect)`

Calculates where a line segment intersects a rectangle's border.

- **Parameters:**
  - `from`: Starting point of the line
  - `to`: Ending point of the line
  - `rect`: The rectangle to check intersection with
- **Returns:** `Option<Pos2>` - The intersection point closest to `from`, or None

The function checks all four sides of the rectangle and returns the closest intersection point.

#### `get_subgraph_crossing_info(from_id, to_id, from_pos, to_pos, flowchart, subgraph_layouts, offset)`

Determines how an edge crosses subgraph boundaries.

- **Returns:** `Option<SubgraphCrossingInfo>` containing:
  - `entry_point`: Where the edge enters a subgraph
  - `exit_point`: Where the edge exits a subgraph

### Crossing Scenarios

1. **Outside to Inside**: An external node connects to a node inside a subgraph
   - Calculates entry point at the subgraph border
   - Routes edge to enter cleanly

2. **Inside to Outside**: A node inside a subgraph connects to an external node
   - Calculates exit point at the subgraph border
   - Routes edge to exit cleanly

3. **Subgraph to Subgraph**: A node in one subgraph connects to a node in a different subgraph
   - Calculates both exit and entry points
   - Routes through both borders

### Edge Routing Strategy

When a crossing is detected, the edge uses orthogonal routing with intermediate waypoints:

```
Start ─── waypoint ─── border crossing ─── waypoint ─── End
```

For **Top-Down / Bottom-Up** flow:
- Horizontal segments connect vertical routing

For **Left-Right / Right-Left** flow:
- Vertical segments connect horizontal routing

## Visual Behavior

| Scenario | Routing |
|----------|---------|
| Same subgraph | Direct line |
| Outside → Inside | Entry at border, then to node |
| Inside → Outside | Exit at border, then to node |
| Subgraph A → Subgraph B | Exit A border → Enter B border |

## Test Cases

See `test_md/test_flowcharts.md` for visual test cases:
- "Edge Routing Across Subgraph Boundaries (TD)"
- "Edge Routing Across Subgraph Boundaries (LR)"
- "Complex Cross-Subgraph Routing"

## Related Documentation

- [Subgraph Internal Layout](subgraph-internal-layout.md) - Task 50
- [Subgraph Layer Clustering](subgraph-layer-clustering.md) - Task 49

## File Locations

| File | Purpose |
|------|---------|
| `src/markdown/mermaid/flowchart.rs` | Contains all routing functions |
