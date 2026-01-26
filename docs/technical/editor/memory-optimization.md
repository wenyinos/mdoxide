# FerriteEditor Memory Optimization

## Overview

This document describes the memory management strategy for the FerriteEditor, particularly for handling large files efficiently.

## Performance Achievements

| Metric | Before (egui TextEdit) | After (FerriteEditor) |
|--------|------------------------|----------------------|
| 30K line file RAM | ~1.5 GB | ~90 MB |
| Responsiveness | Laggy | Smooth |
| Tab close cleanup | Memory leaked | Properly freed |

## Architecture

### Data Storage

The editor uses a **rope-based text buffer** (`ropey::Rope`) instead of a flat `String`:

```
FerriteEditor
├── buffer: TextBuffer (Rope)     ← O(log n) operations
├── line_cache: LineCache         ← 200 cached galleys max
├── history: EditHistory          ← Operation-based, not snapshots
├── view: ViewState               ← Viewport tracking
└── selections: Vec<Selection>    ← Cursor positions
```

### Memory-Efficient Patterns

1. **Virtual Scrolling**: Only visible lines are rendered each frame
2. **Galley Caching**: LRU cache of 200 text layouts, auto-evicted
3. **Operation-Based Undo**: Stores edit operations, not full content copies
4. **Viewport-Aware Features**: Syntax highlighting, search, etc. only process visible content

## Tab Closure Cleanup

When a tab is closed, `cleanup_ferrite_editor()` is called to free memory:

```rust
// In src/editor/widget.rs
pub fn cleanup_ferrite_editor(ctx: &egui::Context, tab_id: usize) {
    ctx.data_mut(|data| {
        let storage = data.get_temp_mut_or_default::<FerriteEditorStorage>(egui::Id::NULL);
        if let Some(mut editor) = storage.editors.remove(&tab_id) {
            // Explicitly clear large data structures before drop
            editor.line_cache.invalidate();
            editor.search_matches.clear();
            editor.search_matches.shrink_to_fit();
            // editor drops here, freeing TextBuffer and remaining fields
        }
        storage.content_hashes.remove(&tab_id);
        storage.content_lengths.remove(&tab_id);
    });
}
```

### Cleanup Call Sites

The cleanup is triggered from `cleanup_tab_state()` in `app.rs`:
- Tab close button clicked
- Ctrl+W keyboard shortcut
- File deletion in file tree
- Unsaved changes dialog (discard)

## Debug vs Release Performance

**Critical**: Always use `--release` builds for performance testing.

| Aspect | Debug Build | Release Build |
|--------|-------------|---------------|
| Optimization | None (`-O0`) | Maximum (`-O3`) |
| Bounds checks | Every access | Optimized away |
| Typical speed | 1x | 10-100x faster |

Debug builds have significant overhead from:
- Bounds checking on every array/rope access
- Debug assertions in egui and ropey
- No inlining of hot paths
- Overflow checking on math operations

## Memory Behavior Notes

Some memory retention after closing large files is **expected**:

1. **Allocator pooling**: Windows/Linux allocators keep freed memory for reuse
2. **egui caches**: Font atlas, texture atlas persist
3. **Fragmentation**: Small allocations between freed blocks

This is not a leak - opening/closing the same file repeatedly should NOT cause unbounded growth.

## Related Files

- `src/editor/widget.rs` - EditorWidget, FerriteEditorStorage, cleanup
- `src/editor/ferrite/buffer.rs` - TextBuffer (Rope wrapper)
- `src/editor/ferrite/line_cache.rs` - Galley caching with LRU eviction
- `src/editor/ferrite/history.rs` - Operation-based undo/redo
- `src/app.rs` - Tab lifecycle, `cleanup_tab_state()`

## Complexity Tiers

See `docs/technical/editor/architecture.md` for the full complexity tier system:

| Tier | Per-Frame OK? | Examples |
|------|---------------|----------|
| O(1) | Always | `line_count()`, `is_dirty()` |
| O(log N) | Always | `get_line()`, rope ops |
| O(visible) | Yes | Render visible lines |
| O(N) | User-initiated only | Find All, Save |
