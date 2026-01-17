# Viewer State Cleanup

## Overview

Fixes a memory leak where viewer state HashMap entries (`tree_viewer_states`, `csv_viewer_states`, `sync_scroll_states`) were not cleaned up when tabs were closed.

## Key Files

- `src/app.rs` - Contains viewer state HashMaps and cleanup logic

## Implementation Details

### Problem

When tabs with structured files (JSON/YAML/TOML) or tabular files (CSV/TSV) were opened, state was stored in HashMaps keyed by `tab_id`. When tabs were closed, these entries remained in memory indefinitely, causing unbounded growth over open/close cycles.

### Solution

Added `cleanup_tab_state()` helper method that removes entries from all three viewer state HashMaps:

```rust
fn cleanup_tab_state(&mut self, tab_id: usize) {
    self.tree_viewer_states.remove(&tab_id);
    self.csv_viewer_states.remove(&tab_id);
    self.sync_scroll_states.remove(&tab_id);
}
```

### Tab Close Paths Updated

| Location | Context |
|----------|---------|
| CLI file open | Closing empty default tab when opening files from command line |
| Tab bar X button | User clicks close button on tab |
| File deletion | Tabs auto-close when their file is deleted |
| Ctrl+W | `handle_close_current_tab()` keyboard shortcut |
| Confirmation dialog (Save) | User confirms save-then-close for modified file |
| Confirmation dialog (Discard) | User confirms close without saving |

### Key Pattern

Before each `close_tab()` call, capture the `tab_id`:

```rust
// Get tab_id before closing for viewer state cleanup
let tab_id = self.state.tabs().get(index).map(|t| t.id);
self.state.close_tab(index);
if let Some(id) = tab_id {
    self.cleanup_tab_state(id);
}
```

For confirmation dialogs, the `tab_id` is extracted before the dialog buttons are processed, since `handle_confirmed_action()` consumes the pending action.

## Dependencies Used

None - uses standard library `HashMap::remove()`.

## Testing

1. Open 10+ JSON/CSV files
2. Close all tabs
3. Repeat cycle 5+ times
4. Memory should stay stable (not grow unbounded)

Note: This does not reduce baseline memory (~250MB) which is dominated by embedded fonts.
