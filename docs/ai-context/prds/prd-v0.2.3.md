# PRD: Ferrite v0.2.3 - Polish & Editor Productivity

## Product Overview

Ferrite v0.2.3 is a polish release focused on editor productivity features, platform compatibility improvements, and Mermaid diagram enhancements. This release makes the editor feel more complete with essential editing shortcuts while improving Linux support and Mermaid UX.

## Problem Statement

After v0.2.2 addressed critical bugs and CLI improvements, users expect standard editor productivity features that are missing:
1. **Missing editor shortcuts** - No Go to Line, duplicate line, or move line features
2. **Friction in markdown editing** - Manual bracket closing and link formatting
3. **Linux compatibility** - Cursor flicker bug and glibc dependency limits portability
4. **Mermaid UX gaps** - No easy way to insert diagrams or learn syntax
5. **Readability** - No option to limit line width for comfortable reading

## Target Users

1. Power users who rely on keyboard shortcuts for productivity
2. Markdown writers who want smoother editing experience
3. Linux users on various distributions (including musl-based like Alpine)
4. Users creating Mermaid diagrams who need syntax help

## Goals

1. Add essential editor productivity shortcuts (Go to Line, Duplicate, Move)
2. Improve markdown editing flow with auto-close and smart paste
3. Fix Linux-specific bugs and add musl build for maximum compatibility
4. Enhance Mermaid UX with insertion toolbar and syntax help
5. Add configurable line width for readability

## Non-Goals

1. Custom editor widget replacement (v0.3.0)
2. Mermaid crate extraction (v0.3.0)
3. Wikilinks/backlinks support (v0.3.0)
4. New diagram types

---

## Features

### 1. Editor Productivity (P0 - High Impact)

#### 1.1 Go to Line (Ctrl+G)
- **Feature:** Quick navigation to specific line number
- **Shortcut:** `Ctrl+G` (standard across most editors)
- **UI:** Small modal dialog with line number input
- **Behavior:**
  - Show current line number as placeholder
  - Accept line number, press Enter or click Go
  - Jump to line and center it in viewport
  - Close dialog on Escape or clicking outside
- **Files:** `src/editor/widget.rs`, `src/ui/dialogs.rs` (new dialog)
- **Testing:** Press Ctrl+G, type line number, verify cursor moves to that line

#### 1.2 Duplicate Line (Ctrl+Shift+D)
- **Feature:** Duplicate current line or selection
- **Shortcut:** `Ctrl+Shift+D`
- **Behavior:**
  - No selection: duplicate entire current line below cursor
  - With selection: duplicate selected text immediately after selection
  - Preserve cursor position relative to duplicated content
- **Files:** `src/editor/widget.rs`
- **Testing:** 
  - Place cursor on line, Ctrl+Shift+D → line duplicated below
  - Select text, Ctrl+Shift+D → selection duplicated after

#### 1.3 Move Line Up/Down (Alt+↑/↓)
- **Feature:** Move current line or selected lines up/down
- **Shortcuts:** `Alt+Up`, `Alt+Down`
- **Behavior:**
  - Single line: swap with line above/below
  - Multiple lines selected: move entire block
  - Maintain selection after move
  - Stop at document boundaries (don't wrap)
- **Files:** `src/editor/widget.rs`
- **Testing:**
  - Move single line up/down
  - Select 3 lines, move them as a block
  - Try to move past top/bottom of document (should stop)

#### 1.4 Auto-close Brackets & Quotes
- **Feature:** Automatically insert closing character when typing opener
- **Characters:** `()`, `[]`, `{}`, `""`, `''`, ``` `` ```, `**`, `__`
- **Behavior:**
  - Type `(` → insert `()` with cursor between
  - Type `[` → insert `[]` with cursor between
  - Type `"` → insert `""` with cursor between (if not already inside quotes)
  - With selection: wrap selection (type `(` with "hello" selected → `(hello)`)
  - Typing closing char when next char is same → just move cursor (skip over)
- **Settings:** Add `auto_close_brackets` toggle in Editor settings (default: true)
- **Files:** `src/editor/widget.rs`, `src/config/settings.rs`, `src/ui/settings.rs`
- **Testing:**
  - Type each opener, verify closer appears
  - Select text, type opener, verify wrapping
  - Type closer when cursor is before same char, verify skip-over

#### 1.5 Smart Paste for Links
- **Feature:** Paste URL on selected text to create markdown link
- **Behavior:**
  - Select text "Click here", paste `https://example.com`
  - Result: `[Click here](https://example.com)`
  - If no selection and paste URL: insert URL as-is (normal paste)
  - Detect URL by checking for `http://`, `https://`, or common patterns
- **Bonus:** Paste image URL → insert as `![](url)` if no selection
- **Files:** `src/editor/widget.rs` or paste handling code
- **Testing:**
  - Select "example", paste URL → verify link syntax created
  - No selection, paste URL → verify plain URL inserted
  - Paste non-URL → verify normal paste behavior

---

### 2. UX Improvements (P1 - Medium)

#### 2.1 Configurable Line Width (Issue #15)
- **Feature:** Limit text width for improved readability
- **Setting:** `max_line_width` in Settings > Editor
- **Options:** 
  - Off (default, current behavior)
  - 80 characters
  - 100 characters
  - 120 characters
  - Custom pixel width
- **Behavior:**
  - Center text column when width is limited
  - Apply to Raw, Rendered, and Split views
  - Zen mode should respect this setting
- **Files:** `src/config/settings.rs`, `src/ui/settings.rs`, `src/editor/widget.rs`, `src/markdown/editor.rs`
- **Testing:**
  - Set to 80 chars, open long-line document
  - Verify text wraps at ~80 chars and is centered
  - Toggle between settings, verify immediate update

---

### 3. Platform & Distribution (P1 - Medium)

#### 3.1 Linux Musl Build
- **Feature:** Statically-linked Linux binary using musl libc
- **Benefit:** Works on any Linux distro without glibc dependency (Alpine, old distros, etc.)
- **Implementation:**
  - Add new CI job `build-linux-musl`
  - Use `rust-musl-builder` Docker image or install musl target
  - Target: `x86_64-unknown-linux-musl`
  - Artifact: `ferrite-linux-musl-x64.tar.gz`
- **Files:** `.github/workflows/release.yml`
- **Testing:**
  - Build succeeds in CI
  - Binary runs on Alpine Linux container
  - Binary runs on Ubuntu without any shared library dependencies

#### 3.2 Linux Close Button Cursor Flicker
- **Problem:** Cursor rapidly switches between pointer/move/resize near window close button on Linux (Mint)
- **Root Cause:** Likely conflicting hit-test zones between custom title bar and window resize areas
- **Solution:** Investigate cursor handling in custom window frame code
- **Files:** `src/ui/window.rs`, `src/app.rs` (title bar rendering)
- **Testing:**
  - On Linux Mint (or similar), move cursor near close button
  - Verify cursor stays stable (no rapid flickering)
  - Verify close button still works
  - Verify window resize still works from corners/edges

---

### 4. Mermaid Improvements (P2 - Medium)

#### 4.1 Rendering Performance
- **Problem:** Complex diagrams (50+ nodes) can be slow to render
- **Solution:** Profile and optimize hot paths
- **Areas to investigate:**
  - Parsing: reuse buffers, avoid repeated allocations
  - Layout: cache computations, avoid O(N²) loops
  - Rendering: batch draw calls, reuse shape objects
- **Implementation:**
  - Add lightweight caching keyed by mermaid source hash
  - Only recompute for modified code blocks
- **Files:** `src/markdown/mermaid.rs`
- **Testing:**
  - Create diagram with 50+ nodes
  - Measure render time before/after optimization
  - Verify visual output unchanged

#### 4.2 Code Cleanup
- **Problem:** Large monolithic file with unused code warnings
- **Solution:**
  - Run `cargo clippy` and fix warnings
  - Remove dead code or mark with `#[allow(dead_code)]` + comment
  - Add documentation comments to public functions
  - Optionally split into submodules: `parser.rs`, `layout.rs`, `render.rs`
- **Files:** `src/markdown/mermaid.rs` → potentially `src/markdown/mermaid/`
- **Testing:**
  - `cargo clippy` passes with no warnings in mermaid code
  - `cargo doc` builds successfully
  - All diagram types still render correctly

#### 4.3 Diagram Insertion Toolbar (Issue #4)
- **Feature:** Ribbon button to insert mermaid code block templates
- **UI:** Dropdown menu from a "Diagram" or "Mermaid" button in ribbon
- **Templates:**
  ```
  Flowchart, Sequence, Class, State, ER, 
  Gantt, Pie, Mindmap, Timeline, Git Graph, User Journey
  ```
- **Behavior:**
  - Click template → insert fenced code block with basic example
  - Example for Flowchart:
    ````
    ```mermaid
    flowchart TD
        A[Start] --> B{Decision}
        B -->|Yes| C[Action]
        B -->|No| D[End]
    ```
    ````
- **Files:** `src/ui/ribbon.rs`, new templates in code or resource file
- **Testing:**
  - Click Flowchart in dropdown
  - Verify code block inserted at cursor
  - Verify diagram renders in preview

#### 4.4 Mermaid Syntax Hints in Help (Issue #4)
- **Feature:** Add Mermaid reference to Help panel
- **Content:**
  - List of supported diagram types
  - Basic syntax example for each type
  - Link to official Mermaid documentation
- **UI:** New section in Help panel (Ctrl+? or Help menu)
- **Files:** `src/ui/about.rs` (Help panel)
- **Testing:**
  - Open Help panel
  - Verify Mermaid section is present
  - Verify examples are readable and accurate

---

## Technical Requirements

### No New Dependencies Required
All features can be implemented with existing crates.

### Testing Requirements
1. Manual test all new keyboard shortcuts
2. Test auto-close with various character combinations
3. Test smart paste with URLs and non-URLs
4. Test musl binary on Alpine Linux container
5. Test Mermaid changes don't break existing diagrams

### Backward Compatibility
- All new features have sensible defaults
- `auto_close_brackets`: default true (opt-out)
- `max_line_width`: default off (current behavior)
- Existing config.json files work unchanged

---

## Implementation Priority

### Wave 1 - Editor Productivity (Most User Impact)
1. Go to Line (Ctrl+G)
2. Duplicate Line (Ctrl+Shift+D)
3. Move Line Up/Down (Alt+↑/↓)

### Wave 2 - Markdown Polish
4. Auto-close Brackets & Quotes
5. Smart Paste for Links

### Wave 3 - Platform & UX
6. Configurable Line Width
7. Linux Musl Build
8. Linux Close Button Cursor Flicker

### Wave 4 - Mermaid
9. Mermaid Rendering Performance
10. Mermaid Code Cleanup
11. Diagram Insertion Toolbar
12. Mermaid Syntax Hints in Help

---

## Success Metrics

1. Ctrl+G opens Go to Line dialog and navigates correctly
2. Ctrl+Shift+D duplicates lines
3. Alt+↑/↓ moves lines
4. Typing `(` produces `()` with cursor in middle
5. Pasting URL on selection creates markdown link
6. Line width setting limits text column width
7. Musl binary runs on Alpine Linux
8. No cursor flicker on Linux near close button
9. Mermaid diagrams render faster for complex examples
10. Ribbon has diagram insertion dropdown
11. Help panel includes Mermaid syntax reference

---

## GitHub Issues Reference

| Issue | Title | Task |
|-------|-------|------|
| #4 | Mermaid toolbar & syntax hints | Tasks 4.3, 4.4 |
| #15 | Configurable line width | Task 2.1 |
| - | Linux cursor flicker | Task 3.2 |

---

## Files Likely to Change

```
src/editor/widget.rs         # Go to Line, Duplicate, Move, Auto-close, Smart Paste
src/config/settings.rs       # max_line_width, auto_close_brackets
src/ui/settings.rs           # Settings UI for new options
src/ui/dialogs.rs            # Go to Line dialog (new or extend)
src/ui/ribbon.rs             # Diagram insertion dropdown
src/ui/about.rs              # Mermaid syntax hints in Help
src/ui/window.rs             # Linux cursor flicker fix
src/markdown/editor.rs       # Line width in rendered view
src/markdown/mermaid.rs      # Performance, cleanup
.github/workflows/release.yml # Musl build job
```

---

## Appendix: Keyboard Shortcuts Added

| Shortcut | Action |
|----------|--------|
| `Ctrl+G` | Go to Line |
| `Ctrl+Shift+D` | Duplicate Line/Selection |
| `Alt+Up` | Move Line Up |
| `Alt+Down` | Move Line Down |
