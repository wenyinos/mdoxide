# Undo/Redo System

## Overview

Ferrite uses a **unified, operation-based** undo/redo system. A single `EditHistory` instance per tab records minimal diffs (insert/delete operations) rather than full content snapshots. This handles both raw (FerriteEditor) and rendered (MarkdownEditor) modes identically.

## Architecture

### Per-Tab State

Each `Tab` owns one `EditHistory`:

```rust
struct Tab {
    edit_history: EditHistory,       // Operation-based undo/redo
    content_version: u64,            // Bumped on undo/redo for widget sync
    pending_undo_snapshot: Option<String>,  // Lazy pre-edit content clone
    pending_cursor_restore: Option<usize>,  // Cursor char index to restore
}
```

### Storage Model

Operations store only the changed text, not full document copies:

| Metric (4 MB file, 100 edits) | Old (snapshots) | New (operations) |
|-------------------------------|-----------------|-----------------|
| Memory usage                  | ~400 MB         | ~2 KB           |
| Per-edit cost                 | O(n) clone      | O(diff) ≈ small |

### Maximum History

Default: 500 operation groups. Large files (>1 MB): 200 groups. Each group is tiny (a few bytes to a few KB of changed text), so even 500 groups use negligible memory.

## Recording Flow

All editor modes follow the same pattern:

```rust
// Before editor.show():
tab.prepare_undo_snapshot();  // Clones content ONLY if no pending snapshot

// After editor.show():
if editor_output.changed {
    tab.record_edit_from_snapshot();  // Diffs snapshot vs current, records ops
}
```

`prepare_undo_snapshot()` avoids cloning every frame — it only clones once after each edit is recorded.

### Diff Algorithm

`compute_edit_ops(old, new)` uses prefix/suffix matching to find the minimal changed region:

1. Find common prefix (char-by-char from start)
2. Find common suffix (char-by-char from end)
3. Emit Delete for removed text, Insert for added text

This is O(n) worst case but near-instant for typical single-point edits.

### Legacy Recording

Some code paths (formatting, line operations, file operations) still use the legacy API:

```rust
let old_content = tab.content.clone();
tab.content = new_content;
tab.record_edit(old_content, cursor_pos);
```

This works the same way — `record_edit` computes the diff internally.

## Undo/Redo Operations

```rust
impl Tab {
    pub fn undo(&mut self) -> Option<usize> {
        // Applies inverse ops to self.content via edit_history.undo_string()
        // Bumps content_version
        // Returns cursor char position from the operation
    }

    pub fn redo(&mut self) -> Option<usize> {
        // Reapplies ops to self.content via edit_history.redo_string()
        // Bumps content_version
        // Returns cursor char position
    }
}
```

### Content Version

`content_version` is bumped on every undo/redo. The `EditorWidget` (raw mode) includes this in the egui widget ID, forcing FerriteEditor to re-read content via `set_content()`.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Z` (`Cmd+Z` on macOS) | Undo last edit group |
| `Ctrl+Y` | Redo undone edit group |
| `Ctrl+Shift+Z` | Redo (alternative) |

### Event Consumption

Keys are consumed in `consume_undo_redo_keys()` **before** UI rendering. This prevents both egui's built-in TextEdit undo and FerriteEditor's internal handling from triggering.

## Operation Grouping

Edits within 500 ms are grouped into a single undo unit. `break_undo_group()` forces a new group (used after formatting operations).

## Behavior Notes

- **Redo stack clearing**: New edits clear the redo stack (standard behavior).
- **Tab independence**: Each tab has its own `EditHistory`. Closing a tab discards it.
- **Save interaction**: Saving does not clear undo history.
- **Cursor restoration**: Undo returns the char position of the first operation in the group. Redo returns the position after the last operation. Both are clamped to content length.
- **Scroll preservation**: `handle_undo`/`handle_redo` in `navigation.rs` preserve scroll offset across operations.

## Testing

Tests in `state.rs` and `history.rs`:

```bash
cargo test undo
cargo test history
```

Tests cover: basic undo/redo, operation grouping, max group cap, unicode/emoji, roundtrip diff-undo, cursor restoration, redo clearing, extensive operation sequences.

## Related Documentation

- [EditHistory Module](./edit-history.md) — API reference and implementation details
