# Flowchart Subgraph Title Width Fix

## Overview

This document describes the fix for subgraph title truncation in Mermaid flowcharts. Prior to this fix, subgraph titles would be truncated when they exceeded the content width.

## Problem

Subgraph titles were being cut off when rendered. For example:
- `'Outer Group'` displayed as `'Outer Grou...'`
- `'Inner Group'` showed similar truncation

The root cause was that the subgraph bounding box calculation only considered the content (nodes and child subgraphs) when determining width, without accounting for the title text width.

## Solution

Modified `compute_subgraph_layouts()` in `src/markdown/mermaid/flowchart.rs` to:

1. **Measure title text width** using the `TextMeasurer` trait
2. **Expand subgraph width** if the title requires more space than the content provides
3. **Apply consistent padding** (12px left + 12px right = 24px total) around the title

### Key Changes

The function signature was updated to accept text measurement capabilities:

```rust
fn compute_subgraph_layouts(
    layout: &mut FlowchartLayout,
    flowchart: &Flowchart,
    config: &FlowLayoutConfig,
    font_size: f32,           // NEW
    text_measurer: &impl TextMeasurer,  // NEW
)
```

Title width check is applied in two places:
1. **Pre-computed layouts**: After updating the title from the flowchart
2. **Fallback path**: After computing bounds from node positions

```rust
// Ensure subgraph width accommodates the title text
if let Some(title) = &subgraph.title {
    let title_text_size = text_measurer.measure(title, font_size);
    let min_width_for_title = title_text_size.width + 24.0;  // 12px padding each side
    if existing.size.x < min_width_for_title {
        existing.size.x = min_width_for_title;
    }
}
```

## Testing

Two new tests verify the fix:

1. **`test_subgraph_title_width_expansion`**: Verifies that long titles expand the subgraph width
2. **`test_subgraph_short_title_no_extra_expansion`**: Verifies that short titles don't unnecessarily expand subgraphs

### Test Case Example

```rust
let source = r#"flowchart TD
    subgraph veryLongTitle[This Is A Very Long Subgraph Title]
        A[Small] --> B[Node]
    end"#;
```

The test verifies that `sg_layout.size.x >= title_width + 24.0`.

## Related Files

| File | Purpose |
|------|---------|
| `src/markdown/mermaid/flowchart.rs` | Layout computation and rendering |
| `src/markdown/mermaid/text.rs` | Text measurement traits |

## Related Documentation

- [Flowchart Subgraphs](./flowchart-subgraphs.md) - General subgraph parsing and layout
- [Subgraph Internal Layout](./subgraph-internal-layout.md) - Internal node positioning
- [Mermaid Text Measurement](./mermaid-text-measurement.md) - Text measurement utilities
