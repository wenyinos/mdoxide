# Configurable Line Width

## Overview

Task 6 implements a configurable maximum line width setting that constrains and centers text content in the editor viewport. This feature applies to Raw, Rendered, Split, and Zen mode views.

**Reference:** GitHub Issue #15

## Implementation

### Config Model (`src/config/settings.rs`)

Added `MaxLineWidth` enum with the following variants:

```rust
pub enum MaxLineWidth {
    Off,           // No width limit
    Col80,         // 80 characters
    Col100,        // 100 characters (default)
    Col120,        // 120 characters
    Custom(u32),   // Custom pixel width (400-2000px)
}
```

Key methods:
- `to_pixels(char_width: f32) -> Option<f32>` - Converts column count to pixels
- `display_name() -> &'static str` - Returns UI display name
- `description() -> &'static str` - Returns descriptive text
- `is_custom() -> bool` - Checks if this is a custom width

Added `max_line_width` field to `Settings` struct with default value `MaxLineWidth::Col100`.

### Settings UI (`src/ui/settings.rs`)

Added "Maximum Line Width" section in the Editor settings panel:
- ComboBox dropdown for preset options (Off, 80, 100, 120 chars)
- Custom option with DragValue for pixel input (400-2000px range)
- Preset buttons for common custom values (600px, 800px, 1000px)

### Editor Layout

#### Raw/Split Views (`src/editor/widget.rs`)

- Added `max_line_width` field and builder method to `EditorWidget`
- Modified centering logic to use `max_line_width` when not in Zen Mode:
  - Zen Mode uses `zen_max_column_width` (in characters)
  - Otherwise uses `max_line_width` setting
- Applied `content_margin` for centering and `max_content_width_px` for TextEdit width constraint

#### Rendered View (`src/markdown/editor.rs`)

- Added `max_line_width` field and builder method to `MarkdownEditor`
- Updated `with_settings()` to apply `max_line_width` from settings
- Modified `show_rendered_editor()` to wrap content in a centered container:
  - Horizontal layout with left/right margins for centering
  - Vertical container with `set_max_width()` for content constraint

### App Integration (`src/app.rs`)

Added `max_line_width` capture before mutable borrows in:
- Raw mode editor
- Split mode left (raw) editor
- Split mode right (preview) editor
- Rendered mode editor

## Usage

1. Open Settings (Ctrl+,)
2. Navigate to Editor section
3. Find "Maximum Line Width" dropdown
4. Select a preset (80, 100, 120 characters) or Custom
5. If Custom, enter pixel width (400-2000px)

The setting takes effect immediately without restart.

## Interaction with Zen Mode

- **Both Zen Mode and non-Zen Mode use the same `max_line_width` setting**
- The only difference is centering behavior:
  - **Zen Mode**: Content is constrained AND centered horizontally
  - **Non-Zen Mode**: Content is constrained but left-aligned (no centering)
- The legacy `zen_max_column_width` setting is no longer used for width calculation

## Pane Boundary Behavior (Fix from Task 36)

When `max_line_width` is set but the available pane width is smaller than the configured width:
- Content width is capped to the available pane width
- Text wraps at the pane edge instead of overflowing
- This applies to split view panes when resizing the divider

## Test Strategy

| Scenario | Expected Behavior |
|----------|-------------------|
| Set to Off | Full width, left-aligned in all views |
| Set to 80 characters, wide window | Text constrained to ~80 columns, left-aligned |
| Set to 80 characters, Zen Mode ON | Text constrained to ~80 columns, **centered** |
| Switch to Split view | Same width constraint in both panes |
| Narrow the split pane below 80 chars | Text wraps at pane edge (no overflow) |
| Toggle 80/100/120 chars | Width changes immediately without restart |
| Custom pixel width (600px) | Column width ~600px |
| Extreme custom values | Clamped to 400-2000px range |
| App restart | Setting persisted and applied |

## Files Modified

| File | Changes |
|------|---------|
| `src/config/settings.rs` | Added `MaxLineWidth` enum and field |
| `src/ui/settings.rs` | Added Settings UI dropdown and input |
| `src/editor/widget.rs` | Raw/Split view layout logic |
| `src/markdown/editor.rs` | Rendered view width application |
| `src/app.rs` | Pass setting to editor widgets |
