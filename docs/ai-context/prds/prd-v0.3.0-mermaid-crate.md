# PRD: Mermaid Diagram Rendering Crate

## Product Overview

Extract Ferrite's native MermaidJS diagram renderer into a standalone, pure-Rust crate that can generate diagrams as SVG, PNG, or render directly to GUI frameworks like egui.

## Problem Statement

The Rust ecosystem lacks a comprehensive, pure-Rust solution for rendering Mermaid diagrams. Current options require Node.js (mermaid-cli), only support single diagram types (Pisnge for pie charts), or only generate syntax without rendering. This forces Rust projects to either:
- Depend on Node.js installation
- Use client-side JavaScript
- Go without diagram support

## Target Users

1. Static site generator developers (mdBook, Zola, Cobalt)
2. Documentation tool authors
3. CLI tool developers who need diagram generation
4. GUI application developers (egui, iced, druid)
5. WASM application developers

## Goals

1. Create the first comprehensive pure-Rust Mermaid graphical renderer
2. Support all major Mermaid diagram types (11 types)
3. Provide multiple output backends (SVG, PNG, egui)
4. Zero Node.js or JavaScript dependencies
5. Easy integration for Rust projects

## Non-Goals

1. 100% MermaidJS syntax compatibility (aim for 90%+ of common usage)
2. Interactive diagrams (static rendering only)
3. Custom diagram types beyond Mermaid spec
4. Real-time collaborative editing

## Features

### Core Features (P0 - Must Have)

1. **Diagram Parsing**
   - Flowchart (TD, TB, LR, RL, BT directions)
   - Sequence Diagram
   - Class Diagram
   - State Diagram
   - Entity-Relationship Diagram
   - Pie Chart
   - Gantt Chart
   - Git Graph
   - Mindmap
   - Timeline
   - User Journey

2. **SVG Output**
   - Generate valid SVG strings
   - Proper viewBox and sizing
   - Embedded fonts or system font fallback
   - Works in all major browsers

3. **Error Handling**
   - Graceful handling of invalid syntax
   - Informative error messages with line/column
   - No panics on any input

4. **Basic Theming**
   - Light and dark color schemes
   - Configurable colors

### Secondary Features (P1 - Should Have)

5. **egui Backend**
   - Render directly to egui Painter
   - Widget wrapper for easy integration
   - Theme integration with egui Visuals

6. **PNG Output**
   - Convert SVG to PNG via resvg
   - Configurable resolution/DPI
   - Transparent background option

7. **Comprehensive Documentation**
   - Supported syntax reference
   - Examples for each diagram type
   - Integration guides

### Future Features (P2 - Nice to Have)

8. **Additional Backends**
   - iced integration
   - wgpu direct rendering
   - PDF output

9. **Advanced Features**
   - Custom themes (import/export)
   - Subgraph support improvements
   - Click regions for interactivity metadata

## Technical Requirements

### Architecture

The crate must use a backend-agnostic architecture:

```
mermaid source → Parser → AST → Layout → RenderBackend → Output
```

Where RenderBackend is a trait with implementations for:
- SVG (string output)
- egui (Painter integration)
- PNG (via SVG + resvg)

### Dependencies

Core (always included):
- None beyond std

Optional (feature-gated):
- egui (for egui backend)
- resvg (for PNG output)
- serde (for AST serialization)

### Performance Targets

- Parse + render flowchart with 50 nodes: < 10ms
- Memory usage: < 10MB for large diagrams
- SVG output size: reasonable (not excessively verbose)

### Compatibility

- Rust 1.70+ (match Ferrite's MSRV)
- All tier-1 platforms (Windows, Linux, macOS)
- WASM compatible (for SVG backend)

## Implementation Phases

### Phase 1: Polish in Ferrite (2-3 weeks)
Complete and test all diagram types within Ferrite. Add error handling and basic theming.

### Phase 2: Abstract Backend (2 weeks)
Create RenderBackend trait and refactor current code to use it. Verify Ferrite still works.

### Phase 3: SVG Backend (2 weeks)
Implement SVG output. Create CLI example. Test in browsers.

### Phase 4: Extract Crate (1-2 weeks)
Create separate repository. Move code. Set up CI. Write documentation.

### Phase 5: PNG & Release (1-2 weeks)
Add PNG output. Polish. Publish to crates.io.

### Phase 6: Ferrite Integration (1 week)
Replace Ferrite's internal mermaid.rs with the external crate.

## Success Metrics

1. All 11 diagram types render correctly
2. SVG output validates and displays in Chrome, Firefox, Safari
3. egui backend matches current Ferrite quality
4. 100+ crates.io downloads in first month
5. Positive community reception (Reddit, HN)

## Risks

1. **Scope creep** - Limit to MermaidJS parity, don't invent new features
2. **Text measurement** - Different across backends; provide sensible defaults
3. **Competition** - Move quickly to establish presence
4. **Maintenance burden** - Clear scope, good documentation

## Open Questions

1. Crate name: `mermaid-render`, `chartlet`, or other?
2. Should we support Mermaid config directives?
3. How to handle fonts across different backends?
4. License: MIT to match Ferrite, or dual MIT/Apache-2.0?

## Appendix: Current Implementation Stats

- Lines of code: ~4000
- Diagram types: 11
- Node shapes: 10+ for flowchart
- Edge styles: 3 (solid, dotted, thick)
- Arrow types: 4 (arrow, circle, cross, none)
