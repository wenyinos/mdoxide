# Intel Mac 100% CPU Usage Analysis

## Problem Summary
Users on Intel Macs experience 100% CPU usage when using Ferrite, particularly in Rendered (WYSIWYG) mode.

## Log Analysis Results

From the 48,926-line debug log (`ferrite_macos_intel_log.txt`):

| Metric | Value |
|--------|-------|
| Total log lines | 48,926 |
| Unique log lines | 1,021 |
| Duplication ratio | 47.9x |
| Time span | 22 seconds |
| Avg lines/second | 2,224 |
| Peak messages/sec | 6,367 |

### Module Distribution
- `ferrite::markdown::editor`: **99.9%** (48,850 messages)
- All other modules combined: 0.1%

### Top Repeated Messages
1. `[LIST_ITEM_DEBUG] Rendering item at line N` - 11,871x
2. `[LIST_ITEM_DEBUG] Para at line N` - 10,503x
3. `[LIST_ITEM_DEBUG] Rendering decision at line N` - 10,502x
4. `[LIST_ITEM_DEBUG] Taking simple text path` - 10,502x

## Root Cause

The markdown editor in **Rendered mode** is continuously re-rendering at ~60fps instead of throttling to the intended 100ms intervals. This causes:

1. **Excessive debug logging**: 4-5 debug log lines per list item per frame
2. **High CPU usage**: Continuous parsing and rendering of the entire markdown AST every frame
3. **Intel Mac sensitivity**: Intel Macs may be more CPU-bound for this workload compared to Apple Silicon

### Why the Throttling Isn't Working

The app has throttling logic in `needs_continuous_repaint()` which returns `false` when idle, triggering `ctx.request_repaint_after(100ms)`. However, in Rendered mode:

1. **TextEdit widgets may request repaints**: Multiple `TextEdit` widgets in the rendered view may be causing continuous repaint requests
2. **Hover effects**: egui may be requesting repaints for hover state tracking
3. **Focus tracking**: The editor tracks focus state which may trigger repaints

## Immediate Fixes

### 1. Reduce Debug Logging Verbosity (Quick Win)

The `[LIST_ITEM_DEBUG]` logging in `src/markdown/editor.rs` is extremely verbose. These should be:
- Removed entirely (they were likely added for debugging a specific issue)
- Or changed to `trace!` level instead of `debug!`

```rust
// Current (in src/markdown/editor.rs around line 3076):
debug!("[LIST_ITEM_DEBUG] Rendering item at line {}...", ...);

// Should be:
trace!("[LIST_ITEM_DEBUG] Rendering item at line {}...", ...);
// Or removed entirely
```

### 2. Add Rendered Mode to Throttling Check

The `needs_continuous_repaint()` function should return `false` for Rendered mode when:
- No text input is focused
- No content has changed
- No user interaction is occurring

### 3. Cache Rendered Output

The markdown AST parsing and rendering should be cached and only regenerated when:
- Content changes
- Window resizes
- User scrolls beyond cached region

## Long-term Fixes

### 1. Implement Dirty Flag for Rendered Mode

Track when the rendered view actually needs to be re-rendered:
- Content hash change
- Theme change
- Font size change
- Window width change

### 2. Virtual Scrolling for Large Documents

Only render visible portion of the document plus a small buffer.

### 3. Progressive/Async Rendering

For large documents, render incrementally across multiple frames.

## Testing the Fix

After implementing fixes:
1. Open a markdown file with lists
2. Switch to Rendered mode
3. Leave the app idle for 30 seconds
4. Check CPU usage (should be <5% when idle)
5. Check log output (should be minimal when idle)

## Files to Modify

1. `src/markdown/editor.rs` - Remove/reduce debug logging
2. `src/app.rs` - Update `needs_continuous_repaint()` for rendered mode
3. Consider adding caching layer for rendered markdown

## Priority

**High** - This significantly impacts user experience on Intel Macs and wastes battery/CPU on all platforms.
