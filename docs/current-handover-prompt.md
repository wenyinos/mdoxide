# Handover: Scroll & Navigation Accuracy Fixes

## Rules
- Never auto-update this file - only update when explicitly requested
- Complete entire task before requesting next instruction
- Run `cargo build` / `cargo check` after changes to verify code compiles
- Follow existing code patterns and conventions
- Update task status via Task Master when starting (`in-progress`) and completing (`done`)
- Use Context7 MCP tool to fetch library documentation when needed
- Document by feature (e.g., `scroll-accuracy.md`), not by task
- Update `docs/index.md` when adding new documentation
- **Use MCP tools** for Task Master operations, not CLI
- **Avoid `git diff`** - causes disconnections

---

## Current Task

**Scroll & Navigation Accuracy Fixes - Critical Hotfix for v0.2.5.1**

- **Status**: pending
- **Priority**: high
- **Goal**: Fix scroll positioning accuracy issues in find, search-in-files, semantic minimap, and outline navigation

### Problem Description

In large files (3000+ lines), when jumping to search results or clicking in semantic minimap/outline:
1. **Scroll position is off** - Target line is not centered in viewport, sometimes out of view entirely
2. **Highlight is correct** - The text is highlighted at the right position, but scroll puts it in wrong place
3. **Cumulative error** - Error magnifies in large files (at line 2000, can be 1000+ pixels off)

### Root Causes Identified

1. **Off-by-one inconsistencies** - Different functions use 0-indexed vs 1-indexed line numbers inconsistently
2. **Stale `raw_line_height`** - Uses default 20.0 or outdated value instead of actual line height
3. **Multiple scroll calculation methods** - 3 different approaches used across codebase
4. **No unified scroll function** - Each feature implements its own scroll logic

### Implementation Plan

#### Phase 1: Unify Scroll Calculations (Primary Fix)

**Create unified scroll function in `src/app.rs`:**

```rust
/// Calculate accurate scroll offset to center a target line in viewport.
/// Uses actual galley positioning when available, falls back to line_height calculation.
fn calculate_scroll_for_target_line(
    target_line: usize,  // 1-indexed
    line_height: f32,
    viewport_height: f32,
) -> f32 {
    // Convert to 0-indexed for calculation
    let line_index = target_line.saturating_sub(1);
    let target_y = line_index as f32 * line_height;
    // Position target at 1/3 from top (better visibility than center)
    (target_y - viewport_height / 3.0).max(0.0)
}
```

#### Phase 2: Fix Individual Locations

**File: `src/app.rs`**

1. **`navigate_to_heading()` (~line 6450)** - Fix off-by-one error:
   ```rust
   // BEFORE (bug):
   let target_scroll = (target_line as f32 * line_height) - (viewport_height / 3.0);
   
   // AFTER (fixed):
   let target_scroll = calculate_scroll_for_target_line(target_line + 1, line_height, viewport_height);
   // OR: Use (target_line as f32 - 1.0) since target_line is 0-indexed here
   ```

2. **`handle_search_navigation()` (~line 3990)** - Use unified function

**File: `src/editor/widget.rs`**

3. **`scroll_to_line` handling (~line 597)** - Already correct (uses `saturating_sub(1)`)

4. **Search highlight scrolling (~line 608)** - Verify consistency

**File: `src/markdown/editor.rs`**

5. **`scroll_to_line` in rendered mode (~line 664)** - Ensure matches raw mode calculation

#### Phase 3: Ensure Fresh Line Height

**In `navigate_to_heading()` and similar:**
- Check if `raw_line_height` is still default (20.0) 
- If so, use a sensible fallback based on font_size setting
- Consider: `line_height = settings.font_size * 1.4` as reasonable estimate

### Files to Modify

| File | Changes |
|------|---------|
| `src/app.rs` | Add unified scroll function, fix `navigate_to_heading()`, fix search navigation |
| `src/editor/widget.rs` | Verify scroll calculations are consistent |
| `src/markdown/editor.rs` | Ensure rendered mode scroll matches raw mode |

### Test Strategy

1. **Large file test** - Create/use a markdown file with 3000+ lines
2. **Test find function** - Search for text at lines 100, 1000, 2000, 2500
3. **Test outline panel** - Click headings at various positions
4. **Test semantic minimap** - Click items throughout document
5. **Test search-in-files** - Navigate to results in large file

**Success criteria:** Target line should be visible and roughly centered (within ~50px of intended position).

### Specific Bug Locations

| Location | Line | Issue |
|----------|------|-------|
| `navigate_to_heading()` | app.rs:6450 | Uses `target_line` directly without -1 adjustment |
| `navigate_to_heading()` | app.rs:6448 | Uses potentially stale `raw_line_height` |
| Minimap output | minimap.rs:803 | Outputs `item.line` which is 1-indexed |
| Outline output | outline_panel.rs:381 | Outputs `item.line` which is 1-indexed |

---

## Key Files Reference

| File | Purpose |
|------|---------|
| `src/app.rs` | Main app, `navigate_to_heading()`, search navigation |
| `src/editor/widget.rs` | EditorWidget, scroll handling, highlight rendering |
| `src/editor/minimap.rs` | SemanticMinimap click handling |
| `src/editor/outline.rs` | OutlineItem extraction, char_offset calculation |
| `src/ui/outline_panel.rs` | Outline panel click handling |
| `src/markdown/editor.rs` | Rendered mode editor, scroll handling |
| `src/state.rs` | Tab state including `raw_line_height`, `viewport_height` |

---

## Environment
- **Project**: Ferrite (Markdown editor)
- **Language**: Rust
- **GUI Framework**: egui
- **Version**: 0.2.5.1 (hotfix)

---

## Quick Start
```bash
# Build and run
cargo run

# Test with large file
cargo run -- test_md/large_test_file.md

# Run tests
cargo test
```

Or use MCP tools: `get_task`, `set_task_status`, `next_task`

---

## Related Documentation
- [Search Highlight](docs/technical/editor/search-highlight.md)
- [Semantic Minimap](docs/technical/editor/semantic-minimap.md)
- [Galley Cursor Positioning](docs/technical/editor/galley-cursor-positioning.md)
- [Find and Replace](docs/technical/editor/find-replace.md)

## Known Limitation
Pixel-perfect accuracy is blocked by egui's architecture. The custom editor widget planned for v0.3.0 will fully resolve this. This hotfix aims for ~80% improvement in scroll accuracy.
