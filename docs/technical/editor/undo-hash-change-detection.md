# Hash-Based Undo Change Detection

## Overview

Undo recording uses **blake3 hash-based change detection** on `tab.content` so Ferrite avoids unnecessary per-frame `String` clones when taking a diff baseline. The same `Tab::edit_history` stack serves **all** view modes; only the snapshot path differs by mode.

## Problem

Previously, `prepare_undo_snapshot()` cloned `tab.content` whenever the pending snapshot was `None`, which occurred:

- On the first frame after opening a file
- On every frame following an edit (snapshot was consumed by older `record_edit_from_snapshot`)
- After every undo/redo operation (snapshot was cleared)

For a 10 MB file, each unnecessary clone allocated 10 MB and copied the full string.

## Solution

### All modes (unified stack)

Every edit path records into `tab.edit_history` via `compute_edit_ops` + `record_operations`. Ctrl+Z / Ctrl+Y always use `tab.undo()` / `tab.redo()`.

### Snapshot baseline: `prepare_undo_snapshot_hashed()`

Called before editors that may mutate `tab.content`:

1. Compute blake3 hash of `tab.content` (~1–3 ms for 10 MB, no allocation).
2. Compare with stored `undo_content_hash`.
3. **If hash matches**: skip clone — existing snapshot still valid.
4. **If hash differs**: update snapshot via `clone_from` (reuses buffer capacity) or fresh `clone` if none exists.

**Raw mode** (`EditorWidget::show`): snapshot at frame start; after FerriteEditor marks `content_dirty`, sync rope → `tab.content` → `record_edit_from_snapshot()`.

**Rendered / tree / split preview** (`central_panel.rs`): same pattern when the widget reports `changed`.

### Supporting Changes

- `record_edit_from_snapshot()`: After recording ops, updates the snapshot in-place with `clone_from` and refreshes the hash. Snapshot is **not** consumed, so the next unchanged frame skips cloning.
- `undo()` / `redo()`: Update hash and snapshot in-place instead of clearing.
- `record_edit()` (legacy shim): Clears `pending_undo_snapshot` after recording; prefer the hashed snapshot path for new code.

## State Fields

```rust
// Tab struct (state.rs)
edit_history: EditHistory,
pending_undo_snapshot: Option<String>,   // baseline for diff
undo_content_hash: [u8; 32],            // blake3 digest
content_version: u64,                    // bumped on edit / undo / redo
```

## Performance

| Scenario | Before | After |
|----------|--------|-------|
| Idle scrolling (10 MB file) | O(1) None check | O(n) blake3 hash (~1–3 ms, zero allocation) |
| After each edit | 10 MB clone (new allocation) | `clone_from` (reuses buffer when possible) |
| After undo/redo | 10 MB clone (snapshot cleared) | `clone_from` (reuses buffer) |
| Raw mode idle (unchanged hash) | Clone on many frames | Hash compare only |

Net effect: eliminates fresh `String` allocations during idle/scrolling when content is unchanged, and reduces allocation churn after edits.

## Key Files

| File | Role |
|------|------|
| `src/state.rs` | `undo_content_hash`, `prepare_undo_snapshot_hashed()`, `record_edit_from_snapshot`, `undo`, `redo` |
| `src/editor/widget.rs` | Raw mode: snapshot + diff on `content_dirty` |
| `src/app/central_panel.rs` | Rendered / tree / split: snapshot + diff on `changed` |
| `src/editor/ferrite/history.rs` | `EditHistory`, `compute_edit_ops` |

## Related

- [Undo/Redo System](./undo-redo.md)
- [EditHistory Module](./edit-history.md)
