# Session Handover

## Environment
- **Project:** Ferrite (markdown editor, Rust + egui)
- **Tech Stack:** Rust 2021, egui 0.28
- **Context file:** Always read `ai-context.md` first — it contains project rules, architecture, and conventions.
- **Branch:** master

## Core Handover Rules
- **NO HISTORY:** Do not include a project history document or past task details unless they directly impact this specific task.
- **SCOPE:** Focus ONLY on the current task detailed below.
- Run `cargo build` or `cargo check` after code changes.
- Mark tasks in Task Master: `in-progress` when starting, `done` when verified.
- Document by feature under `docs/technical/` and add an entry to `docs/index.md`.
- Prefer Task Master MCP tools over CLI when available.
- Use Context7 MCP when needed.

## Current Task: #52 — Fix Custom Font Crash on Linux (GitHub Issue #114)
- **Status:** pending
- **Priority:** high
- **Complexity:** 7
- **Dependencies:** Task 37 (done)

### Description
Prevent app crash when selecting custom system fonts on Linux by catching epaint panics, validating TTF/OTF font data with magic bytes, falling back to Inter font, and adding toast notifications. Extend shaping support for `FONT_CUSTOM` in `ttf_bytes_for_font_id_shaping`.

### Implementation Details
Five changes across fonts and central panel:
1. **Panic protection** — Wrap font loading in `std::panic::catch_unwind` in `load_system_font_by_name`.
2. **Font validation** — Add TTF/OTF magic byte check (`\0\1\0\0` for TTF, `OTTO` for OTF) before passing data to epaint. Reject `.ttc` collections, Type 1, broken files.
3. **Graceful fallback + toast** — On failure, reset to Inter font and show a toast notification with the error.
4. **Fix shaping** — Add `FONT_CUSTOM` case to `ttf_bytes_for_font_id_shaping` so custom fonts get proper text shaping.
5. **Reload integration** — Ensure `reload_fonts()` in `central_panel.rs` handles errors without propagating panics.

### Key Files
- `src/fonts.rs` — `load_system_font_by_name`, `FONT_CUSTOM`, `create_font_definitions_with_cjk_spec`, `ttf_bytes_for_font_id_shaping`
- `src/app/central_panel.rs` — `reload_fonts` call

### Edge Cases
- Empty/broken font files, font collections (.ttc), non-TTF/OTF formats (Type 1, WOFF)
- font-kit returning invalid data
- Rapid font switching

### Model Selection
- **Complexity:** 7/10 (medium-high)
- **Recommendation:** Default model is fine. Involves panic handling, byte validation, and fallback logic but is well-scoped to two files.

## Verification
Before starting any new task:
`cargo build`
