# Intel Mac Continuous Repaint Investigation

## Status: ROOT CAUSE IDENTIFIED AND FIXED

**Date**: 2026-01-17

## Root Cause Found

The continuous repaint issue was caused by **hover sensing on the scroll area content** in the rendered markdown editor.

### The Problem

In `src/markdown/editor.rs` line 796 (now ~798), the rendered editor's scroll area content returned:

```rust
ui.allocate_response(Vec2::ZERO, egui::Sense::hover())
```

This `Sense::hover()` caused egui to:
1. Continuously track hover state across the entire rendered content area
2. Request immediate repaints whenever the mouse moved over the area
3. Bypass the 100ms throttling in `needs_continuous_repaint()`

### Why Throttling Was Bypassed

The app's throttling works like this:
```rust
if !self.needs_continuous_repaint() {
    ctx.request_repaint_after(std::time::Duration::from_millis(100));
}
```

However, when egui's internal hover tracking calls `ctx.request_repaint()` (immediate), it takes priority over `request_repaint_after(100ms)`. Result: ~60fps instead of ~10fps.

## Fix Applied

Changed the scroll area content response from `Sense::hover()` to `Sense::focusable_noninteractive()`:

```rust
// Before (caused continuous repaints):
ui.allocate_response(Vec2::ZERO, egui::Sense::hover())

// After (fixes the issue):
ui.allocate_response(Vec2::ZERO, egui::Sense::focusable_noninteractive())
```

This removes the hover tracking overhead while maintaining scroll area functionality.

## Additional Changes

### FPS Diagnostic Logging (Debug Builds Only)

Added frame rate tracking to help verify the fix works:

```rust
// In FerriteApp struct (debug builds only)
#[cfg(debug_assertions)]
frame_count: u64,
#[cfg(debug_assertions)]
last_fps_log: std::time::Instant,

// In update() - logs FPS every 5 seconds
[REPAINT_DEBUG] FPS: X.X, needs_continuous_repaint: false, frames: N
```

## Files Modified

1. **`src/markdown/editor.rs`** - Changed `Sense::hover()` to `Sense::focusable_noninteractive()` on scroll area content
2. **`src/app.rs`** - Added FPS diagnostic logging (debug builds only)

## Testing the Fix

To verify the fix works:

1. Build in debug mode: `cargo build`
2. Run the app: `cargo run`
3. Open a markdown file with lists
4. Switch to Rendered mode
5. Leave the app idle for 30 seconds with mouse over the rendered content
6. Check the logs for `[REPAINT_DEBUG]` messages
7. **Expected**: FPS should be ~10 (100ms intervals) not ~60

## Expected Outcomes

- **Idle CPU usage**: Should drop to <5% on Intel Macs
- **Frame rate when idle**: ~10fps (100ms intervals) instead of ~60fps
- **Rendered mode**: Only re-renders when actual interaction occurs

## Related Files

- Analysis document: `docs/technical/platform/intel-mac-cpu-issue-analysis.md`
- Original log file: `docs/ferrite_macos_intel_log.txt` (48,926 lines)
- Log analysis script: `scripts/analyze_log.py`

---

## Original Investigation Context (For Reference)

<details>
<summary>Click to expand original handover prompt</summary>

### What We Knew

1. **Symptom**: In Rendered (WYSIWYG) mode, the app renders at ~60fps continuously instead of throttling to 100ms intervals when idle

2. **Evidence from logs**:
   - 48,926 log lines in 22 seconds = ~2,224 lines/second
   - ~37 frames per second (each frame generates ~60 log lines for list items)
   - The app SHOULD be using `ctx.request_repaint_after(100ms)` when idle (10fps max)

3. **The throttling logic existed** in `src/app.rs` around line 7077:
   ```rust
   if !self.needs_continuous_repaint() {
       ctx.request_repaint_after(std::time::Duration::from_millis(100));
   }
   ```

### Investigation Findings

1. **`needs_continuous_repaint()` was working correctly** - It checks for pipeline running, toast messages, dialogs, etc.

2. **egui was requesting repaints internally** due to:
   - `Sense::hover()` on scroll area content (PRIMARY CAUSE)
   - Multiple hover-sensing elements in the rendered editor

3. **8 instances of hover sensing** in `editor.rs`:
   - `Sense::hover()` on scroll area content
   - Multiple `.hovered()` checks for cursor icon changes
   - `on_hover_text()` calls (tooltips)

</details>
