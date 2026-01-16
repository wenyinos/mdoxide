# Windows Path Normalization

## Overview

Utility module to normalize Windows paths by stripping the extended-length path prefix (`\\?\`) that `std::fs::canonicalize()` adds on Windows. This prevents duplicate entries in recent files/folders and fixes git integration issues.

## Problem

On Windows, `std::fs::canonicalize()` returns paths with the verbatim prefix:
- Input: `G:\DEV\project`
- Output: `\\?\G:\DEV\project`

This caused several issues:
1. **Duplicate entries** - Same path stored with and without prefix
2. **Git integration failures** - `git2` library doesn't handle verbatim paths
3. **Path comparison failures** - `\\?\G:\path` != `G:\path`

## Key Files

| File | Purpose |
|------|---------|
| `src/path_utils.rs` | New utility module with normalization functions |
| `src/main.rs` | Module declaration |
| `src/state.rs` | Session capture/restore path handling |
| `src/app.rs` | CLI path handling |
| `src/config/settings.rs` | Recent files/workspaces storage |

## Implementation Details

### Core Function

```rust
pub fn normalize_path(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        // Strip \\?\ prefix from VerbatimDisk paths
        // E.g., \\?\G:\DEV\project -> G:\DEV\project
    }
    #[cfg(not(windows))]
    {
        path // No-op on other platforms
    }
}
```

### Integration Points

1. **After `canonicalize()` calls** - All three locations now normalize:
   - Session capture (`state.rs`)
   - Session restore (`state.rs`)
   - CLI path handling (`app.rs`)

2. **When adding recent items** - `Settings` methods normalize on add:
   - `add_recent_file()` - Normalizes and deduplicates
   - `add_recent_workspace()` - Normalizes and deduplicates

3. **On settings load** - `sanitize()` calls `normalize_stored_paths()`:
   - Fixes existing paths in config
   - Removes duplicates caused by mixed prefix usage

### Path Prefixes Handled

| Prefix | Description | Example |
|--------|-------------|---------|
| `\\?\` | Verbatim disk path | `\\?\C:\path` → `C:\path` |
| `\\?\UNC\` | Verbatim UNC path | `\\?\UNC\server\share` → `\\server\share` |

## API

```rust
// Primary function - normalize a path
pub fn normalize_path(path: PathBuf) -> PathBuf;

// Reference version
pub fn normalize_path_ref(path: &Path) -> PathBuf;

// Convenience: canonicalize + normalize (returns Option)
pub fn canonicalize_and_normalize(path: &Path) -> Option<PathBuf>;

// Convenience: canonicalize + normalize with fallback
pub fn canonicalize_or_normalize(path: &Path) -> PathBuf;
```

## Testing

```bash
cargo test path_utils
```

Tests cover:
- Regular path passthrough
- Verbatim disk path stripping
- Verbatim UNC path conversion
- Drive letter normalization (lowercase → uppercase)
- Non-existent path handling

## Usage Example

```rust
// Before (problematic)
let canonical = path.canonicalize()?;  // Returns \\?\G:\path on Windows

// After (fixed)
let canonical = path
    .canonicalize()
    .map(crate::path_utils::normalize_path)?;  // Returns G:\path
```

## Related

- [Recent Files](./recent-files.md) - Uses normalized paths
- [Session Persistence](./session-persistence.md) - Stores normalized workspace paths
- [Git Integration](./git-integration.md) - Benefits from normalized paths
