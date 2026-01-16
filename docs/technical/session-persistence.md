# Session Persistence & Crash Recovery

## Overview

Ferrite implements crash-safe session persistence that saves and restores the full editor session state (open tabs, active tab, scroll positions, cursor positions, and unsaved content) across restarts and after crashes.

## Architecture

### Session State Model

The session persistence system consists of three main components:

1. **SessionState** - Top-level session state containing:
   - Schema version (for migration support)
   - Timestamp of last save
   - Clean shutdown flag
   - List of open tabs (SessionTabState)
   - Active tab index
   - Application mode (single file or workspace)

2. **SessionTabState** - Per-tab state including:
   - Tab ID
   - File path (if saved)
   - Display title
   - View mode (Raw/Rendered)
   - Cursor position (character index and line/column)
   - Selection range
   - Scroll offset
   - Has unsaved content flag
   - File modification time (for conflict detection)
   - Original content hash

3. **RecoveryContent** - Stored separately for tabs with unsaved changes:
   - Tab ID
   - Full document content
   - Save timestamp

### File Locations

All session files are stored in the user's config directory:

- **Windows**: `%APPDATA%\ferrite\`
- **macOS**: `~/Library/Application Support/ferrite/`
- **Linux**: `~/.config/ferrite/`

Files:
- `session.json` - Clean session state (saved on normal shutdown)
- `session.recovery.json` - Crash recovery state (saved periodically)
- `session.lock` - Lock file (indicates app is running)
- `recovery/` - Directory containing per-tab recovery content files

### Persistence Flow

#### On Startup
1. Create lock file to detect future crashes
2. Check for crash recovery file and lock file presence
3. If crash detected (lock file existed from previous session):
   - Load crash recovery state
   - If unsaved changes exist, show recovery dialog
   - User can choose to restore or start fresh
4. If clean shutdown (no lock file):
   - Silently restore session from session.json

#### While Running
- Session save throttle (5 second debounce)
- On content changes, mark session as dirty
- Periodically save crash recovery snapshot
- Save recovery content for tabs with unsaved changes

#### On Clean Shutdown
1. Capture session state
2. Mark as clean shutdown
3. Save to session.json
4. Clear crash recovery data
5. Remove lock file

## Data Flow

```
                    ┌─────────────────┐
                    │   AppState      │
                    │  (Runtime)      │
                    └────────┬────────┘
                             │
              ┌──────────────┴──────────────┐
              │                             │
              ▼                             ▼
    ┌─────────────────┐           ┌─────────────────┐
    │ capture_session │           │restore_from_    │
    │ _state()        │           │session_result() │
    └────────┬────────┘           └────────▲────────┘
             │                             │
             ▼                             │
    ┌─────────────────┐           ┌────────┴────────┐
    │ SessionState    │◄─────────►│ load_session_   │
    │ (Serializable)  │           │ state()         │
    └────────┬────────┘           └─────────────────┘
             │
             ▼
    ┌─────────────────┐
    │ session.json /  │
    │ .recovery.json  │
    └─────────────────┘
```

## Recovery UI

When crash recovery is detected with unsaved changes, a modal dialog appears offering:

- **Restore Session**: Restores all tabs from the previous session, including recovered unsaved content
- **Start Fresh**: Discards the recovery data and starts with an empty editor

## Conflict Detection

The system tracks file modification time to detect conflicts:

- **NoConflict**: File hasn't changed since last read
- **ModifiedOnDisk**: File was modified externally
- **FileDeleted**: File no longer exists
- **NoFile**: Tab has no associated file

Currently, conflict information is tracked but not actively surfaced to the user (planned for future enhancement).

## Implementation Files

- `src/config/session.rs` - Session state model and persistence functions
- `src/config/mod.rs` - Module exports
- `src/state.rs` - `capture_session_state()` and `restore_from_session_result()` methods
- `src/app.rs` - Lifecycle integration (startup, periodic saves, shutdown)

## Workspace Session Persistence

### Workspace Path Handling

The session system stores the workspace root path when in workspace mode:

1. **On Save**: The workspace path is canonicalized before saving to ensure consistent storage across restarts and to resolve any relative paths or symlinks.

2. **On Restore**: The system:
   - Validates the saved path is non-empty
   - Attempts to canonicalize the path for consistency
   - Checks if the path exists on disk
   - Verifies it's actually a directory
   - Falls back to single-file mode if any validation fails

### Debug Logging

Comprehensive debug logging is available for troubleshooting session persistence issues. Enable debug logging with `--log-level debug` to see:

- Session file selection (recovery vs. session.json)
- File modification time comparisons
- Workspace path during save/load operations
- Path canonicalization results
- Validation failures and reasons

### Error Handling for Invalid Paths

The system handles various error conditions gracefully:

- **Empty path**: Session starts in single-file mode
- **Non-existent path**: Warning logged, starts in single-file mode
- **Path not a directory**: Warning logged, starts in single-file mode
- **Workspace open failure**: Warning logged with error details, starts in single-file mode

## Testing Strategy

### Unit Tests
- Session state serialization/deserialization roundtrip
- Content hashing consistency
- Save throttle timing logic

### Integration Testing
1. Open multiple documents, make edits, close cleanly
2. Reopen - verify all tabs restored with correct positions
3. Open documents, make unsaved changes, force-kill process
4. Reopen - verify recovery dialog appears
5. Test both "Restore" and "Start Fresh" options

### Manual Testing
- Verify session files are created in correct location
- Verify lock file is removed on clean exit
- Verify periodic saves occur while editing
- Test with large files and many tabs

## Future Enhancements

1. **Conflict Resolution UI**: Show dialog when file changed on disk
2. **Rendered Scroll Sync**: Track rendered mode scroll offset separately
3. **Multi-window Support**: Track multiple window states
4. **Session History**: Keep history of previous sessions
5. **Selective Restore**: Allow restoring individual tabs from recovery
