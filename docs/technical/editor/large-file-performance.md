# Large File Performance Optimizations

## Overview

Performance optimizations made during Phase 1 validation to ensure smooth editing of large files (5MB+). These fixes address per-frame operations that scaled with file size. Additionally, users are notified when opening very large files so they are aware that performance may be affected.

## Open-time Warning Toast (v0.2.7)

When opening a file, the app checks its size via `std::fs::metadata(path)` before loading. If the file is larger than **10 MB**, a non-blocking toast is shown: *"Large file (X MB). Performance may be affected."* The file is then loaded as usual; the toast is informational only.

- **Threshold**: 10 MB (`LARGE_FILE_THRESHOLD_BYTES` in `src/state.rs`). Intended to be configurable in settings in a future release.
- **Implementation**: `AppState::open_file_with_focus()` in `src/state.rs`. Callers pass `app_time: Option<f64>` so the toast can be shown when time is available (e.g. from the UI); tests and internal callers pass `None` and do not show the toast.
- **i18n**: Message key `notification.large_file_performance` in `locales/en.yaml`.

## Key Optimizations

### 1. Content Hash Avoidance (widget.rs)

**Problem**: Content hash was computed every frame to detect external changes, causing O(n) hashing of large files 60 times per second.

**Solution**: Use content length as a quick check first, only compute hash if length matches and file is small.

```rust
// Before: Hash 5MB every frame
let content_hash = compute_content_hash(&self.tab.content);

// After: Length check first, skip hash for large files
if is_large_file {
    // Large file with same length - assume unchanged (fast path)
    (false, existing_hash)
} else {
    // Small file - compute hash to check
    let hash = compute_content_hash(&self.tab.content);
    (hash != existing, hash)
}
```

### 2. Minimap Disabled for Large Files (app.rs)

**Problem**: Minimap data collection iterated entire content:
- `t.content.lines().count()` - O(n) line counting
- `extract_outline_for_file(&t.content)` - O(n) parsing
- `t.content.clone()` - O(n) copy for pixel minimap

**Solution**: Disable minimap for files > 1MB.

```rust
let is_tab_large_file = self.state.active_tab()
    .map(|t| t.is_large_file()).unwrap_or(false);
let minimap_enabled = self.state.settings.minimap_enabled 
    && !zen_mode 
    && !is_tab_large_file;
```

### 3. Outline Hash Optimization (app.rs)

**Problem**: `update_outline_if_needed()` hashed entire content every frame.

**Solution**: Use Tab's `content_version` field (O(1)) instead of content hash.

```rust
// Before: Hash 5MB every frame
tab.content.hash(&mut hasher);

// After: O(1) version check
let change_key = (tab_id as u64)
    .wrapping_mul(31)
    .wrapping_add(content_version)
    .wrapping_mul(31)
    .wrapping_add(path_hash);
```

### 4. CJK Font Detection Caching (app.rs)

**Problem**: `fonts::needs_cjk(&tab.content)` scanned all characters looking for CJK text every frame.

**Solution**: Cache check result using content_version, only re-scan when content changes.

```rust
let check_key = (tab.id as u64)
    .wrapping_mul(31)
    .wrapping_add(tab.content_version());
if check_key != self.last_cjk_check_key {
    self.last_cjk_check_key = check_key;
    if !fonts::are_cjk_fonts_loaded() && fonts::needs_cjk(&tab.content) {
        self.load_cjk_fonts_for_content(ctx, &tab.content);
    }
}
```

### 5. Line Counting Optimization (widget.rs)

**Problem**: `count_lines(&self.tab.content)` iterated all characters for gutter width calculation.

**Solution**: Use fixed gutter width (7 digits) for large files.

```rust
let digit_count = if is_large {
    7  // Fixed width for large files
} else {
    let line_count = count_lines(&self.tab.content);
    (line_count as f32).log10().floor() as usize + 1
};
```

### 6. Auto-Close Brackets Skip (app.rs)

**Problem**: Content was cloned every frame for auto-close bracket detection:
```rust
let (pre_render_content, pre_render_cursor) = self.state.active_tab()
    .map(|tab| (tab.content.clone(), ...))  // 5MB clone at 60fps!
```

**Solution**: Skip auto-close for large files.

```rust
let is_large_file = self.state.active_tab()
    .map(|t| t.is_large_file()).unwrap_or(false);
let auto_close_enabled = self.state.settings.auto_close_brackets 
    && !is_large_file;
```

## Performance Impact

| Fix | Before | After |
|-----|--------|-------|
| Content hash | ~300MB/s hashing | O(1) length check |
| Minimap | O(n) per frame | Disabled for large files |
| Outline hash | O(n) per frame | O(1) version check |
| CJK detection | O(n) per frame | Cached |
| Line counting | O(n) per frame | O(1) fixed width |
| Auto-close | ~300MB/s cloning | Disabled for large files |

## Files Modified

| File | Changes |
|------|---------|
| `src/editor/widget.rs` | Content hash optimization, line count optimization |
| `src/app.rs` | Minimap disable, outline hash, CJK cache, auto-close skip |

## Testing

Test with a 5MB+ file:
```bash
cargo run
```

Expected behavior:
- Smooth scrolling
- No lag when typing
- RAM usage ~100MB (not 2GB)

## Known Limitations

- Auto-close brackets disabled for large files
- Minimap disabled for large files

## Related Documentation

- [FerriteEditor Technical Docs](ferrite-editor.md)
- [Memory Optimization Plan](../planning/memory-optimization.md)
