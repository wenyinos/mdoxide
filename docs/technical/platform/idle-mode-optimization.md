# Idle Mode CPU Optimization

## Overview

This document describes Ferrite's idle mode system that significantly reduces CPU usage when the application is not being actively used. This optimization applies to **all platforms** (Windows, Linux, macOS).

## Problem

GUI applications using immediate-mode rendering (like egui) typically repaint at 60 FPS continuously, even when nothing is changing on screen. This results in:

- **~10% CPU usage** even when the app is completely idle
- Unnecessary battery drain on laptops
- Higher power consumption and heat generation
- Poor behavior for an app that users often leave open in the background

## Solution: Tiered Idle Mode

Ferrite implements a **tiered idle repaint system** that adjusts the refresh rate based on user activity:

### Repaint States

| State | Condition | Interval | FPS | CPU |
|-------|-----------|----------|-----|-----|
| **Active** | Animations, dialogs, pipelines running | Continuous | ~60 | Normal |
| **Light Idle** | No activity, but recent interaction (<2s) | 100ms | ~10 | Low |
| **Deep Idle** | No interaction for 2+ seconds | 500ms | ~2 | Minimal (<1%) |

### How It Works

1. **Interaction Tracking**: Every frame, the app checks for user input:
   - Keyboard keys pressed
   - Mouse buttons clicked/held
   - Mouse wheel scrolling
   - Any input events (paste, etc.)

2. **Activity Detection**: The `needs_continuous_repaint()` method checks for ongoing activity:
   - Pipeline commands running (output streaming)
   - Toast messages displayed (expiry checking)
   - Modal dialogs open (settings, about, confirmation, error)
   - Recovery dialogs showing
   - Scroll animations in progress

3. **Tiered Intervals**: When no continuous activity is needed:
   ```rust
   fn get_idle_repaint_interval(&self) -> Duration {
       let idle_duration = self.last_interaction_time.elapsed();
       
       if idle_duration.as_secs() >= 2 {
           Duration::from_millis(500)  // Deep idle: ~2 FPS
       } else {
           Duration::from_millis(100)  // Light idle: ~10 FPS
       }
   }
   ```

## Implementation Details

### Code Location

All idle mode logic is in `src/app.rs`:

| Method | Purpose |
|--------|---------|
| `last_interaction_time` | Field tracking last user interaction |
| `needs_continuous_repaint()` | Checks if continuous repainting is needed |
| `get_idle_repaint_interval()` | Returns appropriate interval based on idle time |
| `update_interaction_time()` | Updates the interaction timestamp |
| `had_user_input()` | Detects user input in current frame |

### Continuous Repaint Conditions

The app repaints continuously (60 FPS) when any of these are true:

- `pipeline_panel.is_running()` - Command output streaming
- `toast_message.is_some()` - Toast expiry checking
- `show_recovery_dialog` - Crash recovery dialog
- `pending_auto_save_recovery.is_some()` - Auto-save recovery pending
- `show_confirm_dialog` - Confirmation dialog open
- `show_error_modal` - Error modal open
- `show_settings` - Settings panel open
- `show_about` - About/Help panel open
- `sync_scroll_states.is_animating()` - Scroll animation running

### Animation Time

Animation time is set to 0.0 at startup for instant UI transitions. This eliminates animation-related repaints and makes the UI feel snappier.

## Impact

### Before (v0.2.5)
- Light idle: ~10% CPU (100ms fixed interval)
- No deep idle mode

### After (v0.2.6)
- Light idle: ~10 FPS, low CPU
- Deep idle: ~2 FPS, **<1% CPU**

## Testing

### Debug Logging

In debug builds, idle state is logged every 5 seconds:
```
[REPAINT_DEBUG] FPS: 2.0, continuous: false, idle: 5.2s, interval: 500ms, frames: 10
```

### Manual Verification

1. Open Ferrite and let it sit idle for 5+ seconds
2. Open Task Manager (Windows), Activity Monitor (macOS), or `top` (Linux)
3. CPU usage should be near 0% (typically <1%)
4. Move mouse or type - CPU briefly increases
5. Stop interaction - CPU drops back after ~2 seconds

## Design Decisions

### Why 2 seconds for deep idle?

- Short enough that users won't notice the transition
- Long enough to avoid thrashing between light and deep idle during normal pauses
- Allows UI elements like toast messages to update promptly

### Why 500ms for deep idle?

- Slow enough to reduce CPU to near-zero
- Fast enough that periodic tasks (git refresh, auto-save checks) still run reasonably often
- UI still feels responsive when user returns to the app

### Why 100ms for light idle?

- Quick enough for responsive UI feedback
- Slow enough to significantly reduce CPU vs 60 FPS
- Good balance for the "just stopped interacting" state

## Version History

- **v0.2.5**: Initial idle optimization (100ms fixed interval) - targeted macOS Intel
- **v0.2.6**: Enhanced tiered idle mode (100ms/500ms) - all platforms
