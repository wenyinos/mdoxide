# Hash-Based Undo Change Detection

## Overview

The central panel's undo recording for non-Raw view modes uses **blake3 hash-based change detection** to eliminate unnecessary per-frame `String` clones. In Raw mode, FerriteEditor's own `EditHistory` handles undo — the central panel does not participate.

## Problem

Previously, `prepare_undo_snapshot()` cloned `tab.content` whenever the pending snapshot was `None`, which occurred:
- On the first frame after opening a file
- On every frame following an edit (snapshot consumed by `record_edit_from_snapshot`)
- After every undo/redo operation (snapshot cleared)

For a 10 MB file, each unnecessary clone allocated 10 MB and copied the full string.

## Solution

### Raw Mode

Removed `prepare_undo_snapshot()` and `record_edit_from_snapshot()` calls entirely from:
- Single Raw view (`central_panel.rs`)
- Split view left pane (Raw editor)

FerriteEditor records undo operations directly through its rope-based `EditHistory`.

### Non-Raw Modes (Rendered, Tree, Split right pane)

Replaced `prepare_undo_snapshot()` with `prepare_undo_snapshot_hashed()`:

1. Compute blake3 hash of `tab.content` (~1–3 ms for 10 MB, no allocation).
2. Compare with stored `undo_content_hash`.
3. **If hash matches**: skip — content unchanged, existing snapshot still valid.
4. **If hash differs**: update snapshot via `clone_from` (reuses existing buffer capacity) or fresh `clone` if no snapshot exists.

### Supporting Changes

- `record_edit_from_snapshot()`: After recording ops, updates the snapshot in-place with `clone_from` and refreshes the hash. The snapshot is preserved (not consumed), so the next frame finds an up-to-date snapshot and skips cloning.
- `undo()` / `redo()`: Update hash and snapshot in-place instead of clearing. Avoids a re-clone on the next frame.
- `record_edit()` (legacy shim): Also updates the hash for consistency.

## State Fields

```rust
// Tab struct (state.rs)
pending_undo_snapshot: Option<String>,   // existing — snapshot for diff
undo_content_hash: [u8; 32],            // new — blake3 digest
```

## Performance

| Scenario | Before | After |
|----------|--------|-------|
| Idle scrolling (10 MB file) | O(1) None check | O(n) blake3 hash (~1–3 ms, zero allocation) |
| After each edit | 10 MB clone (new allocation) | `clone_from` (reuses buffer, no allocation if capacity suffices) |
| After undo/redo | 10 MB clone (snapshot cleared) | `clone_from` (reuses buffer) |
| Raw mode idle | 10 MB clone on first frame | Zero cost (no snapshot) |

Net effect: eliminates all fresh `String` allocations during idle/scrolling in Raw mode and reduces allocation churn in other modes.

## Key Files

| File | Change |
|------|--------|
| `src/state.rs` | `undo_content_hash` field, `prepare_undo_snapshot_hashed()`, updated `record_edit_from_snapshot`, `undo`, `redo` |
| `src/app/central_panel.rs` | Removed Raw-mode snapshot calls; switched non-Raw modes to hashed variant |
| `src/editor/ferrite/history.rs` | Updated module-level doc comment |
