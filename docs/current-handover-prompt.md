# Handover: Task 36 - Add setting for adjustable header spacing in rendered view

**Current Priority**: Task 36 (Low Priority)

---

## CURRENT TASK: Task 36 - Add setting for adjustable header spacing in rendered view

- **ID**: 36
- **Priority**: Low
- **Status**: Pending (next task to work on)
- **Dependencies**: None
- **Complexity**: 3

### Summary

Allow users to customize the vertical spacing between markdown headers (H1-H6) in the rendered/split view.

### Implementation Details

1. Add new setting `header_spacing` or similar in `src/config/settings.rs` with options like `compact`, `normal`, `relaxed` or a numeric value (e.g., pixels or em units)
2. Apply this spacing in `src/markdown/widgets.rs` or `src/markdown/editor.rs` where headers are rendered
3. The setting should affect margin-bottom or padding after each heading level
4. Expose the control in Settings → Editor → Markdown Rendering
5. Default should be `normal` (current behavior)
6. Ensure the setting persists across sessions

### Key Files

| File | Purpose |
|------|---------|
| `src/config/settings.rs` | Add `header_spacing` setting |
| `src/markdown/widgets.rs` | Apply spacing where headers are rendered |
| `src/markdown/editor.rs` | Alternative location for header rendering |
| `src/ui/settings.rs` | Settings → Editor → Markdown Rendering section |

### Test Strategy

1. Open Settings → see Header Spacing option
2. Select `compact` → headers have less space between them in rendered view
3. Select `relaxed` → more space between headers
4. Restart app → setting persists
5. Verify raw editor view unaffected

---

## Also Pending (Lower Priority)

### Task 34 - Investigate and fix wrapped line scroll stuttering (Medium)
Profile and fix micro-stuttering when scrolling documents with many word-wrapped lines. Likely causes: per-line galley layout cost, height cache granularity.

### Task 35 - Address remaining v0.2.7 polish (Medium)
General bug fixes and polish from GitHub issues before v0.2.7 release.

---

## Recently Completed

### Task 33 - Add Settings UI for Complex Script Font Preferences (Done)
Settings → Appearance → Additional Scripts section for pre-selecting fonts per script (Arabic, Bengali, Devanagari, Thai, Hebrew, Tamil, Georgian, Armenian, Ethiopic, Other Indic, Southeast Asian). User preferences tried first before platform defaults; persisted in config. Documentation: `docs/technical/config/complex-script-font-preferences.md`.

### Task 39 - Fix Open Folder functionality in Flatpak builds (Done)
Fixed portal dialog initialization in Flatpak sandbox. Documentation: `docs/technical/platform/flatpak-file-dialog-portal.md`.

### Task 38 - Fix task list checkbox rendering in markdown preview (Done)
Replaced ASCII `[ ]`/`[x]` text with proper egui Checkbox UI elements. Documentation: `docs/technical/markdown/task-list-checkbox.md`.

### Task 32 - Implement Basic Visual Frontmatter Editor (Done)
Form-based YAML frontmatter editing in FM tab inside outline panel. Documentation: `docs/technical/ui/frontmatter-panel.md`.

---

## Rules (DO NOT UPDATE)

- Never auto-update this file - only update when explicitly requested
- Run `cargo build` after changes to verify code compiles
- Follow existing code patterns and conventions
- Use Context7 MCP tool to fetch library documentation when needed
- Document by feature (e.g., `memory-optimization.md`), not by task
- Update `docs/index.md` when adding new documentation
- **Branch**: `master`

---

## Environment

- **Project**: Ferrite (Markdown editor)
- **Language**: Rust
- **GUI Framework**: egui 0.28
- **Branch**: `master`
- **Build**: `cargo build`
- **Version**: v0.2.7 (in progress)
