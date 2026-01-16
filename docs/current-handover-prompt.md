# Session Handover - Ferrite v0.2.5

## Rules

- Never auto-update this file - only update when explicitly requested
- Complete entire task before requesting next instruction
- Run `cargo build` / `cargo check` after changes to verify code compiles
- Follow existing code patterns and conventions
- Update task status via Task Master when starting (`in-progress`) and completing (`done`)
- Use Context7 MCP tool to fetch library documentation when needed (e.g., egui)
- Document by feature (e.g., `config-persistence.md`), not by task (e.g., `task-35.md`)
- Update `docs/index.md` when adding new documentation

## Environment

- **Project:** Ferrite
- **Path:** G:\DEV\markDownNotepad
- **GitHub:** https://github.com/OlaProeis/Ferrite
- **Version:** Working on v0.2.5
- **Tech Stack:** Rust, egui 0.28, eframe 0.28, comrak 0.22, pulldown-cmark 0.11, clap 4, rust-i18n 3, git2 0.19, csv 1.3, palette 0.7, chrono 0.4

---

## Current Task: Add custom font selection

| Field | Value |
|-------|-------|
| **ID** | 39 |
| **Title** | Add custom font selection |
| **Status** | `pending` |
| **Priority** | Medium |
| **Dependencies** | 3 |
| **Issue** | [#15](https://github.com/OlaProeis/Ferrite/issues/15) |

### Description

Allow users to select preferred font for editor and UI, important for CJK regional glyph preferences.

### Implementation Notes

1. **Settings UI for font selection**
   - Add section in Settings panel for font configuration
   - Separate options for: editor font, UI font, monospace font
   - List available system fonts using `font-kit` crate

2. **CJK regional glyph support**
   - Important for users needing specific regional variants
   - Simplified Chinese vs Traditional Chinese vs Japanese vs Korean glyphs
   - Font selection affects which glyph variants are rendered

3. **Apply changes immediately**
   - Font changes should apply without restart
   - Store selections in `config.json`

### Key Files

| File | Purpose |
|------|---------|
| `src/fonts.rs` | Font loading and `EditorFont` enum |
| `src/ui/settings.rs` | Settings panel UI |
| `src/config/settings.rs` | Settings struct with font preferences |
| `src/app.rs` | Font application at runtime |

### Relevant Dependencies

- `font-kit` - System font enumeration and loading
- `egui::FontDefinitions` - egui font configuration

### Test Strategy

- Test font picker shows system fonts
- Test changing editor font applies to editor
- Test CJK fonts render correct regional glyphs
- Test persistence across restart

---

## Recently Completed (v0.2.5)

- Task 38: Windows borderless window fixes (top edge resize, fullscreen toggle)
- Task 37: macOS Intel CPU optimization (idle repaint scheduling)
- Task 36: Fix line width in rendered/split view (centering behavior, pane boundaries)
- Task 35: Fix config.json persistence (window state dirty flag)
- Task 34: CJK paragraph indentation (done)
- Task 28: Zen mode centering in rendered/split views (done)
- Task 27: Table editing - data sync fixed
- Task 26: Quick switcher mouse support (done)
- Task 25: Keyboard shortcut customization (done)

## Deferred

- Task 29, 30: Mermaid toolbar/help (→ v0.3.0)
- Task 31: Chinese translation (waiting for contributor)
- Task 32: Mermaid code cleanup (→ v0.2.6)
- Task 33: Weblate setup (manual task for maintainer)
