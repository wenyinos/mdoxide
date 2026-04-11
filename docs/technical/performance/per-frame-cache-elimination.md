# Per-Frame O(N) Cache Elimination

Eliminates 7 per-frame O(N) operations on `tab.content` using the existing `content_version: u64` counter on `Tab` for cache invalidation.

## Problem

Several code paths scanned the entire `tab.content` string every frame:

| Site | Operation | Cost (50MB file) |
|------|-----------|-----------------|
| `status_bar.rs` | `TextStats::from_text()` | ~25ms (word/char/line counting) |
| `central_panel.rs` / `mod.rs` | `tab.title()` → `is_modified()` | ~15ms (hash or string compare) |
| `mod.rs` | `needs_cjk()` / `needs_complex_script_fonts()` | ~10ms (char scan) |
| `mod.rs` | `should_auto_save()` → `hash_content()` | ~10ms |
| `mod.rs` | frontmatter `update_from_content()` clone + hash | ~30ms |
| `editor.rs` | `show_raw_editor()` content clone | ~25ms |
| `editor.rs` | `show_rendered_editor()` content clone | ~25ms |

## Solution

### Cache Pattern

Each cached value uses a version guard:

```rust
if tab.content_version != tab.cached_xxx_version {
    tab.cached_xxx = compute_xxx(&tab.content);
    tab.cached_xxx_version = tab.content_version;
}
```

`content_version` increments on undo, redo, and `increment_content_version()` calls — never during scroll or idle frames.

### Cached Fields on `Tab` (state.rs)

| Field | Guards | Source |
|-------|--------|--------|
| `cached_text_stats` | `cached_text_stats_version` | `TextStats::from_text()` |
| `cached_is_modified` | `cached_is_modified_version` + `cached_is_modified_save_version` | `is_modified_uncached()` |
| `cached_needs_cjk` | `cached_needs_cjk_version` | `fonts::needs_cjk()` |
| `cached_needs_complex_script` | `cached_needs_complex_script_version` | `fonts::needs_complex_script_fonts()` |
| `last_auto_save_content_version` | — | Replaces `hash_content()` call |

### Dual-Version Guard for is_modified

`is_modified()` depends on both content and save state. It uses two version counters:
- `content_version` — changes on edits
- `save_version` — changes on `mark_saved()`

Both must match for the cache to be valid.

### Per-Frame Warm-Up

`AppState::warm_tab_caches()` is called at the start of each `update()` frame to pre-compute `is_modified_cached()` for all tabs. This ensures subsequent `&self` calls to `title()` and `is_modified()` are O(1).

### MarkdownEditor Clone Elimination

- **Raw editor**: Replaced `content.clone()` + string compare with `response.changed()` from egui's `TextEdit` output.
- **Rendered editor**: Eliminated the full content clone. `rebuild_markdown()` already ignores the `original` parameter. `get_focused_element()` now uses the current content directly (self-corrects on the next frame if an edit happened).

### Frontmatter Panel

Replaced `DefaultHasher` content hashing with `content_version` comparison. The content clone only happens when content actually changes (not every frame).

## Files Changed

| File | Change |
|------|--------|
| `src/state.rs` | Added 12 cache fields to `Tab`, `is_modified_cached()`, `text_stats()`, `needs_cjk_cached()`, `needs_complex_script_cached()`, `warm_tab_caches()` |
| `src/app/mod.rs` | Call `warm_tab_caches()` at frame start; use cached font detection |
| `src/app/status_bar.rs` | Use `tab.text_stats()` instead of `TextStats::from_text()` |
| `src/ui/frontmatter_panel.rs` | `update_from_content_versioned()` with version guard |
| `src/markdown/editor.rs` | Eliminate content clones in both editor modes |
