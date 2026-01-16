# CSV Rainbow Column Coloring

## Overview
Subtle alternating column colors for CSV/TSV tables to help visually track data across wide tables. Uses perceptually uniform colors (Oklch color space) that work well in both light and dark themes.

## Key Files
- `src/markdown/csv_viewer.rs` - Color generation, blending, and cell rendering
- `src/config/settings.rs` - `csv_rainbow_columns` setting
- `src/app.rs` - Status bar toggle button

## Implementation Details

### Color Generation
Colors are generated using the Oklch perceptually uniform color space:
- 12 distinct hues evenly distributed around the color wheel
- Very low chroma (0.04) for subtle, non-distracting tints
- Different lightness values for dark (0.25) and light (0.85) modes
- sRGB values are clamped to ensure valid colors

### Color Blending
Column colors are blended with the row's base background color:
- 35% column color + 65% base row color (alternating even/odd rows)
- Header rows do not get rainbow coloring (solid header background)
- Blending preserves the alternating row pattern while adding column distinction

### Cell Rendering
Cells are rendered using direct painter calls for performance:
- `ui.allocate_exact_size()` reserves cell space
- `ui.painter().rect_filled()` paints the blended background
- `ui.painter().text()` draws the cell text
- No spacing between cells (`item_spacing.x = 0.0`)
- No spacing between rows (`item_spacing.y = 0.0`)

## Dependencies Used
- `palette = "0.7"` - Oklch color space and sRGB conversion

## Usage
Toggle via status bar button "Colors: ○/🌈" when viewing CSV/TSV files in Rendered or Split mode. Setting persists across sessions.

## Configuration
- Setting: `csv_rainbow_columns: bool` (default: `false`)
- Location: User settings, persisted to config file
