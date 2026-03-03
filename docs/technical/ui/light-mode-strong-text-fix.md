# Light Mode Strong Text Fix

## Overview
Fixed invisible text in light mode caused by `RichText::strong()` labels rendering as white-on-white. This affected all section headers in the Settings panel, Terminal panel, Files section, About panel, and any other UI that used `.strong()` styled text.

## Root Cause

egui's `RichText::strong()` method uses `Visuals::strong_text_color()` to determine the text color. This method returns `visuals.widgets.active.fg_stroke.color` — **not** `visuals.override_text_color` or the general text color.

In `src/theme/light.rs`, the active widget foreground stroke was set to `Color32::WHITE`:

```rust
visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);
```

This was originally intended for the pressed/active button state (white text on an accent-colored background). However, because egui reuses this field for `strong_text_color()`, **all** `.strong()` labels everywhere in the app became white — invisible on light backgrounds.

## Key Files
- `src/theme/light.rs` — The fix was applied here
- `src/theme/dark.rs` — Dark theme was unaffected (white text is visible on dark backgrounds)
- `src/theme/manager.rs` — Applies visuals via `ctx.set_visuals()`

## Fix Applied

Changed `active.fg_stroke` color from `Color32::WHITE` to `colors.text.primary` (a dark gray/near-black color):

```rust
// NOTE: `active.fg_stroke.color` is also returned by `Visuals::strong_text_color()`
// which egui uses for `RichText::strong()`. Using WHITE here would make all
// `.strong()` labels invisible on light backgrounds. We use the primary text
// color which has good contrast on both the accent bg_fill and light panels.
visuals.widgets.active.fg_stroke = Stroke::new(2.0, colors.text.primary);
```

## Why This Is Safe

- **Active buttons**: The accent `bg_fill` still provides sufficient contrast with dark text
- **Dark theme**: Unaffected — its `active.fg_stroke` was already white, which works on dark backgrounds
- **Other UI**: `welcome.rs` and `format_toolbar.rs` use explicit color settings (`.color(text_color)` or `is_dark` checks), so they were not affected by this bug

## Key Insight

egui's `Visuals::strong_text_color()` is implicitly derived from `widgets.active.fg_stroke.color`. This coupling is not obvious from the API and is a common source of theme bugs. When customizing egui themes, always check that `active.fg_stroke` provides good contrast against both:

1. The active widget `bg_fill` (for actual pressed buttons)
2. General panel backgrounds (for `RichText::strong()` usage everywhere)

## Affected Components

| Component | Symptom |
|-----------|---------|
| Settings panel section headers | Invisible labels ("Appearance", "Editor", etc.) |
| Terminal panel headers | Invisible section titles |
| Files section headers | Invisible labels |
| About panel | Missing strong text |
| Any `RichText::new(...).strong()` usage | White-on-white text |

## Dependencies
- egui 0.28 — `Visuals::strong_text_color()` behavior
