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

## Current Status: v0.2.8 Release Ready

### All Tasks Complete
All 39 Task Master tasks are `done`. 10 tasks are `deferred`:
- Tasks 11-17: Executable Code Blocks → deferred to v0.2.9+
- Tasks 27-28: Alt-Key Menu Bar → deferred to v0.3.0
- Task 38: eframe 0.28→0.31 upgrade → deferred to v0.2.9

### Release Readiness
- ROADMAP.md updated with all 49 tasks cross-referenced
- v0.2.7 section removed from roadmap (in CHANGELOG.md)
- Known Issues section updated (fixed items checked off)
- All documentation in `docs/technical/` up to date
- `docs/index.md` current

### Suggested Next Steps
1. **Cut v0.2.8 release** — all planned work complete; tag and build
2. **Add new tasks** — Use `task-master add-task` or parse a new PRD for v0.2.9
3. **Start Task 38** — If ready to attempt the egui 0.28→0.31 upgrade

## Verification
Before starting any new task:
`cargo build`
