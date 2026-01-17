# CSV Header Row Detection

## Overview

Automatic detection of header rows in CSV/TSV files with heuristic-based analysis and manual override capability. Headers are styled distinctly and rendered with proper column alignment.

## Key Files

- `src/markdown/csv_viewer.rs` - Header detection logic, table rendering, state management
- `src/app.rs` - Status bar header toggle UI

## Implementation Details

### Header Detection Heuristics

The `detect_header_row()` function uses multi-factor scoring to determine if the first row is likely a header:

| Heuristic | Points | Description |
|-----------|--------|-------------|
| No numeric values | +30 | Headers typically don't contain numbers |
| Mostly text | +15 | First row has few numeric values (<30%) |
| Data is more numeric | +25 | Data rows have more numbers than first row |
| Common keywords | +10 each (max 30) | Detects: id, name, date, email, price, etc. |
| Unique values | +10 | All values in first row are distinct |

Score threshold: >33 points = treat as header row.

### Numeric Value Detection

The `is_numeric_value()` helper handles various formats:
- Integers: `123`, `-456`
- Floats: `123.45`, `-0.5`
- Currency: `$100`, `€50.00`, `£25`, `¥1000`
- Percentages: `50%`
- Thousands: `1,234`, `1,234.56`

### State Management

```rust
pub struct CsvViewerState {
    // ... existing fields
    
    /// Auto-detected header status (None = not yet detected)
    detected_has_header: Option<bool>,
    /// Manual override for header row (None = use auto-detected)
    header_override: Option<bool>,
}
```

Key methods:
- `has_headers()` - Returns effective header status (override or detected)
- `toggle_header()` - Toggles the current header state
- `set_header_override(bool)` - Sets manual override
- `clear_header_override()` - Returns to auto-detect mode

### Table Rendering

The table uses `ui.add_sized()` for proper column alignment:

```rust
// Each cell takes exact calculated width
let response = ui.add_sized(
    Vec2::new(cell_width, row_height),
    egui::Label::new(text).sense(Sense::hover()),
);
```

Column widths are calculated from content and clamped between `MIN_COLUMN_WIDTH` (50px) and `MAX_COLUMN_WIDTH` (300px).

### Status Bar Toggle

Located next to the delimiter picker, shows:
- `Headers: ✓` - Header row enabled
- `Headers: ✗` - Header row disabled
- `✓` suffix indicates manual override

Popup menu options:
1. Auto-detect (default)
2. First row is header
3. No header row

## Usage

1. Open any CSV/TSV file in Ferrite
2. Header detection runs automatically on first render
3. Click "Headers: ✓/✗" in status bar to toggle or override
4. Select "Auto-detect" to return to automatic behavior

## Test Coverage

| Test | Description |
|------|-------------|
| `test_detect_header_row_clear_headers` | Text headers with numeric data |
| `test_detect_header_row_numeric_first_row` | All-numeric first row |
| `test_detect_header_row_keyword_headers` | Common header keywords |
| `test_detect_header_row_single_row` | Single row defaults |
| `test_detect_header_row_empty` | Empty data defaults |
| `test_detect_header_row_mixed_content` | Mixed first row |
| `test_is_numeric_value` | Numeric detection patterns |
| `test_csv_viewer_state_header` | State management |
