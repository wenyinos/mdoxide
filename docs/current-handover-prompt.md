# Handover: v0.2.7 Features & Polish

## Rules (DO NOT UPDATE)
- Never auto-update this file - only update when explicitly requested
- Run `cargo build` after changes to verify code compiles
- Follow existing code patterns and conventions
- Use Context7 MCP tool to fetch library documentation when needed
- Document by feature (e.g., memory-optimization.md), not by task
- Update docs/index.md when adding new documentation
- **Branch**: `master`

---

## Current Task

**Task 18: Implement native macOS window controls and icon polish**
- **Priority**: Medium
- **Dependencies**: None
- **Status**: Pending
- **Task Master ID**: 18
- **Complexity**: 4

### Description
Add macOS traffic-light controls and ensure consistent icons across themes.

### Implementation Details
1. In `src/ui/window.rs`, configure eframe/egui for native macOS window decoration (close/min/maximize traffic lights)
2. Verify icons scale correctly in light/dark themes following HIG
3. Test cross-platform consistency (Windows/Linux should remain unchanged)

### Key Files

| File | Purpose |
|------|---------|
| `src/ui/window.rs` | Custom window resize handles, borderless window logic |
| `src/platform/macos.rs` | macOS-specific code (app delegate, Open With) |
| `src/ui/icons.rs` | Icon loading for window/taskbar icons |
| `src/main.rs` | eframe setup, window configuration |

### Test Strategy
1. macOS build - native traffic lights visible
2. Icons crisp in both light/dark themes
3. Verify Windows/Linux unchanged

---

## Recently Completed (Previous Sessions)

- **Task 17**: Flowchart modular refactoring (DONE)
  - Split monolithic `flowchart.rs` (3600 lines) into 12 focused modules
  - New structure: `flowchart/types.rs`, `parser.rs`, `layout/` (config, graph, subgraph, sugiyama), `render/` (colors, nodes, edges, subgraphs), `utils.rs`
  - Zero behavior changes, all 83 mermaid tests pass
  - Technical doc: `docs/technical/mermaid/flowchart-modular-refactor.md`

- **Task 27**: Image rendering in rendered/split view (DONE)

- **Task 25**: Single-instance file opening (DONE)

- **Task 16**: Backlinks panel with graph-based indexing (DONE)

- **Task 15**: Wikilinks parsing, resolution, and navigation (DONE)

---

## Environment

- **Project**: Ferrite (Markdown editor)
- **Language**: Rust
- **GUI Framework**: egui 0.28
- **Branch**: `master`
- **Build**: `cargo build`
- **Version**: v0.2.7 (in progress)
