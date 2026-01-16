# macOS Intel CPU Optimization

## Overview

This document describes the idle repaint optimization added to address high CPU usage on macOS Intel (x86_64) systems.

## Problem

Issue [#24](https://github.com/OlaProeis/Ferrite/issues/24) reported that Ferrite exhibited high CPU usage on Intel-based Macs, even when the application was idle. This was caused by egui's default repaint behavior combined with the app's periodic checks (auto-save, git refresh, toast messages, etc.) which could cause unnecessary continuous repainting.

Intel Macs have different power management behavior compared to Apple Silicon, and may keep the CPU at higher frequencies when the app appears "busy" even for minor operations.

## Solution

Added idle repaint optimization in `src/app.rs` that:

1. **Detects idle state**: A new `needs_continuous_repaint()` method checks if the app has ongoing activity that requires immediate repaints:
   - Pipeline commands running (output streaming)
   - Toast messages displayed (need expiry checking)
   - Recovery dialogs showing
   - Modal dialogs open (settings, about, confirmation, error)

2. **Schedules delayed repaints**: At the end of `update()`, if the app is idle, it schedules the next repaint for 100ms later instead of repainting continuously at 60fps:
   ```rust
   if !self.needs_continuous_repaint() {
       ctx.request_repaint_after(std::time::Duration::from_millis(100));
   }
   ```

## Impact

- **Idle CPU usage**: Reduced from ~60fps continuous repainting to ~10fps when idle
- **Responsiveness**: Still immediately responsive to user input (egui handles input events)
- **Periodic tasks**: Still checked ~10 times per second (auto-save, git refresh, toast expiry)
- **Active operations**: No change - continuous repainting when pipelines run, dialogs open, etc.

## Technical Details

### Code Location

- `src/app.rs`:
  - `needs_continuous_repaint()` method (lines ~732-766)
  - Idle repaint scheduling at end of `update()` (lines ~6887-6896)

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

### Why 100ms?

The 100ms interval (~10fps idle) was chosen as a balance between:
- **Responsiveness**: Quick enough for toast message expiry, auto-save checks
- **Power efficiency**: Slow enough to significantly reduce CPU usage
- **User experience**: Fast enough that any UI updates feel responsive

## Testing Notes

This optimization requires testing on actual Intel Mac hardware to verify CPU usage improvements. On Windows and Apple Silicon Macs, the app should behave identically to before (responsive UI with reasonable CPU usage).

## Related Issues

- [#24](https://github.com/OlaProeis/Ferrite/issues/24) - macOS Intel: High CPU usage, broken sync scroll, wrong window icons

## Version

Added in v0.2.5
