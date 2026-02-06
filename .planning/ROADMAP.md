# Roadmap: Ferrite v0.5.0 "Swiss Army Knife"

**Created:** 2026-01-24
**Milestone:** v0.5.0
**Phases:** 6-11 (continuing from v0.4.0 which ended at phase 5)

---

## Phase Overview

| # | Phase | Goal | Requirements | Status |
|---|-------|------|--------------|--------|
| 6 | Async Foundation | Establish background worker infrastructure | INFRA-01..04 | ✓ Complete |
| 7 | Productivity Hub | Task tracking, Pomodoro, notes panels | PROD-01..07 | ✓ Complete |
| 8 | AI Assistant | Streaming chat panel with provider selection | AI-01..07 | On Hold |
| 9 | Power Terminal | Command history and SSH session management | TERM-01..06 | Skipped |
| 10 | Database Tools | SQLite browser with query editor | DB-01..08 | Skipped |
| 11 | Integration & Polish | Menu items, shortcuts, persistence | INT-01..04 | ✓ Complete |

---

## Phase 6: Async Foundation

**Goal:** Establish background worker infrastructure for all async features without breaking existing functionality.

**Requirements:**
- INFRA-01: Tokio runtime runs in background threads
- INFRA-02: Channel-based communication (mpsc)
- INFRA-03: Settings struct extended with panel visibility
- INFRA-04: Lazy panel initialization

**Success Criteria:**
1. User can toggle AI/Database/SSH panel visibility in View menu
2. Panel visibility persists across app restarts
3. Background worker thread starts when panel first opened
4. Existing terminal features work unchanged (regression test)
5. No UI freezing during worker operations (frame time <16ms)

**Implementation Notes:**
- Add `tokio` to Cargo.toml (feature-gated)
- Create `src/workers/mod.rs` with generic worker pattern
- Extend `Settings` struct with `*_panel_visible` fields
- Add View menu items for new panels
- Test with simple echo worker before real features

**Estimated Complexity:** Medium
**Dependencies:** None (foundation phase)

**Plans:** 3 plans

Plans:
- [x] 06-01-PLAN.md — Tokio runtime and worker infrastructure
- [x] 06-02-PLAN.md — Panel visibility settings and persistence
- [x] 06-03-PLAN.md — View menu integration with lazy initialization

---

## Phase 7: Productivity Hub

**Goal:** Add task tracking, Pomodoro timer, and quick notes as toggleable panels.

**Requirements:**
- PROD-01: Create tasks with markdown checkbox syntax
- PROD-02: Mark tasks complete
- PROD-03: Tasks persist (JSON in .ferrite/)
- PROD-04: Pomodoro timer (25/5 cycle)
- PROD-05: Sound notification when Pomodoro ends
- PROD-06: Quick notes panel
- PROD-07: Notes persist per workspace

**Success Criteria:**
1. User can type `- [ ] Task` and see a checkbox
2. Clicking checkbox marks task complete (strikethrough)
3. Tasks survive app restart
4. Pomodoro timer counts down visually
5. Sound plays when timer reaches zero (reuse existing sound module)
6. Notes panel saves automatically
7. Different workspaces have different notes

**Implementation Notes:**
- Create `src/ui/productivity_panel.rs`
- Reuse existing sound notification from `src/terminal/sound.rs`
- Store in `.ferrite/tasks.json` and `.ferrite/notes/`
- Use `std::time::Instant` for timer logic (NOT chrono - immune to clock changes)
- No async needed (all local operations)

**Estimated Complexity:** Low-Medium
**Dependencies:** Phase 6 (panel infrastructure)

**Plans:** 3 plans

Plans:
- [x] 07-01-PLAN.md — Data models and persistence (Task, PomodoroTimer, AutoSave)
- [x] 07-02-PLAN.md — ProductivityPanel UI with egui integration
- [x] 07-03-PLAN.md — UX polish and requirements verification

---

## Phase 8: AI Assistant

**Goal:** Add streaming AI chat panel with provider selection and secure API key storage.

**Requirements:**
- AI-01: Open AI panel via View menu
- AI-02: Streaming responses
- AI-03: Current file as context
- AI-04: Provider selection (Claude, OpenAI, Ollama)
- AI-05: API key configuration
- AI-06: Cancel requests
- AI-07: Markdown rendering

**Success Criteria:**
1. User can open AI panel and send a message
2. Response streams in real-time (token by token)
3. AI knows current file content
4. User can switch between Claude/OpenAI/Ollama
5. API keys stored securely (not in plain JSON)
6. Cancel button stops streaming immediately
7. Code blocks and formatting render correctly

**Implementation Notes:**
- Create `src/ui/ai_panel.rs`
- Create `src/workers/ai_worker.rs`
- Use reqwest + eventsource-client for streaming
- Use keyring crate for secure API key storage
- Implement provider abstraction (trait)
- Rate limit UI updates (every 50ms, not every token)

**Estimated Complexity:** High
**Dependencies:** Phase 6 (async foundation)

---

## Phase 9: Power Terminal

**Goal:** Add command history search and SSH session management.

**Requirements:**
- TERM-01: Search history with Ctrl+R
- TERM-02: History persists
- TERM-03: SSH connection profiles
- TERM-04: Connect to SSH servers
- TERM-05: SSH output display
- TERM-06: Search scrollback with Ctrl+F

**Success Criteria:**
1. User presses Ctrl+R and can search past commands
2. Commands from previous sessions appear in history
3. User can save SSH profile (host, user, key path)
4. Clicking "Connect" opens SSH session
5. SSH output displays in terminal-style widget
6. Ctrl+F opens search overlay for terminal content

**Implementation Notes:**
- Create `src/ui/ssh_panel.rs`
- Create `src/workers/ssh_worker.rs`
- Use russh crate for SSH
- Store history in `.ferrite/history`
- Integrate with reedline for history search
- SSH sessions could open in existing terminal tabs or dedicated widget

**Estimated Complexity:** High
**Dependencies:** Phase 6 (async foundation), existing terminal

---

## Phase 10: Database Tools

**Goal:** Add SQLite database browser with query editor and export capabilities.

**Requirements:**
- DB-01: Open database panel
- DB-02: Connect to SQLite files
- DB-03: Browse schema
- DB-04: Write/execute queries
- DB-05: Results in table grid
- DB-06: Export to CSV
- DB-07: Export to markdown table
- DB-08: Connections persist

**Success Criteria:**
1. User can open database panel from View menu
2. User can browse/select .db files and connect
3. Schema tree shows tables and columns
4. User can write SQL and click Execute
5. Results display in scrollable grid
6. "Export CSV" downloads results
7. "Export Markdown" copies table to clipboard
8. Recent connections appear on next app launch

**Implementation Notes:**
- Create `src/ui/database_panel.rs`
- Create `src/workers/database_worker.rs`
- Use rusqlite with bundled feature (no system deps)
- Use egui::Grid or egui_extras::TableBuilder for results
- Implement LIMIT default (1000 rows) for safety
- Store connection history in Settings

**Estimated Complexity:** Medium-High
**Dependencies:** Phase 6 (async foundation)

---

## Phase 11: Integration & Polish

**Goal:** Ensure all features work together, add menu items, shortcuts, and final polish.

**Requirements:**
- INT-01: All panels in View menu
- INT-02: Panel visibility persists
- INT-03: Context-aware shortcuts
- INT-04: Consistent UI patterns

**Success Criteria:**
1. View menu lists: AI Panel, Database, SSH, Productivity
2. Panel state (open/closed, width) survives restart
3. Shortcuts only active for focused panel
4. All panels use same visual style (borders, colors, spacing)

**Implementation Notes:**
- Add keyboard shortcuts (Ctrl+Shift+A for AI, etc.)
- Add settings section for each feature
- Write integration tests for shortcut conflicts
- Document all new shortcuts in help
- Profile memory/performance with all panels open

**Estimated Complexity:** Low
**Dependencies:** Phases 7-10

---

## Dependency Graph

```
Phase 6: Async Foundation
    │
    ├──────────────────────────────────────┐
    │                │                     │
    ▼                ▼                     ▼
Phase 7:         Phase 8:              Phase 9:
Productivity     AI Assistant          Power Terminal
    │                │                     │
    │                │                     │
    └────────────────┴─────────────────────┤
                                           │
                                           ▼
                                      Phase 10:
                                      Database Tools
                                           │
                                           ▼
                                      Phase 11:
                                      Integration
```

**Parallelizable:** Phases 7, 8, 9 can be developed in parallel after Phase 6.
**Sequential:** Phase 10 after at least one async feature proven. Phase 11 after all features.

---

## Risk Assessment

| Phase | Risk Level | Primary Risk | Mitigation |
|-------|------------|--------------|------------|
| 6 | Medium | Breaking existing features | Comprehensive regression tests |
| 7 | Low | None significant | Simple local operations |
| 8 | High | UI blocking during streaming | Strict channel pattern, rate limit |
| 9 | High | SSH session leaks | Use russh, explicit cleanup |
| 10 | Medium | Large query results | LIMIT by default, pagination |
| 11 | Low | Shortcut conflicts | Context-aware routing |

---

## Quality Gates (Per Phase)

**Before marking phase complete:**
- [ ] All requirements implemented
- [ ] All success criteria verified
- [ ] No regressions in existing features
- [ ] Frame time <16ms with feature active
- [ ] Binary size increase documented
- [ ] Settings migration tested (old → new)

---

## Milestone Exit Criteria

**v0.5.0 is complete when:**
1. All 29 requirements marked complete
2. All 6 phases pass quality gates
3. Binary size increase <10MB
4. No regressions in v0.4.0 features
5. Documentation updated (README, ROADMAP)
6. Release notes written

---

*Last updated: 2026-01-24*
