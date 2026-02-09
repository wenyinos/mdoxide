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

**Task 15: Implement wikilinks parsing, resolution, and navigation**
- **Priority**: High
- **Dependencies**: Task 12 (callouts — DONE)
- **Status**: Pending
- **Task Master ID**: 15

### Description
Add `[[wikilinks]]` and `[[target|display]]` syntax support with file resolution, spaces in filenames, and click-to-navigate in rendered/split view.

### Implementation Details
1. **Parser** (`src/markdown/parser.rs`): Extend to parse `[[target]]` and `[[target|display text]]` syntax. Store as a new AST node (similar to how callouts were added in Task 12).
2. **Resolution**: Resolve wikilink targets to actual files:
   - Relative to current file's directory first
   - Then workspace root
   - Tie-breaker: same-folder-first → shortest path → prompt if ambiguous
   - Support spaces: `[[My Note]]` → `My Note.md`
3. **Rendering** (`src/markdown/editor.rs`): Render wikilinks as clickable links in rendered/split view. Show display text if provided, otherwise the target name.
4. **Navigation**: Click opens the target file in a new tab (or switches to existing tab).
5. **Broken links**: Handle gracefully — style differently (red/dimmed) or show tooltip.

### Key Files

| File | Purpose |
|------|---------|
| `src/markdown/parser.rs` | Add wikilink parsing to AST |
| `src/markdown/editor.rs` | Render wikilinks as clickable links |
| `src/markdown/widgets.rs` | Wikilink widget serialization |
| `src/state.rs` | File navigation / tab opening |
| `src/app/file_ops.rs` | Open file in tab |

### Reference
- Task 12 (callouts) added `CalloutType` enum and `Callout` AST node — follow the same pattern for wikilinks
- Existing link rendering in `editor.rs` — look at how `[text](url)` links are handled for click behavior
- GitHub issue: [#1](https://github.com/OlaProeis/Ferrite/issues/1)

### Test Strategy
1. `[[note-b]]` → renders as clickable, clicking opens `note-b.md`
2. `[[note-b|Custom Text]]` → shows "Custom Text", navigates to `note-b.md`
3. `[[My Document]]` with spaces → resolves to `My Document.md`
4. Ambiguous targets → prompt user to choose
5. Broken/missing links → handled gracefully (visual indicator, no crash)

---

## Recently Completed (This Session)

- **Task 13**: Manual "Check for Updates" button in Settings (DONE)
  - New module `src/update.rs` with GitHub API check, version comparison, URL validation
  - Added `ureq` dependency (lightweight blocking HTTP with rustls TLS)
  - New "About" section in Settings panel with inline update state display
  - Security: URL prefix validation, pure-Rust TLS
  - Technical doc: `docs/technical/ui/check-for-updates.md`

- **Task 12**: GitHub-style callouts parsing and rendering (DONE)

---

## Environment

- **Project**: Ferrite (Markdown editor)
- **Language**: Rust
- **GUI Framework**: egui 0.28
- **Branch**: `master`
- **Build**: `cargo build`
- **Version**: v0.2.7 (in progress)
