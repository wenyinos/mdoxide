---
phase: 06-async-foundation
plan: 01
subsystem: infra
tags: [tokio, async, workers, mpsc, rust]

# Dependency graph
requires:
  - phase: 05-terminal
    provides: Terminal panel infrastructure requiring async-safe background operations
provides:
  - Tokio runtime infrastructure with feature gate (async-workers)
  - Generic worker pattern using std::sync::mpsc channels
  - Echo worker proof-of-concept demonstrating async in background thread
  - Foundation for AI assistant, SSH client, and database connections
affects: [06-02-panel-visibility, 06-03-workspace-panel, ai-assistant, ssh-client, database]

# Tech tracking
tech-stack:
  added:
    - tokio 1.49 (rt-multi-thread, macros, sync, time)
    - poll-promise 0.3 (UI integration for async results)
  patterns:
    - Worker pattern: background thread with tokio runtime, mpsc communication
    - Feature-gated async infrastructure (not in default features yet)
    - UI thread never blocked by async operations

key-files:
  created:
    - src/workers/mod.rs
    - src/workers/echo_worker.rs
  modified:
    - Cargo.toml (tokio dependencies, async-workers feature)
    - src/main.rs (workers module declaration)

key-decisions:
  - "Tokio runs in background threads, NOT main thread (egui constraint)"
  - "Use std::sync::mpsc for UI ↔ worker communication (cross-thread safe)"
  - "Feature gate async-workers (not default) for gradual rollout"
  - "Added 'time' feature to tokio for sleep/timeout functionality"

patterns-established:
  - "WorkerHandle::spawn() pattern for launching background workers"
  - "Command/Response enums for type-safe worker communication"
  - "Automatic shutdown on WorkerHandle drop"

# Metrics
duration: 6min
completed: 2026-01-24
---

# Phase 06 Plan 01: Tokio Runtime and Worker Infrastructure Summary

**Tokio runtime with generic worker pattern using mpsc channels, feature-gated for background async operations**

## Performance

- **Duration:** 6 minutes
- **Started:** 2026-01-24T12:19:14Z
- **Completed:** 2026-01-24T12:25:00Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Tokio 1.49 added with feature gate (async-workers) for gradual adoption
- Generic worker infrastructure with WorkerHandle, WorkerCommand, WorkerResponse
- Echo worker demonstrates async operations in background thread without blocking UI
- Foundation ready for AI assistant, SSH client, and database workers

## Task Commits

Each task was committed atomically:

1. **Task 1: Add tokio dependency with feature gate** - `e7261cd` (feat)
2. **Task 2: Create worker module with generic pattern** - `18c9955` (feat)
3. **Task 3: Create echo worker as proof-of-concept** - `2dc8a8d` (feat)
4. **Fix: Add tokio time feature and workers module** - `6d3cf67` (fix)

## Files Created/Modified

- `Cargo.toml` - Added tokio 1.49 and poll-promise with async-workers feature gate
- `Cargo.lock` - Locked tokio dependencies
- `src/workers/mod.rs` - Generic worker infrastructure (WorkerCommand, WorkerResponse, WorkerHandle)
- `src/workers/echo_worker.rs` - Proof-of-concept echo worker with tokio runtime
- `src/main.rs` - Feature-gated workers module declaration

## Decisions Made

1. **Tokio runtime placement**: Workers create tokio runtime in background threads, NOT main thread
   - Rationale: egui is single-threaded and must never block on async operations

2. **Channel choice**: std::sync::mpsc (not tokio::sync::mpsc)
   - Rationale: Crosses thread boundaries (UI thread ↔ worker thread)

3. **Feature gate**: async-workers not in default features yet
   - Rationale: Allows testing foundation before enabling by default

4. **Tokio features**: Added 'time' feature for sleep/timeout
   - Rationale: Echo worker needs tokio::time::sleep for demonstration

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added tokio 'time' feature**
- **Found during:** Task 3 (echo_worker compilation)
- **Issue:** tokio::time::sleep not available - 'time' feature missing from Cargo.toml
- **Fix:** Added "time" to tokio features list
- **Files modified:** Cargo.toml, Cargo.lock
- **Verification:** `cargo test --features async-workers test_echo_worker` passes
- **Committed in:** `6d3cf67` (separate fix commit)

**2. [Rule 3 - Blocking] Added workers module declaration**
- **Found during:** Task 3 (echo_worker test compilation)
- **Issue:** Workers module not declared in main.rs, tests couldn't compile
- **Fix:** Added `#[cfg(feature = "async-workers")] mod workers;` to main.rs
- **Files modified:** src/main.rs
- **Verification:** Worker tests compile and run
- **Committed in:** `6d3cf67` (combined with feature fix)

---

**Total deviations:** 2 auto-fixed (2 blocking issues)
**Impact on plan:** Both fixes necessary for compilation. No scope creep.

## Issues Encountered

None - plan executed smoothly with only expected compilation blockers.

## User Setup Required

None - no external service configuration required.

## Verification Results

### Build Tests
- ✅ `cargo build` (without async-workers) - succeeds (no breaking changes)
- ✅ `cargo build --features async-workers` - succeeds
- ✅ Feature gate works correctly

### Unit Tests
- ✅ `test_echo_worker_responds` - Worker processes echo commands correctly
- ✅ `test_echo_worker_shutdown` - Graceful shutdown on command
- ✅ Both tests verify async operations work in background thread

### Regression Tests
- ✅ Existing terminal features compile
- ⚠️  Pre-existing test failure: `vcs::git::tests::test_git_service_non_repo` (unrelated to this work)

## Next Phase Readiness

**Ready for 06-02 (Panel Visibility Settings):**
- Worker infrastructure established
- Can now add panel visibility controls for async features

**Ready for future async features:**
- AI assistant can use worker pattern for OpenAI API calls
- SSH client can use worker pattern for remote connections
- Database client can use worker pattern for queries

**Blockers:**
None

**Concerns:**
- Pre-existing git test failure should be investigated separately
- Terminal panel has unused methods (warnings) - cleanup opportunity

---
*Phase: 06-async-foundation*
*Completed: 2026-01-24*
