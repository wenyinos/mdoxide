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

**Task 12: Implement GitHub-style callouts parsing and rendering**
- **Priority**: High
- **Dependencies**: None
- **Status**: Pending

### Description
Add support for GitHub-style admonition blocks like `> [!NOTE]` with color-coded rendering and optional collapsible state.

### Implementation Details
1. **Parser** (`src/markdown/parser.rs`): Extend to recognize `> [!TYPE]` and `> [!TYPE] Custom Title` syntax. Store type, title, and collapsed state in AST.
2. **Renderer** (`src/markdown/widgets.rs`): Render styled blocks with icons and colors:
   - NOTE = blue
   - TIP = green
   - WARNING = orange
   - CAUTION = yellow
   - IMPORTANT = red
3. **Collapsible**: Support `> [!NOTE]-` for collapsed-by-default blocks.
4. **Interaction**: Add toggle interaction preserving state per block.

### Test Strategy
1. Parse/render `> [!NOTE]\n> Content` -> blue note block
2. `> [!WARNING] Custom Title` -> orange with custom title
3. `> [!NOTE]-` -> collapsed by default, expands on click
4. Verify all 5 types render correctly with proper colors and icons

---

## Key Files for Task 12

| File | Purpose |
|------|---------|
| `src/markdown/parser.rs` | Markdown parser - extend for callout syntax |
| `src/markdown/widgets.rs` | Markdown rendering widgets - add styled callout blocks |
| `src/markdown/editor.rs` | WYSIWYG rendered editing |
| `src/markdown/mod.rs` | Markdown module exports |

### Reference
- Look at how blockquotes are currently parsed and rendered
- GitHub callout spec: `> [!TYPE]` where TYPE is NOTE, TIP, WARNING, CAUTION, IMPORTANT
- Optional custom title: `> [!TYPE] Custom Title`
- Collapsed-by-default: `> [!TYPE]-`

---

## Recently Completed (This Session)

- **Task 11**: Preload explicit CJK font at startup for restored tabs (DONE)
  - Added `preload_explicit_cjk_font()` in `src/fonts.rs`
  - Updated startup flow in `src/app/mod.rs` to call it before system-locale preload

---

## Environment

- **Project**: Ferrite (Markdown editor)
- **Language**: Rust
- **GUI Framework**: egui 0.28
- **Branch**: `master`
- **Build**: `cargo build`
- **Version**: v0.2.7 (in progress)
