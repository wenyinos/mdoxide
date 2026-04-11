# Windows Borderless Window Transparent Compositing Fix

## Problem

On certain Windows 10/11 systems with specific GPU drivers (notably Intel HD Graphics 4600), using `with_decorations(false)` causes a rendering offset: the UI is drawn shifted right/down, leaving black bars on the left and top edges. Clicks register at the "correct" (unshifted) positions, so buttons appear unclickable because the visual and input coordinate spaces don't match.

**Key symptoms:**
- Black bars on left and top edges of the window
- UI buttons don't respond to clicks (input offset from visuals)
- Title bar drag area misaligned
- Going fullscreen eliminates the issue entirely
- Affects both portable and MSI installer builds

## Root Cause

This is a known winit bug introduced after winit 0.27.2 (specifically [winit PR #2419](https://github.com/rust-windowing/winit/pull/2419)). When `decorations = false`, Windows DWM still applies an invisible resize border/frame to the window. On certain GPU drivers (especially older Intel integrated graphics), the glow (OpenGL) backend's rendering surface is sized to include this invisible border, but the client area coordinates don't account for the offset.

Tracked in:
- [emilk/egui #2770](https://github.com/emilk/egui/issues/2770) — same GPU, same symptoms
- [rust-windowing/winit #2109](https://github.com/rust-windowing/winit/issues/2109) — size bug with `set_decorations(false)`
- [OlaProeis/Ferrite #112](https://github.com/OlaProeis/Ferrite/issues/112) — our user report

## Fix

Add `.with_transparent(true)` to the `ViewportBuilder` in `src/main.rs`:

```rust
let mut viewport = eframe::egui::ViewportBuilder
    ::default()
    .with_title(APP_NAME)
    .with_app_id("ferrite")
    .with_decorations(false)
    .with_transparent(true)  // <-- fixes DWM compositing offset
    .with_inner_size([window_size.width, window_size.height])
    .with_min_inner_size([400.0, 300.0]);
```

This changes how DWM composites the borderless window, eliminating the invisible border that causes the offset. The window still appears fully opaque because eframe paints an opaque background (via `clear_color` and panel fills) every frame.

This is the same approach used in egui's official [`custom_window_frame` example](https://github.com/emilk/egui/tree/master/examples/custom_window_frame).

## Diagnostic Logging

A one-shot window diagnostic log was added in `src/app/mod.rs` to capture viewport geometry on the first frame:

```
Window diagnostic: screen_rect=..., pixels_per_point=..., inner=..., outer=...
```

This helps verify the fix (inner/outer rects should align with screen_rect) and provides data for debugging if similar issues appear on other hardware.

## Why Not Upgrade egui?

Task 38 (egui 0.28 → 0.31+ upgrade) was originally set as a dependency, but the issue is independent of the egui version — it's a winit `WM_NCCALCSIZE` bug that persists across versions. The `with_transparent(true)` workaround is effective on eframe 0.28 without requiring a major dependency upgrade.

## Files Changed

| File | Change |
|------|--------|
| `src/main.rs` | Added `.with_transparent(true)` to ViewportBuilder |
| `src/app/mod.rs` | Added `window_diagnostic_logged` field + one-shot viewport logging |

## Affected Hardware

The issue is GPU-driver specific. Known affected:
- Intel HD Graphics 4600 (Haswell, driver 20.19.15.4624)

Known unaffected:
- NVIDIA RTX 3060
- Intel HD Graphics 620

The fix (transparent compositing) is harmless on unaffected hardware.
