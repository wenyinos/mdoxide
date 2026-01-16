# Git Auto-Refresh

Automatic Git status refreshing for file tree indicators and status bar branch display.

## Overview

Ferrite automatically refreshes Git status information to keep the UI in sync with the actual repository state. This ensures that file tree status badges and the branch indicator remain accurate without requiring manual refresh.

## Refresh Triggers

### 1. On File Save
When a file is saved (Ctrl+S or File > Save), git status is refreshed to update indicators for the saved file and any dependent status (e.g., staging area changes).

### 2. On Window Focus
When the Ferrite window regains focus (e.g., after switching back from another application), git status is refreshed. This catches changes made by external tools like `git add`, `git commit`, or other editors.

### 3. Periodic Refresh (Every 10 Seconds)
While a workspace is open with a git repository, status is refreshed every 10 seconds. This catches external changes from:
- Other applications modifying files
- Background processes (build tools, formatters)
- Other git operations

### 4. On File System Events
When the file watcher detects file tree changes (create, delete, rename), a git refresh is also triggered to update status badges.

## Debouncing

To prevent excessive git2 calls that could impact performance, refresh requests are debounced with a 500ms delay. Multiple rapid triggers (e.g., saving multiple files quickly) are batched into a single refresh operation.

```
Request 1 → 
              [500ms wait] → Single refresh
Request 2 →
```

## Implementation

### Key Components

| File | Component | Purpose |
|------|-----------|---------|
| `src/vcs/git.rs` | `GitAutoRefresh` | Manages refresh timing, debouncing, focus tracking |
| `src/vcs/git.rs` | `GitService::refresh_status()` | Performs actual git2 status query |
| `src/app.rs` | `handle_git_auto_refresh()` | Called each frame in update loop |
| `src/app.rs` | `request_git_refresh()` | Triggers debounced refresh |

### GitAutoRefresh Struct

```rust
pub struct GitAutoRefresh {
    last_refresh: Option<Instant>,      // For periodic timer
    last_request: Option<Instant>,      // For debounce calculation
    pending_refresh: bool,              // Whether refresh is queued
    was_focused: bool,                  // Previous focus state
}
```

### Configuration Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `GIT_REFRESH_INTERVAL` | 10 seconds | Periodic refresh timer |
| `GIT_DEBOUNCE_DURATION` | 500ms | Minimum time between refreshes |

## Performance Considerations

### CPU Impact
- Target: Less than 5% CPU impact during refresh cycles
- Git status queries are lightweight for typical repository sizes
- Debouncing prevents refresh storms

### Memory
- Status cache is stored in `GitService::file_statuses` HashMap
- Cache is invalidated on each refresh (full refresh, not incremental)
- Only non-clean files are stored (clean files = no entry)

### Large Repositories
For very large repositories (10,000+ files), the 10-second periodic refresh may cause brief UI pauses.

**Potential optimizations (deferred unless needed):**
- Make `GIT_REFRESH_INTERVAL` configurable via settings
- Implement incremental status updates (track changed files, partial cache invalidation)
- Use file watcher events to target specific paths instead of full refresh

These are noted in CHANGELOG.md under "Deferred" for v0.2.5.

## Testing

### Unit Tests
Located in `src/vcs/git.rs`:

```rust
#[test] fn test_git_auto_refresh_new()
#[test] fn test_git_auto_refresh_focus_gained()
#[test] fn test_git_auto_refresh_periodic_refresh_never_refreshed()
#[test] fn test_git_auto_refresh_periodic_refresh_recent()
#[test] fn test_git_auto_refresh_mark_refreshed()
#[test] fn test_git_auto_refresh_tick_no_workspace()
#[test] fn test_git_auto_refresh_tick_with_workspace_first_time()
#[test] fn test_git_auto_refresh_debounce_not_ready()
#[test] fn test_git_auto_refresh_debounce_no_pending()
```

### Manual Testing

| Test | Steps | Expected |
|------|-------|----------|
| Auto-refresh | 1. Open workspace with git repo<br>2. Modify a file externally<br>3. Wait up to 10 seconds | File tree badge updates |
| Save trigger | 1. Modify and save a file | Status badge updates immediately |
| Focus trigger | 1. Switch to terminal, run `git add .`<br>2. Switch back to Ferrite | Badges update on focus |
| Debounce | 1. Save files rapidly | Only one refresh executes |

## Related Documentation

- [Git Integration](./git-integration.md) - Overall git integration architecture
- [Workspace Folder Support](./workspace-folder-support.md) - File tree and workspace features
