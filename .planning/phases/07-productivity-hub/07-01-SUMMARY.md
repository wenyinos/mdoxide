---
phase: 07-productivity-hub
plan: 01
subsystem: ui
tags: [productivity, pomodoro, task-management, persistence, serde-json, std-time]

# Dependency graph
requires:
  - phase: 06-async-foundation
    provides: Background worker infrastructure and panel visibility toggles
provides:
  - Task struct with markdown checkbox parsing (- [ ] / - [x] syntax)
  - PomodoroTimer state machine using std::time::Instant
  - AutoSave debouncing helper for workspace persistence
  - Workspace-scoped persistence functions (.ferrite/tasks.json, .ferrite/notes/*.txt)
  - play_notification re-export from terminal module for timer alerts
affects: [07-02, 07-03, productivity-hub]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Workspace-scoped persistence in .ferrite/ directory"
    - "Atomic write pattern for safe file updates (write .bak, then rename)"
    - "std::time::Instant for timer state (immune to system clock changes)"
    - "Markdown checkbox syntax for task representation"

key-files:
  created:
    - src/ui/productivity_panel.rs
  modified:
    - src/ui/mod.rs
    - src/terminal/mod.rs

key-decisions:
  - "Use std::time::Instant instead of chrono for timer (monotonic, no clock drift)"
  - "Store tasks in .ferrite/tasks.json (workspace-scoped, not global config)"
  - "Store notes in .ferrite/notes/*.txt (one file per note, text format)"
  - "1000ms default debounce for AutoSave (prevents excessive writes)"
  - "Use atomic write pattern from config/persistence.rs (write .bak, rename)"
  - "Support priority markers in markdown (! and !! prefixes)"

patterns-established:
  - "Task struct: completed (bool), text (String), priority (u8: 0/1/2)"
  - "PomodoroTimer: 25min work, 5min break defaults, Instant-based timing"
  - "AutoSave: debounce_duration (Duration), pending_content (Option<String>)"
  - "Persistence: save_tasks/load_tasks, save_note/load_note, list_notes"

# Metrics
duration: 4min
completed: 2026-01-24
---

# Phase 07 Plan 01: Data Models Summary

**Task management with markdown parsing, Pomodoro timer using std::time::Instant, and workspace-scoped persistence to .ferrite/ directory**

## Performance

- **Duration:** 4 min
- **Started:** 2026-01-24T15:12:45Z
- **Completed:** 2026-01-24T15:17:01Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments
- Task struct with markdown checkbox parsing (- [ ] / - [x] syntax with priority markers)
- PomodoroTimer state machine using std::time::Instant (immune to clock drift)
- AutoSave helper with 1000ms debounce for performance
- Workspace-scoped persistence functions using atomic write pattern
- play_notification exported from terminal module for timer alerts
- 13 unit tests covering all data model behavior

## Task Commits

Each task was committed atomically:

1. **Task 1: Create productivity_panel.rs with data models** - `6d442bd` (feat)
2. **Task 2: Add workspace-scoped persistence and export play_notification** - `d202328` (feat)
3. **Task 3: Add unit tests for data models** - included in `6d442bd` (tests written with implementation)

## Files Created/Modified
- `src/ui/productivity_panel.rs` - Core data models: Task, PomodoroTimer, AutoSave, persistence functions
- `src/ui/mod.rs` - Export Task, PomodoroTimer, AutoSave from productivity_panel module
- `src/terminal/mod.rs` - Re-export play_notification from sound module

## Decisions Made

**1. Use std::time::Instant instead of chrono**
- Rationale: Monotonic clock immune to system time changes, no external dependency needed
- Impact: Timer accuracy guaranteed even if user adjusts system clock

**2. Workspace-scoped persistence in .ferrite/ directory**
- Rationale: Tasks and notes are project-specific, not global config
- Pattern: .ferrite/tasks.json for tasks, .ferrite/notes/*.txt for notes
- Benefits: Version control ignored, workspace-specific, follows existing .ferrite pattern

**3. Atomic write pattern (write .bak, rename)**
- Rationale: Prevents corruption from interrupted writes
- Pattern: Matches src/config/persistence.rs implementation
- Benefit: Safe even if app crashes mid-save

**4. Markdown checkbox syntax for tasks**
- Rationale: Human-readable, familiar format, easy to hand-edit
- Supports: - [ ] unchecked, - [x] checked, ! and !! priority markers
- Benefit: Can paste from/to markdown files

**5. 1000ms debounce for AutoSave**
- Rationale: Balance between data safety and performance
- Impact: Prevents excessive writes during rapid edits (e.g., typing)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

**Character literal syntax error**
- Found during: Task 1 compilation
- Issue: `name.replace(['/', '\\', '..'], "_")` - `..` cannot be in char array
- Fix: Changed to `name.replace(['/', '\\'], "_").replace("..", "_")`
- Files: src/ui/productivity_panel.rs (save_note, load_note functions)
- Verification: cargo check passes

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

**Ready for 07-02 (Productivity Panel UI):**
- Task struct with markdown parsing complete
- PomodoroTimer state machine ready for widget integration
- AutoSave debouncing ready for note editor
- Persistence functions ready for workspace save/load
- play_notification available for timer completion alerts

**Ready for 07-03 (Integration and Testing):**
- All data models have unit test coverage (13 tests passing)
- Persistence functions follow atomic write pattern
- Clear API surface: Task, PomodoroTimer, AutoSave exported from ui module

**No blockers or concerns.**

---
*Phase: 07-productivity-hub*
*Completed: 2026-01-24*
