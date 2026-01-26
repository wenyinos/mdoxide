# Line Number Display & Gutter System

## Overview

Line number display for the FerriteEditor, showing line numbers alongside editor content with toggleable visibility. The gutter area is dynamically calculated based on which features are enabled (line numbers and/or fold indicators).

## Key Files

- `src/editor/ferrite/rendering/gutter.rs` - Gutter width calculation and rendering functions
- `src/editor/ferrite/editor.rs` - FerriteEditor struct with `show_line_numbers` and `show_fold_indicators` flags
- `src/editor/widget.rs` - EditorWidget passes settings to FerriteEditor
- `src/config/settings.rs` - `show_line_numbers` setting

## Related Documentation

- [Code Folding](./code-folding.md) - Fold indicators in the gutter
- [Architecture](./architecture.md) - FerriteEditor design overview

## Implementation Details

### Gutter Width Calculation

The gutter width is dynamically calculated based on enabled features:

```rust
pub fn calculate_gutter_width(
    ui: &egui::Ui,
    font_id: &FontId,
    line_count: usize,
    show_line_numbers: bool,
    show_fold_indicators: bool,
) -> f32 {
    // Neither shown → no gutter
    if !show_line_numbers && !show_fold_indicators {
        return 0.0;
    }

    // Only fold indicators → minimal width
    if !show_line_numbers && show_fold_indicators {
        return FOLD_INDICATOR_WIDTH;  // 12.0px
    }

    // Line numbers shown → calculate based on digit count
    let digits = (line_count as f32).log10().floor() as usize + 1;
    let sample = "0".repeat(digits.max(GUTTER_CHARS));  // min 3 chars
    let galley = ui.fonts(|f| f.layout_no_wrap(sample, font_id.clone(), Color32::WHITE));
    let line_num_width = galley.size().x + 8.0;

    // Add fold indicator space if also enabled
    if show_fold_indicators {
        line_num_width + FOLD_INDICATOR_WIDTH
    } else {
        line_num_width
    }
}
```

### Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `GUTTER_CHARS` | 3 | Minimum character slots for line numbers |
| `GUTTER_PADDING` | 8.0 | Padding between gutter and text content |
| `FOLD_INDICATOR_WIDTH` | 12.0 | Dedicated space for fold indicators |

### Gutter Layout Scenarios

| Line Numbers | Fold Indicators | Gutter Width |
|--------------|-----------------|--------------|
| ✓ | ✓ | line_num_width + 12px (fold area) |
| ✓ | ✗ | line_num_width only |
| ✗ | ✓ | 12px (minimal fold indicator area) |
| ✗ | ✗ | 0px (text expands to full width) |

### Rendering Order

Fold indicators and line numbers share the gutter space efficiently:

```rust
// 1. Render fold indicators first (left side of gutter)
if self.show_fold_indicators {
    if let Some(region) = self.fold_state.region_at_line(line_idx) {
        gutter::render_fold_indicator(
            &painter,
            rect.min.x + 2.0,  // Left edge
            y,
            line_height,
            region.collapsed,
            gutter_text_color,
        );
    }
}

// 2. Render line numbers (offset when fold indicators shown)
if self.show_line_numbers {
    let line_num_x = if self.show_fold_indicators {
        rect.min.x + gutter::FOLD_INDICATOR_WIDTH  // After fold area
    } else {
        rect.min.x
    };
    gutter::render_line_number(
        &painter, line_idx, line_num_x, y,
        line_num_width, &font_id, gutter_text_color,
    );
}
```

### Settings Integration

The `show_line_numbers` setting flows from config through EditorWidget to FerriteEditor:

```rust
// In EditorWidget::show()
editor.set_show_line_numbers(self.show_line_numbers);
editor.set_show_fold_indicators(self.show_fold_indicators);

// In app.rs
let show_line_numbers = self.state.settings.show_line_numbers;
EditorWidget::new(tab)
    .show_line_numbers(show_line_numbers && !zen_mode)
    .show_fold_indicators(show_fold_indicators && !zen_mode)
```

## Public API

### Gutter Functions (`gutter.rs`)

| Function | Description |
|----------|-------------|
| `calculate_gutter_width(ui, font_id, line_count, show_line_numbers, show_fold_indicators)` | Calculate total gutter width |
| `render_gutter_background(painter, rect, gutter_width, bg_color, separator_color)` | Draw gutter background and separator |
| `render_line_number(painter, line_idx, x, y, width, font_id, color)` | Render a single line number (right-aligned) |
| `render_fold_indicator(painter, x, y, line_height, is_collapsed, color)` | Render fold indicator (▶/▼) |

### FerriteEditor Methods

| Method | Description |
|--------|-------------|
| `set_show_line_numbers(bool)` | Enable/disable line number rendering |
| `set_show_fold_indicators(bool)` | Enable/disable fold indicator rendering |

### EditorWidget Builder Methods

| Method | Description |
|--------|-------------|
| `.show_line_numbers(bool)` | Pass line numbers setting |
| `.show_fold_indicators(bool)` | Pass fold indicators setting |

## Usage

### Basic Usage

```rust
EditorWidget::new(tab)
    .font_size(14.0)
    .show_line_numbers(true)
    .show_fold_indicators(true)
    .theme_colors(theme_colors)
    .show(ui);
```

### Dynamic Toggle

Changes take effect immediately without app restart:

```rust
// In ribbon action handler
RibbonAction::ToggleLineNumbers => {
    self.state.settings.show_line_numbers = !self.state.settings.show_line_numbers;
    self.state.mark_settings_dirty();
}
```

## Visual Design

### With Both Enabled
```
┌───────────────────────────────────────┐
│▼  1 │ # Heading                       │
│    2 │                                │
│▼  3 │ ## Subheading                   │
│    4 │ Some paragraph text here.      │
│    5 │                                │
│▶  6 │ ```rust  (collapsed)            │
│   10 │ More content after fold        │
└───────────────────────────────────────┘
 ↑  ↑
 │  └─ Line numbers (right-aligned)
 └─ Fold indicators (▼ expanded, ▶ collapsed)
```

### Line Numbers Only
```
┌─────────────────────────────────────┐
│  1 │ # Heading                      │
│  2 │                                │
│  3 │ ## Subheading                  │
└─────────────────────────────────────┘
```

### Fold Indicators Only
```
┌──────────────────────────────────────┐
│▼│ # Heading                          │
│ │                                    │
│▼│ ## Subheading                      │
└──────────────────────────────────────┘
```

### Neither (Maximum Text Width)
```
┌──────────────────────────────────────┐
│ # Heading                            │
│                                      │
│ ## Subheading                        │
└──────────────────────────────────────┘
```

## Tests

Run gutter tests:

```bash
cargo test gutter
```

### Test Coverage (gutter.rs)

- `test_gutter_chars_constant` - Verify GUTTER_CHARS = 3
- `test_gutter_padding_constant` - Verify GUTTER_PADDING = 8.0

## History

- **v0.2.x**: Migrated from egui TextEdit to custom FerriteEditor
- **Task 27**: Added `show_line_numbers` and `show_fold_indicators` toggle support
  - Fixed gray gap bug when fold indicators disabled
  - Dynamic gutter width calculation based on enabled features
  - `GUTTER_CHARS` reduced from 5 to 3 for compact layout
  - `FOLD_INDICATOR_WIDTH` set to 12px
