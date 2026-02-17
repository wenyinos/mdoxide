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

**Task 27: Fix images not displaying in rendered markdown view**
- **Priority**: High
- **Dependencies**: None
- **Status**: Pending
- **Task Master ID**: 27
- **Complexity**: 5

### Description
Images in markdown (`![](path)`) do not show in rendered/split view. Fix so images display correctly.

### Implementation Details
1. In `src/markdown/` (editor.rs or widgets.rs), resolve image paths relative to document location
2. Load image bytes, render with egui Image
3. Support common formats (PNG, JPEG, GIF, WebP)
4. Handle missing files and unsupported formats gracefully (placeholder or alt text)
5. Ensure path resolution uses current file directory and workspace root

### Key Files

| File | Purpose |
|------|---------|
| `src/markdown/editor.rs` | WYSIWYG markdown editor — image rendering |
| `src/markdown/widgets.rs` | Editable heading/list/table widgets — may need image widget |
| `src/markdown/parser.rs` | Comrak integration — image node in AST |
| `src/state.rs` | Tab state — file path for relative resolution |

### Test Strategy
1. Rendered view: `![](./img.png)` and `![](assets/logo.png)` show image
2. Missing file → placeholder or alt text
3. Split view shows images correctly

---

## Recently Completed (Previous Sessions)

- **Task 25**: Single-instance file opening (DONE)
  - Lock file + TCP IPC protocol in `src/single_instance.rs`
  - Double-click in OS forwards path to existing window via local TCP
  - Polls listener each frame (non-blocking), opens received paths as tabs
  - Brings window to front with `ViewportCommand::Focus`
  - Stale lock detection, cleanup on exit via `Drop`
  - Technical doc: `docs/technical/platform/single-instance.md`

- **Task 16**: Backlinks panel with graph-based indexing (DONE)

- **Task 15**: Wikilinks parsing, resolution, and navigation (DONE)

- **Task 13**: Manual "Check for Updates" button in Settings (DONE)

- **Task 12**: GitHub-style callouts parsing and rendering (DONE)

---

## Environment

- **Project**: Ferrite (Markdown editor)
- **Language**: Rust
- **GUI Framework**: egui 0.28
- **Branch**: `master`
- **Build**: `cargo build`
- **Version**: v0.2.7 (in progress)
