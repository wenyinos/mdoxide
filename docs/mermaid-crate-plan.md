# Mermaid Crate Extraction Plan

> **Project:** Extract Ferrite's Mermaid renderer into a standalone Rust crate  
> **Status:** Planning  
> **Created:** 2025-01-10

## Executive Summary

Extract and generalize Ferrite's native MermaidJS diagram renderer (~4000 lines) into a standalone, backend-agnostic Rust crate. This would be the **first comprehensive pure-Rust Mermaid graphical renderer** in the ecosystem.

---

## Current State

### What We Have (in `src/markdown/mermaid.rs`)

- **11 diagram types** rendered natively:
  - Flowchart (TD, TB, LR, RL, BT)
  - Sequence Diagram
  - Pie Chart
  - State Diagram
  - Mindmap
  - Class Diagram
  - Entity-Relationship (ER)
  - Git Graph
  - Gantt Chart
  - Timeline
  - User Journey

- **Pure Rust implementation** - no JavaScript, no Puppeteer
- **~4000 lines** of parsing and rendering code
- **Tightly coupled to egui** - uses egui Painter primitives directly

### Limitations to Address

1. **egui dependency** - Can't be used outside egui apps
2. **No SVG output** - Can't generate static files
3. **No PNG output** - Can't export images
4. **Incomplete diagram support** - Some diagram types have partial coverage
5. **Limited styling options** - Hardcoded colors/fonts

---

## Market Analysis

### Existing Rust Solutions

| Crate | Coverage | Output | Limitation |
|-------|----------|--------|------------|
| Pisnge | Pie only | SVG/PNG | Single diagram type |
| Mermaid Builder | N/A | Syntax only | No rendering |
| MermaidParser | Class only | N/A | Parser only |
| mermaid-cli | All | All | Requires Node.js |

**Gap:** No pure-Rust crate renders multiple Mermaid diagram types to graphics.

### Target Users

1. **Static site generators** (mdBook, Zola, Cobalt) - embed diagrams without JS
2. **Documentation tools** - generate docs with diagrams in CI
3. **CLI tools** - create diagrams without Node.js dependency
4. **GUI applications** - embed diagrams (egui, iced, others)
5. **WASM applications** - browser apps without JS runtime overhead

---

## Architecture

### Proposed Crate Structure

```
mermaid-render/                    # or: chartlet, diagrammer, etc.
├── Cargo.toml
├── src/
│   ├── lib.rs                     # Public API
│   ├── parser/                    # Diagram parsing
│   │   ├── mod.rs
│   │   ├── flowchart.rs
│   │   ├── sequence.rs
│   │   ├── pie.rs
│   │   ├── state.rs
│   │   ├── mindmap.rs
│   │   ├── class.rs
│   │   ├── er.rs
│   │   ├── gitgraph.rs
│   │   ├── gantt.rs
│   │   ├── timeline.rs
│   │   └── journey.rs
│   ├── layout/                    # Layout algorithms
│   │   ├── mod.rs
│   │   ├── graph.rs               # DAG layout (flowchart, etc.)
│   │   ├── linear.rs              # Linear layout (sequence, timeline)
│   │   └── radial.rs              # Radial layout (pie, mindmap)
│   ├── render/                    # Rendering backends
│   │   ├── mod.rs                 # RenderBackend trait
│   │   ├── primitives.rs          # Abstract drawing primitives
│   │   ├── svg.rs                 # SVG output (feature: svg)
│   │   ├── egui.rs                # egui integration (feature: egui)
│   │   └── png.rs                 # PNG via resvg (feature: png)
│   ├── style/                     # Theming and colors
│   │   ├── mod.rs
│   │   ├── theme.rs               # Theme definitions
│   │   └── colors.rs              # Color palettes
│   └── types.rs                   # Shared types
├── examples/
│   ├── svg_output.rs
│   ├── egui_widget.rs
│   └── cli_tool.rs
└── tests/
```

### Core Abstraction: RenderBackend Trait

```rust
pub trait RenderBackend {
    type Context;
    
    fn draw_rect(&mut self, ctx: &mut Self::Context, rect: Rect, style: &ShapeStyle);
    fn draw_rounded_rect(&mut self, ctx: &mut Self::Context, rect: Rect, radius: f32, style: &ShapeStyle);
    fn draw_circle(&mut self, ctx: &mut Self::Context, center: Point, radius: f32, style: &ShapeStyle);
    fn draw_line(&mut self, ctx: &mut Self::Context, from: Point, to: Point, style: &LineStyle);
    fn draw_path(&mut self, ctx: &mut Self::Context, points: &[Point], style: &LineStyle);
    fn draw_text(&mut self, ctx: &mut Self::Context, pos: Point, text: &str, style: &TextStyle);
    fn draw_arrow(&mut self, ctx: &mut Self::Context, from: Point, to: Point, style: &ArrowStyle);
    
    fn measure_text(&self, text: &str, style: &TextStyle) -> Size;
}
```

### Feature Flags

```toml
[features]
default = ["svg"]
svg = []                           # SVG string output
egui = ["dep:egui"]               # egui Painter backend
png = ["svg", "dep:resvg"]        # PNG via SVG→resvg
serde = ["dep:serde"]             # Serialize/deserialize AST
```

---

## Implementation Phases

### Phase 1: Polish in Ferrite (2-3 weeks)

**Goal:** Improve current implementation while still in Ferrite

- [ ] Complete partial diagram implementations
- [ ] Improve layout algorithms (better spacing, edge routing)
- [ ] Add comprehensive error handling
- [ ] Add missing node shapes and edge styles
- [ ] Improve text measurement and wrapping
- [ ] Add basic theming support
- [ ] Write tests for each diagram type
- [ ] Document supported syntax for each diagram

**Deliverable:** Production-quality Mermaid rendering in Ferrite v0.3.0

### Phase 2: Abstract Rendering Backend (2 weeks)

**Goal:** Decouple from egui without breaking Ferrite

- [ ] Define `RenderBackend` trait with drawing primitives
- [ ] Define style types (ShapeStyle, LineStyle, TextStyle, ArrowStyle)
- [ ] Create `EguiBackend` implementing the trait
- [ ] Refactor all renderers to use `RenderBackend` instead of egui directly
- [ ] Verify Ferrite still works identically
- [ ] Add benchmarks for rendering performance

**Deliverable:** Backend-agnostic rendering in Ferrite

### Phase 3: SVG Backend (2 weeks)

**Goal:** Enable static diagram generation

- [ ] Implement `SvgBackend` producing SVG strings
- [ ] Handle text measurement without runtime fonts
- [ ] Add SVG-specific optimizations (viewBox, precision)
- [ ] Create CLI example for SVG output
- [ ] Test SVG output in browsers and documentation tools

**Deliverable:** Generate SVG diagrams from Mermaid source

### Phase 4: Extract to Separate Crate (1-2 weeks)

**Goal:** Create standalone crate

- [ ] Create new repository/crate structure
- [ ] Move parser, layout, and render modules
- [ ] Remove all Ferrite-specific dependencies
- [ ] Set up CI/CD (tests, clippy, formatting)
- [ ] Write comprehensive documentation
- [ ] Create examples for each backend
- [ ] Choose crate name and reserve on crates.io

**Deliverable:** Publishable crate with SVG and egui backends

### Phase 5: PNG & Polish (1-2 weeks)

**Goal:** Complete the crate for release

- [ ] Implement PNG output via resvg
- [ ] Add more themes (dark, light, forest, etc.)
- [ ] Performance optimization
- [ ] Fuzzing/property testing for parser
- [ ] Write README with examples
- [ ] Publish to crates.io

**Deliverable:** v0.1.0 release on crates.io

### Phase 6: Update Ferrite Integration (1 week)

**Goal:** Ferrite uses the external crate

- [ ] Add crate as dependency in Ferrite
- [ ] Remove internal mermaid.rs module
- [ ] Verify all functionality preserved
- [ ] Update Ferrite documentation

**Deliverable:** Ferrite v0.3.x using external mermaid crate

---

## Development Strategy

### Where to Develop

| Phase | Location | Reason |
|-------|----------|--------|
| 1-2 | Ferrite repo | Test with real app, iterate quickly |
| 3 | Ferrite repo | Still need Ferrite for testing egui |
| 4-5 | New repo | Clean separation, proper crate structure |
| 6 | Ferrite repo | Integration work |

### Testing Strategy

1. **Visual regression tests** - Render diagrams, compare output
2. **Parser tests** - Test each diagram type's syntax
3. **Layout tests** - Verify node positions are reasonable
4. **Backend parity tests** - SVG and egui produce equivalent results
5. **Fuzzing** - Parser shouldn't panic on any input

---

## Naming Considerations

| Option | Pros | Cons |
|--------|------|------|
| `mermaid-render` | Clear purpose | Might confuse with official |
| `chartlet` | Unique, memorable | Not obvious what it does |
| `diagrammer` | Descriptive | Generic |
| `flowrender` | Clear | Doesn't cover all diagram types |
| `mermaid-egui` | Specific | Too narrow if we add SVG/PNG |

**Recommendation:** `mermaid-render` or `chartlet`

---

## Success Criteria

### For v0.1.0 Release

- [ ] All 11 diagram types parse and render
- [ ] SVG output works in all major browsers
- [ ] egui backend maintains current Ferrite quality
- [ ] Documentation covers all supported syntax
- [ ] At least 80% test coverage for parser
- [ ] No panics on malformed input

### For Ecosystem Impact

- [ ] mdBook plugin using the crate
- [ ] At least 100 downloads in first month
- [ ] Positive feedback from Rust community
- [ ] No competing pure-Rust solution emerges

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Scope creep (more diagram types) | High | Stick to MermaidJS parity, defer extensions |
| Text measurement differences | Medium | Provide sensible defaults, allow configuration |
| Performance regression | Medium | Benchmark early, profile regularly |
| Another crate emerges first | Medium | Move quickly on initial release |
| Mermaid syntax changes | Low | Version-pin supported syntax |

---

## Related Documents

- [Modular Refactor Plan](./refactor.md) - Ferrite's feature-flag architecture
- [Custom Editor Plan](./technical/planning/custom-editor-widget-plan.md) - v0.3.0 editor work

---

## Changelog

| Date | Change |
|------|--------|
| 2025-01-10 | Initial planning document |
