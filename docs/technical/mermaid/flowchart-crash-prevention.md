# Flowchart Crash Prevention

## Overview

This document describes the safety mechanisms implemented to prevent application crashes when rendering malformed or syntactically invalid Mermaid flowchart diagrams.

## Problem

Certain malformed Mermaid syntax (e.g., nested quotes in node labels like `H[Show "Try Again"]`) could cause the flowchart layout algorithm to enter an infinite loop, crashing the entire application. The official Mermaid.js parser shows a syntax error for such input, but our parser was more lenient and attempted to render, leading to pathological graph structures.

## Solution

### 1. Iteration Limit in Layer Assignment

The `assign_internal_layers` function in subgraph layout now has a safety limit:

```rust
// Safety limit to prevent infinite loops on malformed input
let max_iterations = n * n + 100;
let mut iteration = 0;

while let Some(local_i) = queue.pop_front() {
    iteration += 1;
    if iteration > max_iterations {
        // Malformed graph causing infinite loop - return simple single-layer layout
        return vec![nodes.to_vec()];
    }
    // ... normal processing ...
}
```

When the limit is exceeded, the function returns a simple single-layer layout instead of crashing.

### 2. Panic Handler for Flowchart Rendering

The flowchart rendering pipeline is wrapped in `catch_unwind` to gracefully handle any remaining panics:

```rust
use std::panic::{catch_unwind, AssertUnwindSafe};

let result = catch_unwind(AssertUnwindSafe(|| {
    let layout = layout_flowchart(&flowchart, ...);
    render_flowchart(ui, &flowchart, &layout, ...);
    RenderResult::Success
}));

match result {
    Ok(render_result) => render_result,
    Err(panic_info) => {
        let msg = /* extract panic message */;
        RenderResult::ParseError(msg)
    }
}
```

## Key Files

- `src/markdown/mermaid/flowchart.rs` - Contains `assign_internal_layers` with iteration limit
- `src/markdown/mermaid/mod.rs` - Contains `render_mermaid_diagram` with panic handler

## Behavior

| Input Type | Behavior |
|------------|----------|
| Valid Mermaid | Renders normally |
| Syntax error causing cycle | Falls back to single-layer layout, renders |
| Panic during layout/render | Shows error message instead of crashing |

## Testing

To test the crash prevention:
1. Try rendering a flowchart with nested quotes: `F -->|No| H[Show "Try Again"]`
2. The diagram should render (possibly with imperfect layout) instead of crashing
3. More severe errors will show a parse error message

## Related

- [mermaid-edge-parsing.md](mermaid-edge-parsing.md) - Edge parsing details
- [flowchart-linkstyle.md](flowchart-linkstyle.md) - Link styling implementation
