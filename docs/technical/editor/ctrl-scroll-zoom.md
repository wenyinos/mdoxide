# Ctrl+Scroll Wheel Zoom

## Overview

Ctrl+Mouse Wheel zoom support for the editor, mapped to the same `egui::gui_zoom` functions used by the existing Ctrl++/Ctrl+- keyboard shortcuts. Also wires up `ZoomIn`, `ZoomOut`, and `ResetZoom` as proper `ShortcutCommand` variants with customizable key bindings.

## Key Files

- `src/editor/ferrite/editor.rs` - Hover scroll handler detects Ctrl+scroll via raw `MouseWheel` events, calls `egui::gui_zoom::zoom_in/zoom_out`; event loop skips Ctrl+MouseWheel to prevent double-processing as scroll
- `src/editor/ferrite/input/mouse.rs` - `handle_mouse_wheel()` returns `NoChange` when `modifiers.command` is true
- `src/config/settings.rs` - `ShortcutCommand::ZoomIn/ZoomOut/ResetZoom` enum variants, display names, categories, default bindings (Ctrl+=, Ctrl+-, Ctrl+0)
- `src/app/keyboard.rs` - Keyboard shortcut dispatch for zoom actions via `egui::gui_zoom`
- `src/app/types.rs` - `KeyboardAction::ZoomIn/ZoomOut/ResetZoom` variants
- `src/ui/settings.rs` - Zoom commands in shortcut customization UI

## Implementation Details

### Scroll-to-Zoom (editor.rs)

The hover scroll handler checks raw `Event::MouseWheel` events for the `command` modifier (Ctrl on Win/Linux, Cmd on Mac). When detected, it calls `egui::gui_zoom::zoom_in()` or `zoom_out()` based on `delta.y` direction, instead of scrolling. This uses per-event detection (not `smooth_scroll_delta`) to ensure one zoom step per wheel notch rather than multi-frame smoothed zoom.

### Event Loop Guard (editor.rs)

The focused event loop already processes `Event::MouseWheel` for scrolling. When `modifiers.command` is true, the event is skipped to avoid conflicting with the zoom handling above.

### Input Handler Guard (input/mouse.rs)

The `handle_mouse_wheel` function returns `InputResult::NoChange` when `modifiers.command` is true, preventing the input dispatcher from processing Ctrl+scroll as normal scroll.

### Keyboard Shortcuts (keyboard.rs)

`ZoomIn` calls `egui::gui_zoom::zoom_in(ctx)`, `ZoomOut` calls `zoom_out(ctx)`, and `ResetZoom` calls `ctx.set_zoom_factor(1.0)`. These are the same egui functions that the built-in Ctrl++/Ctrl+- use, ensuring identical behavior between keyboard and scroll zoom.

## Usage

- **Ctrl + Scroll Up** - Zoom in (everything gets larger)
- **Ctrl + Scroll Down** - Zoom out (everything gets smaller)
- **Ctrl + =** - Zoom in (keyboard)
- **Ctrl + -** - Zoom out (keyboard)
- **Ctrl + 0** - Reset zoom to 100%

Zoom is global (affects the entire UI via egui's `zoom_factor`). It persists within the session via egui's built-in memory.
