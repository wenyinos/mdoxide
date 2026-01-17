# Git Integration (Phase 1)

This document describes the Git integration feature in Ferrite, which provides Git-aware UI elements when working in Git repositories.

## Features

### 1. Branch Display in Status Bar

When Ferrite opens a workspace (folder) that is part of a Git repository, the current branch name is displayed in the status bar on the right side.

**Display format:**
- Normal branch: `⎇ main` or `⎇ feature/xyz`
- Detached HEAD: `⎇ HEAD@abc1234` (short commit hash)
- Unborn branch (new repo): `⎇ main (unborn)`

**Colors:**
- Light mode: Dark blue (#326aAA)
- Dark mode: Light blue (#82B4F0)

### 2. File Tree Git Status Badges

In workspace mode, the file tree shows Git status indicators for modified, staged, and untracked files.

**Status indicators:**

| Status | Icon | Name Color | Description |
|--------|------|------------|-------------|
| Modified | ● | Yellow/Orange | File has unstaged changes |
| Staged | ✓ | Green | File is staged for commit |
| Staged+Modified | ◐ | Yellow-green | Has both staged and unstaged changes |
| Untracked | ? | Light green | New file not tracked by Git |
| Deleted | ✕ | Red | File has been deleted |
| Renamed | → | Blue | File has been renamed |
| Conflict | ⚠ | Bright red | File has merge conflicts |
| Ignored | ○ | Gray | File is ignored by .gitignore |

**Directory indicators:**
Directories show the "worst" status of any file within them, making it easy to spot which folders contain modified files.

## Implementation Details

### Module Structure

```
src/vcs/
├── mod.rs       # Module exports
└── git.rs       # GitService implementation
```

### Key Types

```rust
/// Git status for a single file
pub enum GitFileStatus {
    Clean,          // Tracked, unmodified
    Modified,       // Working directory changes
    Staged,         // Index changes
    StagedModified, // Both staged and unstaged
    Untracked,      // Not tracked by Git
    Ignored,        // Ignored by .gitignore
    Deleted,        // File deleted
    Renamed,        // File renamed
    Conflict,       // Merge conflict
}

/// Git integration service
pub struct GitService {
    repo: Option<Repository>,
    repo_root: Option<PathBuf>,
    file_statuses: HashMap<PathBuf, GitFileStatus>,
    cache_valid: bool,
}
```

### Integration Points

1. **AppState**: Contains `git_service: GitService` field
2. **Workspace open/close**: Git service opens/closes with workspace
3. **Status bar**: Branch display in `app.rs` status bar section
4. **File tree**: Status badges passed via `HashMap<PathBuf, GitFileStatus>`

### Performance Considerations

- Status cache is lazily populated and reused
- Cache is invalidated when `refresh_status()` is called
- Full status scan happens once per file tree render
- Directory status is computed from cached file statuses

## Usage

Git integration is automatic when opening a folder that is a Git repository:

1. Open a folder via File → Open Folder (Ctrl+Shift+O)
2. If the folder is a Git repository:
   - Branch name appears in status bar
   - File tree shows status badges
3. If not a Git repository:
   - No branch display
   - No status badges

## Future Improvements (Phase 2)

The following features are planned for Phase 2:

- Editor gutter line diff markers (added/modified/deleted lines)
- Settings toggle for `git.enabled`
- Status refresh on file save
- Manual refresh command

## Dependencies

- `git2 = "0.19"` - Rust bindings for libgit2

## Testing

Unit tests are provided in `src/vcs/git.rs`:

```bash
cargo test vcs::git
```

Tests include:
- Status label and visibility
- Service creation and closing
- Repository detection
- Untracked file detection
- Status priority comparison
