# Mermaid Modular Structure

## Overview

The native Mermaid diagram rendering implementation has been refactored from a single monolithic file into a modular directory structure. This improves maintainability, reduces compilation times, and makes the codebase easier to navigate.

## Key Files

```
src/markdown/mermaid/
├── mod.rs           # Public API, diagram type dispatcher, unit tests
├── text.rs          # TextMeasurer trait and implementations
├── utils.rs         # Shared drawing utilities
├── flowchart.rs     # Flowchart/graph diagrams
├── sequence.rs      # Sequence diagrams with control-flow blocks
├── pie.rs           # Pie charts
├── state.rs         # State diagrams with composite states
├── mindmap.rs       # Mindmap diagrams
├── class_diagram.rs # UML class diagrams
├── er_diagram.rs    # Entity-Relationship diagrams
├── git_graph.rs     # Git commit visualization
├── gantt.rs         # Gantt charts
├── timeline.rs      # Timeline diagrams
└── journey.rs       # User journey diagrams
```

## Architecture

### Module Responsibilities

| Module | Purpose |
|--------|---------|
| `mod.rs` | Public API entry point. Contains `render_mermaid_diagram()` which dispatches to the appropriate diagram renderer based on the first line of source |
| `text.rs` | Backend-agnostic text measurement via the `TextMeasurer` trait. Includes `EguiTextMeasurer` for runtime and `EstimatedTextMeasurer` for tests |
| `utils.rs` | Shared drawing utilities like `draw_dashed_line()` and `bezier_point()` used by multiple diagram types |

### Each Diagram Module Contains

1. **AST Types** - Structs representing the parsed diagram structure
2. **Parser Function** - `parse_*()` converts source text to AST
3. **Renderer Function** - `render_*()` draws the diagram using egui

### Example: Adding a New Diagram Type

```rust
// In new_diagram.rs
pub struct NewDiagram { ... }

pub fn parse_new_diagram(source: &str) -> Result<NewDiagram, String> {
    // Parse the source
}

pub fn render_new_diagram(ui: &mut Ui, diagram: &NewDiagram, dark_mode: bool, font_size: f32) {
    // Render using egui painter
}
```

Then in `mod.rs`:
1. Add `mod new_diagram;`
2. Add `use new_diagram::{parse_new_diagram, render_new_diagram};`
3. Add a match arm in `render_mermaid_diagram()`

## Supported Diagram Types

| Type | Keyword | Description |
|------|---------|-------------|
| Flowchart | `flowchart`, `graph` | Nodes and edges with various shapes, supports TD/TB/LR/RL/BT directions |
| Sequence | `sequenceDiagram` | Participants and message flows with loop/alt/opt/par blocks |
| Pie | `pie` | Simple pie charts with labels and percentages |
| State | `stateDiagram` | State machines with composite states and transitions |
| Mindmap | `mindmap` | Hierarchical mind maps |
| Class | `classDiagram` | UML class diagrams with relationships |
| ER | `erDiagram` | Entity-relationship diagrams with cardinality |
| Git | `gitGraph` | Git commit history with branches and merges |
| Gantt | `gantt` | Project timeline charts |
| Timeline | `timeline` | Event timelines |
| Journey | `journey` | User experience journey maps |

## Text Measurement

The `TextMeasurer` trait enables backend-agnostic text measurement:

```rust
pub trait TextMeasurer {
    fn measure(&self, text: &str, font_size: f32) -> TextSize;
    fn row_height(&self, font_size: f32) -> f32;
    fn measure_wrapped(&self, text: &str, font_size: f32, max_width: f32) -> TextSize;
    fn truncate_with_ellipsis(&self, text: &str, font_size: f32, max_width: f32) -> String;
}
```

- `EguiTextMeasurer` - Uses egui's font system for accurate measurement at runtime
- `EstimatedTextMeasurer` - Character-based estimation for unit tests without UI context

## Usage

```rust
use crate::markdown::mermaid::{render_mermaid_diagram, RenderResult};

let source = "flowchart TD\n  A[Start] --> B[End]";
match render_mermaid_diagram(ui, source, dark_mode, font_size) {
    RenderResult::Success => { /* Rendered OK */ }
    RenderResult::ParseError(e) => { /* Handle parse error */ }
}
```

## Testing

Unit tests are in `mod.rs` under `#[cfg(test)] mod tests`. Run with:

```bash
cargo test mermaid
```

Key test coverage:
- Flowchart parsing (nodes, edges, shapes, chained edges)
- Direction parsing (TD, LR, RL, BT)
- Layout validation (node positions)
- Text measurer trait implementation
