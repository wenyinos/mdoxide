# eframe / egui 0.28 → 0.31.1 Upgrade

This document captures the v0.3.0 GUI stack upgrade from `eframe` / `egui`
0.28 to 0.31.1, the breaking API changes encountered, and the migration
patterns we applied across the codebase.

## Why 0.31 (and not 0.32+)

- **Meets the v0.3.0 PRD requirement** (egui 0.31+).
- **0.31.x is the last release before the deprecation of `eframe::App::update`**
  (which moves to a new lifecycle in 0.32) and before the Atoms/Panel
  consolidation rewrite.
- **Avoids the `skrifa` font-backend switch** that landed in 0.32, keeping
  our HarfRust shaping integration stable.

We can pick up 0.32+ as a follow-up once the new App lifecycle and font
pipeline have stabilised in the ecosystem.

## Resolved dependency tree

After the bump:

```
egui            0.31.1
eframe          0.31.1
egui-winit      0.31.1
egui_glow       0.31.1
epaint          0.31.1
emath           0.31.1
ecolor          0.31.1
arboard         3.6.1
image           0.25.9
```

No conflicts with `wgpu`, `winit`, `serde`, `arboard`, or `image`.

## API breaking changes & migration patterns

The vast majority of edits were mechanical type changes. The full list:

### 1. `Rounding` → `CornerRadius` (and `f32` → `u8`)

```rust
// Before
.rounding(egui::Rounding::same(6.0))
egui::Rounding { nw: 6.0, ne: 6.0, sw: 0.0, se: 0.0 }

// After
.corner_radius(egui::CornerRadius::same(6))
egui::CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 }
```

Field rename also applies on `Visuals`:

```rust
// Before
visuals.window_rounding = Rounding::same(8.0);
visuals.menu_rounding   = Rounding::same(4.0);
widget_visuals.rounding = Rounding::same(2.0);

// After
visuals.window_corner_radius = CornerRadius::same(8);
visuals.menu_corner_radius   = CornerRadius::same(4);
widget_visuals.corner_radius = CornerRadius::same(2);
```

### 2. `Margin` fields are now `i8`

```rust
// Before
Margin::same(8.0)
Margin::symmetric(12.0, 8.0)
Margin { left: 80.0, right: 40.0, top: 60.0, bottom: 40.0 }

// After
Margin::same(8)
Margin::symmetric(12, 8)
Margin { left: 80, right: 40, top: 60, bottom: 40 }
```

### 3. `Shadow` uses small integers

```rust
// Before
Shadow {
    offset: egui::vec2(0.0, 4.0),  // Vec2
    blur:   12.0,                  // f32
    spread: 0.0,                   // f32
    color:  Color32::from_black_alpha(40),
}

// After
Shadow {
    offset: [0, 4],                // [i8; 2]
    blur:   12,                    // u8
    spread: 0,                     // u8
    color:  Color32::from_black_alpha(40),
}
```

### 4. `painter.rect_stroke` and `painter.rect` take a `StrokeKind`

```rust
use egui::StrokeKind;

// Before
painter.rect_stroke(rect, 4.0, stroke);
painter.rect(rect, 4.0, fill, stroke);

// After
painter.rect_stroke(rect, 4.0, stroke, StrokeKind::Inside);
painter.rect(rect, 4.0, fill, stroke, StrokeKind::Inside);
```

We use `StrokeKind::Inside` everywhere to preserve the previous visual
behaviour (strokes drawn on the inside of the rect).

### 5. `Frame::none()` → `Frame::new()`

```rust
// Before
egui::Frame::none().fill(bg)
// After
egui::Frame::new().fill(bg)
```

`Frame::rounding(...)` is renamed `corner_radius(...)`.

### 6. `FontDefinitions::font_data` now requires `Arc<FontData>`

```rust
use std::sync::Arc;

// Before
fonts.font_data.insert("inter".into(), FontData::from_static(&BYTES));

// After
fonts.font_data.insert("inter".into(), Arc::new(FontData::from_static(&BYTES)));
```

### 7. `Memory::layer_transforms` is a method, not a field

```rust
// Before
ui.memory(|m| m.layer_transforms.get(&layer_id).copied())
    .unwrap_or_default();

// After
ui.memory(|m| m.layer_transforms(layer_id))
    .unwrap_or_default();
```

### 8. Per-theme `Visuals` slots — first-frame theme bug

This is the most subtle gotcha. egui 0.31 keeps **separate `Visuals`
slots per theme** (one for `Theme::Light`, one for `Theme::Dark`), and
`Context::set_visuals(v)` now writes only into the *currently active*
slot. The active slot is resolved from `theme_preference`, which
defaults to `System` and falls back to `Theme::Dark` until eframe
reports the real OS theme on frame 1.

Symptom: app starts with a half-themed UI (some panels light, some
dark). Toggling Light → Dark in settings fixes it because by then the
system theme is known and `set_visuals` lands in the right slot.

Fix in `src/theme/manager.rs::apply`:

```rust
match self.current_theme {
    Theme::Dark => {
        ctx.set_theme(egui::ThemePreference::Dark);
        ctx.set_visuals_of(egui::Theme::Dark, visuals);
    }
    Theme::Light => {
        ctx.set_theme(egui::ThemePreference::Light);
        ctx.set_visuals_of(egui::Theme::Light, visuals);
    }
    Theme::System => {
        // Populate both slots so the UI is correct regardless of which
        // one egui ends up activating after it learns the system theme.
        ctx.set_theme(egui::ThemePreference::System);
        ctx.set_visuals_of(egui::Theme::Dark, dark::create_dark_visuals());
        ctx.set_visuals_of(egui::Theme::Light, light::create_light_visuals());
    }
}
```

Two key rules: explicitly set `ThemePreference` to align egui's
resolution with the user's choice, and use `set_visuals_of(theme, ..)`
so the visuals always land in the intended slot.

### 8. Deprecations (left as warnings — non-blocking)

These compile but emit deprecation warnings; we'll clean them up
incrementally:

- `ScrollArea::id_source` → `id_salt`
- `ComboBox::from_id_source` → `from_id_salt`
- `Ui::child_ui` → `Ui::new_child`
- `Ui::with_layer_id` → `Ui::scope_builder(UiBuilder::new().layer_id(...))`
- `Ui::allocate_ui_at_rect` → `allocate_new_ui`
- `Frame::none` (still works, alias for `Frame::new`)
- `Frame::rounding` (still works, alias for `corner_radius`)
- `Button::rounding` → `corner_radius`
- `PlatformOutput::copied_text` → `Context::copy_text` /
  `PlatformOutput::commands`

## Validation

| Check                                           | Result |
|-------------------------------------------------|--------|
| `cargo check`                                   | 0 errors, 160 warnings |
| `cargo build` (default features)                | clean |
| `cargo check --no-default-features`             | clean |
| `cargo check --features async-workers`          | clean (after fixing pre-existing missing imports in `src/app/navigation.rs`) |
| `cargo check --features lsp`                    | clean |
| `cargo check --all-features`                    | clean |
| `cargo test`                                    | 1407 passed, 0 failed, 3 ignored |
| HarfRust shaping tests (`editor::ferrite::shaping`) | 24/24 pass — Latin, Arabic contextual shaping, Bengali conjuncts intact |

## Files touched

- Theme: `src/theme/{dark,light}.rs`
- Fonts: `src/fonts.rs`
- App shell: `src/app/{title_bar,mod,navigation}.rs`
- Editor: `src/editor/{find_replace,minimap,widget}.rs`,
  `src/editor/ferrite/{highlights,editor}.rs`
- Markdown: `src/markdown/editor.rs`, `src/markdown/widgets.rs`
- Mermaid renderers: `src/markdown/mermaid/{class_diagram,er_diagram,
  journey,mindmap,sequence,state,timeline,flowchart/render/{edges,nodes,
  subgraphs}}.rs`
- Terminal: `src/terminal/widget.rs`
- UI panels: `src/ui/{about,backlinks_panel,command_palette,format_toolbar,
  frontmatter_panel,nav_buttons,outline_panel,productivity_panel,
  quick_switcher,ribbon,terminal_panel,view_segment,welcome}.rs`

## Cross-platform status

- **Windows 11**: built and tested (1407 unit tests passing).
- **macOS / Linux (X11, Wayland)**: validated via shared, `cfg`-agnostic
  code paths and dependency resolution; full platform smoke tests will
  run in CI as part of the v0.3.0 cross-platform regression matrix.

## Follow-ups

- Migrate remaining deprecated APIs (`id_source` → `id_salt`, `child_ui`
  → `new_child`, `with_layer_id` → `scope_builder`, `copied_text` →
  `Context::copy_text`) in a focused cleanup pass.
- Re-evaluate egui 0.32+ once Atoms/Panel consolidation has settled and
  HarfRust ↔ skrifa coexistence is documented.
