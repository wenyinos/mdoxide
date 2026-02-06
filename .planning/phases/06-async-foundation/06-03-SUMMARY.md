---
phase: 06-async-foundation
plan: 03
subsystem: ui
tags: [egui, view-menu, panel-integration, lazy-initialization, worker-pattern]

# Dependency graph
requires:
  - phase: 06-01-tokio-runtime
    provides: Background worker infrastructure with mpsc communication
  - phase: 06-02-settings-persistence
    provides: Panel visibility configuration and settings loading
provides:
  - View menu with panel visibility toggles
  - Lazy worker initialization pattern (Echo Demo as template)
  - AI Assistant panel placeholder (Echo Demo implementation)
  - Menu-driven panel management system
affects:
  - 06-04-cross-platform-integration (will use same panel toggling pattern)
  - Future panel additions (all will follow lazy initialization pattern)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Lazy worker initialization (workers start on first panel visibility)
    - Menu-driven UI state management
    - Panel placeholder pattern for future AI integration

key-files:
  created:
    - src/ui/view_menu.rs (View menu implementation)
  modified:
    - src/ui/mod.rs (menu structure updates)
    - src/terminal/widget.rs (panel integration)
    - src/app.rs (menu routing)

key-decisions:
  - "Echo Demo panel as AI Assistant placeholder - shows pattern for future AI integration"
  - "Lazy initialization: workers only created when panel first becomes visible"
  - "View menu organized by panel type (Panels section)"

patterns-established:
  - "Lazy worker pattern: OnceCell + sender spawning on visibility toggle"
  - "Menu item structure: View > Panels > [Panel Name]"
  - "Panel placeholder impl: Echo + message state + worker communication"

# Metrics
duration: "35min"
completed: 2026-01-24
---

# Phase 6 Plan 3: View Menu and Lazy Panel Integration Summary

**View menu with four panel toggles, lazy Echo Demo worker initialization, and AI Assistant panel placeholder implementing the cross-thread panel pattern**

## Performance

- **Duration:** 35 min
- **Completed:** 2026-01-24
- **Tasks:** 4 (3 implementation + 1 human verification checkpoint)
- **Files modified:** 5

## Accomplishments

- **View menu with panel toggles** - Four checkboxes for each panel (Terminal, Productivity, AI Assistant, Debug)
- **Lazy worker initialization** - Echo Demo worker only spawns when panel first becomes visible
- **AI Assistant placeholder** - Echo Demo panel demonstrates message state management and cross-thread communication pattern
- **Menu routing** - View menu properly connects to egui UI state, toggles persist via settings

## Task Commits

Each task was committed atomically:

1. **Task 1: Add View menu items for panel toggles** - `b049b5a` (feat)
   - Created view_menu.rs with menu structure
   - Integrated View menu into app menu bar
   - Connected visibility toggles to AppState

2. **Task 2: Implement lazy worker initialization pattern** - `40883cf` (feat)
   - Added OnceCell-based lazy worker initialization
   - Echo Demo worker only spawns on first panel visibility
   - Worker communication via mpsc channel

3. **Task 3: Add echo demo panel (AI panel placeholder)** - `aba7696` (feat)
   - Echo Demo panel renders in AI Assistant panel slot
   - Text input + Echo button with worker communication
   - Message state management with async response handling

4. **Task 4: Human verification checkpoint** - APPROVED
   - All 4 view menu checkboxes visible and functional
   - Panel visibility toggles work correctly
   - Echo Demo worker initializes lazily on panel visibility
   - Echo functionality tested: input → send → echo response

## Files Created/Modified

- `src/ui/view_menu.rs` - New View menu implementation with panel visibility toggles
- `src/ui/mod.rs` - Menu structure and integration
- `src/terminal/widget.rs` - Panel container integration with visibility state
- `src/app.rs` - App menu routing and state management
- `src/config/settings.rs` - Panel visibility persistence

## Decisions Made

- **Lazy worker initialization:** Workers don't spawn at startup. They're created on-demand when panel becomes visible for the first time. This reduces startup overhead and memory footprint.
- **Echo Demo as AI Assistant placeholder:** Echo panel demonstrates the complete pattern (menu toggle → lazy worker → message communication → UI update). Future AI integration will reuse this structure.
- **View menu organization:** Grouped all panel toggles under "View > Panels" for consistency and discoverability.
- **Cross-thread communication:** Using mpsc::channel for Echo Demo worker demonstrates the worker communication pattern used in Tokio infrastructure - all future panels will follow this model.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - all verification checkpoints passed.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- **Complete:** Panel visibility system with menu integration and lazy worker infrastructure
- **Pattern established:** Future panels (File Searcher, Settings, etc.) can clone Echo Demo structure
- **Ready for:** Phase 6.4 (Cross-platform Integration) which will reuse the same panel toggling pattern
- **Consideration:** AI integration endpoint can replace echo_worker with actual LLM API calls while keeping the panel infrastructure identical

---

*Phase: 06-async-foundation*
*Completed: 2026-01-24*
