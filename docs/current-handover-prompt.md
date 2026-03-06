# Handover: Bug Investigation - Session Recovery Dialog on "Don't Save"

**Current Priority**: Session Recovery Bug (Blocking v0.2.7)

---

## Rules (DO NOT UPDATE, DO NOT REMOVE RULES)

- Never auto-update this file - only update when explicitly requested
- Complete entire task before requesting next instruction
- Run `cargo build` after changes to verify code compiles
- Follow existing code patterns and conventions
- Update task status via Task Master when starting (`in-progress`) and completing (`done`)
- Use Context7 MCP tool to fetch library documentation when needed (resolve library ID first, then fetch docs)
- Document by feature (e.g., `word-wrap-performance.md`), not by task (e.g., `task-41.md`)
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

---

## Current Issue: Session Recovery Bug (INVESTIGATION IN PROGRESS)

| Field | Value |
|-------|-------|
| **Bug** | Session recovery dialog appears after user chooses "Don't Save" on exit |
| **Status** | `investigating` - NOT FIXED |
| **Priority** | HIGH (blocking v0.2.7 release) |
| **Impact** | User confusion - recovery dialog appears when it shouldn't |

### Bug Description

When user opens Ferrite, makes changes, closes the app, and chooses "Don't Save", the session recovery dialog still appears on next launch. Additionally, the workspace closes when the dialog appears.

**Expected behavior**: After "Don't Save", no recovery dialog should appear on next launch.

**Actual behavior**: Recovery dialog appears asking to restore session.

### Investigation Done So Far

#### Attempt 1: Clear recovery data in "Don't Save" handler
**File**: `src/app/dialogs.rs:97-99`
**Change**: Added `crate::config::clear_all_recovery_data()` when user clicks "Don't Save" for exit
**Result**: FAILED - Recovery dialog still appears

#### Attempt 2: Always clear recovery data in on_exit
**File**: `src/app/mod.rs:2546-2547`
**Change**: Moved `clear_all_recovery_data()` outside the `if save_session_state()` block to always run
**Result**: FAILED - Recovery dialog still appears

#### Attempt 3: Prevent crash recovery saves during exit
**File**: `src/app/mod.rs:726-729`
**Change**: Added early return in `update_session_recovery()` if `self.should_exit` is true
**Result**: FAILED - Recovery dialog still appears

### Root Cause Hypothesis

The issue appears to be a timing problem in the frame lifecycle:

1. User clicks "Don't Save" → We clear recovery data and set `should_exit = true`
2. Tab still has unsaved changes (content wasn't actually discarded?)
3. End of frame: `update_session_recovery()` runs BEFORE app closes
4. It saves a NEW crash recovery file because `has_unsaved_changes()` is still true
5. This new file triggers recovery dialog on next launch

The `should_exit` guard in Attempt 3 should have prevented this, but the bug persists. Possible reasons:
- The dialog is rendered via `render_dialogs()` which may complete in the same frame
- `should_exit` check may not be working as expected
- The tab state is not being properly cleared when "Don't Save" is clicked
- The workspace closing suggests there may be additional state changes happening

### Key Code Locations

| File | Purpose |
|------|---------|
| `src/app/dialogs.rs:90-102` | "Don't Save" button handler - calls `handle_confirmed_action()` and sets `should_exit` |
| `src/app/mod.rs:719-744` | `update_session_recovery()` - saves crash recovery periodically |
| `src/app/mod.rs:700-713` | `handle_close_request()` - handles exit flow |
| `src/app/mod.rs:2527-2554` | `on_exit()` - final cleanup when app closes |
| `src/config/session.rs:758-782` | `clear_all_recovery_data()` - deletes recovery files |
| `src/state.rs:4206-4233` | `handle_confirmed_action()` - processes confirmed exit |

### Related Logic

The recovery dialog shows when `load_session_state()` returns `is_crash_recovery=true`:
- Checks for lock file (indicates crash)
- Checks if recovery file exists with `clean_shutdown=false`
- Shows dialog if `is_crash_recovery && has_unsaved_changes()`

### Next Steps for Investigation

1. **Add debug logging** to trace the exact sequence:
   - Log when `clear_all_recovery_data()` is called
   - Log when `update_session_recovery()` saves
   - Check if `should_exit` guard is actually preventing saves

2. **Verify tab state after "Don't Save"**:
   - Check what `handle_confirmed_action()` does for `PendingAction::Exit`
   - Verify `has_unsaved_changes()` returns false after dialog closes

3. **Check for workspace-related state**:
   - The workspace closing suggests there may be workspace session saving
   - Check `src/config/session.rs` for workspace-related recovery logic

4. **Consider frame timing**:
   - The dialog is rendered via `render_ui()` → `render_dialogs()`
   - `update_session_recovery()` runs in `update()` before rendering
   - After clicking "Don't Save", the frame completes, then `on_exit()` is called
   - Is there a recovery save happening in that final frame?

### Files Modified (for reference)

```
src/app/dialogs.rs     - Added clear_all_recovery_data() in "Don't Save" handler
src/app/mod.rs         - Added should_exit guard in update_session_recovery()
src/app/mod.rs         - Moved clear_all_recovery_data() outside save conditional
```

---

## Context

This is a high-priority bug blocking the v0.2.7 release. The session recovery system is designed to help users recover from crashes, but it's triggering falsely when users explicitly choose to discard changes. The fix attempts haven't resolved the issue, suggesting a deeper timing or state management problem.
