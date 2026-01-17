# macOS Intel CPU Optimization

## Overview

This document describes the idle repaint optimization added to address high CPU usage on macOS Intel (x86_64) systems, and the enhanced tiered idle mode added in v0.2.6.

## Problem

Issue [#24](https://github.com/OlaProeis/Ferrite/issues/24) reported that Ferrite exhibited high CPU usage on Intel-based Macs, even when the application was idle. This was caused by egui's default repaint behavior combined with the app's periodic checks (auto-save, git refresh, toast messages, etc.) which could cause unnecessary continuous repainting.

Intel Macs have different power management behavior compared to Apple Silicon, and may keep the CPU at higher frequencies when the app appears "busy" even for minor operations.

**v0.2.6 Update:** Even with the initial 100ms idle interval, users reported ~10% CPU usage when idle. This was further optimized with a tiered idle system.

## Solution

### Initial Solution (v0.2.5)

Added idle repaint optimization in `src/app.rs` that:

1. **Detects idle state**: A `needs_continuous_repaint()` method checks if the app has ongoing activity that requires immediate repaints
2. **Schedules delayed repaints**: When idle, schedules the next repaint for 100ms later instead of continuously at 60fps

### Enhanced Tiered Idle Mode (v0.2.6)

The idle system now uses a **tiered approach** based on user interaction time:

1. **Tracks user interaction**: A `last_interaction_time` field records when the user last interacted (mouse, keyboard, scroll)

2. **Enhanced activity detection**: `needs_continuous_repaint()` now also checks for:
   - Scroll animations running (`sync_scroll_states`)
   - All previous conditions (pipelines, toasts, dialogs)

3. **Tiered idle intervals** via `get_idle_repaint_interval()`:
   - **Light idle** (0-2 seconds since interaction): 100ms interval (~10 FPS)
   - **Deep idle** (2+ seconds since interaction): 500ms interval (~2 FPS)

```rust
fn get_idle_repaint_interval(&self) -> std::time::Duration {
    let idle_duration = self.last_interaction_time.elapsed();
    
    if idle_duration.as_secs() >= 2 {
        std::time::Duration::from_millis(500)  // Deep idle
    } else {
        std::time::Duration::from_millis(100)  // Light idle
    }
}
```

## Impact

| State | FPS | CPU Usage | Use Case |
|-------|-----|-----------|----------|
| Active (animations/dialogs) | 60 | Normal | User actively working |
| Light idle (<2s) | ~10 | Low | Recent interaction, quick response |
| Deep idle (>2s) | ~2 | Minimal (<1%) | App sitting idle |

- **Idle CPU usage**: Reduced from ~10% to <1% when truly idle
- **Responsiveness**: Still immediately responsive to user input
- **Periodic tasks**: Still checked at appropriate intervals (git refresh, auto-save)
- **Active operations**: No change - continuous repainting when needed

## Technical Details

### Code Location

- `src/app.rs`:
  - `needs_continuous_repaint()` - Checks for ongoing activity
  - `get_idle_repaint_interval()` - Returns appropriate interval based on idle time
  - `update_interaction_time()` - Updates last interaction timestamp
  - `had_user_input()` - Detects user input in current frame
  - `last_interaction_time` field - Tracks last user interaction

### Conditions for Continuous Repaint

The app requests continuous repaints when any of these conditions are true:

| Condition | Reason |
|-----------|--------|
| `pipeline_panel.is_running()` | Output streaming needs immediate display |
| `toast_message.is_some()` | Need to check expiry timer |
| `show_recovery_dialog` | User interaction tracking |
| `pending_auto_save_recovery.is_some()` | Recovery dialog pending |
| `show_confirm_dialog` | Modal needs user input |
| `show_error_modal` | Modal needs user input |
| `show_settings` | Modal needs user input |
| `show_about` | Modal needs user input |
| `sync_scroll_states.is_animating()` | Scroll animation in progress |

### User Input Detection

The `had_user_input()` method detects:
- Any keys down
- Mouse button press/down
- Scroll delta (mouse wheel)
- Any input events (key, paste, etc.)

### Animation Time

Animation time is set to 0.0 at startup for instant animations, which also helps with CPU optimization by eliminating animation-related repaints.

## Testing Notes

### Debug Logging

In debug builds, FPS and idle state are logged every 5 seconds:
```
[REPAINT_DEBUG] FPS: 2.0, continuous: false, idle: 5.2s, interval: 500ms, frames: 10
```

### Verification Steps

1. Open Ferrite and let it sit idle for 5+ seconds
2. Check Task Manager/Activity Monitor - CPU should be <1%
3. Move mouse or type - CPU should briefly increase
4. Stop interaction - CPU should drop back to <1% after ~2 seconds

## Related Issues

- [#24](https://github.com/OlaProeis/Ferrite/issues/24) - macOS Intel: High CPU usage, broken sync scroll, wrong window icons

## Version History

- **v0.2.5**: Initial idle repaint optimization (100ms fixed interval)
- **v0.2.6**: Enhanced tiered idle mode (100ms/500ms based on interaction time)
