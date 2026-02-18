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

**Task 29: Always show view mode bar for all editor tabs**
- **Priority**: Medium
- **Dependencies**: None
- **Status**: Pending
- **Task Master ID**: 29

### Description
When default view is Split and user opens a file that does not support split view (e.g. .rs), the view mode bar is hidden so mode can only be changed via hotkeys. Always display the view mode bar; for unsupported file types show the two-mode segment (Raw | Rendered).

### Implementation Details
In `src/app/title_bar.rs` the view mode segment is only shown when `current_file_type` is markdown, structured, or tabular (line ~320). Change to always show for `has_editor && !is_special_tab`. For file types that support split (markdown, tabular) use `segment.show()` (3-mode); for structured use `segment.show_two_mode()`; for all other types (e.g. .rs) also use `show_two_mode(ui, current_view_mode, is_dark)` so users can switch to Raw or Rendered from the UI. Optionally in open-file flow: when opening a file that does not support split and `default_view_mode` is Split, set `tab.view_mode` to Raw so initial state is consistent.

### Key Files

| File | Purpose |
|------|---------|
| `src/app/title_bar.rs` | View mode segment rendering — change visibility condition |
| `src/ui/view_segment.rs` | View mode segment widget (show / show_two_mode) |
| `src/state.rs` | FileType, ViewMode, Tab — check file type capabilities |

### Test Strategy
1. Set default view to Split.
2. Open a .rs file → view mode bar visible with Raw | Rendered.
3. Switch to Raw and Rendered via bar.
4. Open .md file → 3-mode bar (Raw | Split | Rendered) still works.
5. Hotkey toggle still works for all.

---

## Recently Completed (Previous Sessions)

- **Task 26**: Windows MSI installer overhaul (DONE)
  - Feature tree: file associations, context menu, PATH, desktop shortcut, Default Apps registration
  - WixUI_FeatureTree with per-extension toggles, launch-after-install checkbox
  - Technical doc: `docs/technical/platform/msi-installer-features.md`

- **Task 20 + 21**: Vim mode settings toggle, status bar indicator, and core modal state machine (DONE)
  - Technical doc: `docs/technical/editor/vim-mode.md`

- **Task 19**: Lazy CSV row parsing with byte-offset indexing (DONE)
- **Task 30**: Light mode text readability fix (DONE)
- **Task 17**: Flowchart modular refactoring (DONE)
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
