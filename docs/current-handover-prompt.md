# Handover: Build Warnings Cleanup

## Rules
- Never auto-update this file - only update when explicitly requested
- Complete entire task before requesting next instruction
- Run `cargo build` / `cargo check` after changes to verify code compiles
- Follow existing code patterns and conventions
- Update task status via Task Master when starting (`in-progress`) and completing (`done`)
- Use Context7 MCP tool to fetch library documentation when needed
- Document by feature (e.g., `session-restore-settings.md`), not by task
- Update `docs/index.md` when adding new documentation
- **Use MCP tools** for Task Master operations, not CLI
- **Avoid `git diff`** - causes disconnections

---

## Current Task

**Build Warnings Cleanup - Pre-Release Code Review**

- **Status**: pending
- **Priority**: medium
- **Goal**: Review and resolve all 61 compiler warnings before release

### Description
The codebase has accumulated 61 compiler warnings (unused imports, dead code, unused fields, etc.). Before release, we need to review each warning and either:
1. **Remove** truly dead code that was planned but never implemented
2. **Use** code that was implemented but the calling code is missing
3. **Suppress** warnings for intentionally unused code with `#[allow(dead_code)]` + comment explaining why

### Warning Categories (from `cargo check`)

#### 1. Unused Imports (10 warnings)
Files: `mermaid/mod.rs`, `markdown/mod.rs`, `ui/mod.rs`
- Review if these exports are needed for public API or can be removed

#### 2. Dead Code - Never Constructed/Used (51 warnings)
**High Priority - Likely removable:**
- `src/app.rs`: `KeyboardAction` variants (Undo, Redo, MoveLineUp, MoveLineDown)
- `src/app.rs`: `get_opening_bracket`, `scroll_to_line` methods
- `src/config/snippets.rs`: Multiple `SnippetConfig` and `SnippetManager` methods
- `src/editor/matching.rs`: `DelimiterKind`, `DelimiterToken`, `MatchingPair` methods
- `src/editor/outline.rs`: `ContentType::label`
- `src/editor/stats.rs`: `DocumentStats::new`, `total_headings`
- `src/markdown/csv_viewer.rs`: Multiple functions and fields
- `src/markdown/toc.rs`: `TocOptions` methods, `remove_toc`
- `src/path_utils.rs`: `normalize_path_ref`, `canonicalize_*` functions
- `src/ui/outline_panel.rs`: `set_active_tab`
- `src/ui/ribbon.rs`: Multiple `RibbonAction` variants
- `src/ui/view_segment.rs`: Constants and `ViewModeSegment`
- `src/vcs/git.rs`: Multiple `GitService` methods

**Mermaid diagram code (may be WIP):**
- `src/markdown/mermaid/*.rs`: Multiple structs and fields

### Review Process

For each warning:
1. **Search for usages** - Is it called anywhere? Was calling code removed?
2. **Check git history** - Was it recently added? Part of incomplete feature?
3. **Decide action**:
   - If truly unused and not part of public API → **Remove**
   - If part of incomplete feature to keep → **Add `#[allow(dead_code)]` with TODO comment**
   - If should be used but isn't → **Investigate and fix**

### Implementation Strategy

1. Start with `src/app.rs` - highest impact file
2. Move to utility modules (`path_utils.rs`, `editor/*.rs`)
3. Handle `mermaid/` carefully - may be WIP diagrams
4. Clean up `csv_viewer.rs` - has many unused items
5. Finish with UI modules

### Test Strategy

1. After each file cleanup, run `cargo check` - ensure no new errors
2. Run `cargo test` periodically to ensure no regressions
3. Final `cargo build --release` should have zero or minimal warnings

---

## Key Files (by warning count)

| File | Warnings | Notes |
|------|----------|-------|
| `src/markdown/mermaid/*.rs` | ~15 | Diagram rendering - may be WIP |
| `src/markdown/csv_viewer.rs` | ~8 | CSV viewer features |
| `src/config/snippets.rs` | ~10 | Snippet expansion system |
| `src/app.rs` | ~5 | Main app, keyboard actions |
| `src/editor/matching.rs` | ~6 | Bracket matching |
| `src/ui/*.rs` | ~8 | UI components |
| `src/vcs/git.rs` | ~5 | Git integration |
| `src/path_utils.rs` | ~3 | Path utilities |

---

## Environment
- **Project**: Ferrite (Markdown editor)
- **Language**: Rust
- **GUI Framework**: egui
- **Version**: 0.2.6

---

## Quick Start
```bash
# Check all warnings
cargo check 2>&1 | grep "warning:"

# Build and run
cargo run

# Run tests
cargo test
```

Or use MCP tools: `get_task`, `set_task_status`, `next_task`
