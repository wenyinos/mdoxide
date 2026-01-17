# Editor Minimap Navigation Panel

## Overview

The minimap is a VS Code-style navigation panel that displays a zoomed-out view of the entire document on the right side of the editor. It provides quick visual orientation and click-to-navigate functionality for large documents.

## Features

### Visual Display
- **Scaled Document Preview**: Shows a compressed representation of the entire document
- **Character Density Visualization**: Line widths represent actual line lengths
- **Simplified Syntax Coloring**: Different colors for:
  - Headings (blue tint)
  - Code block markers (purple)
  - Comments (gray)
  - List items (green)
  - Blockquotes (muted)
  - Links (blue)
  - Empty lines (very light)

### Navigation
- **Click-to-Navigate**: Click anywhere on the minimap to jump to that position
- **Drag Support**: Click and drag to smoothly scroll through the document
- **Viewport Indicator**: Semi-transparent rectangle shows the currently visible region

### Search Integration
- **Search Highlight Visualization**: Shows all search matches as colored indicators on the right edge
- **Current Match Highlighting**: The current search match is shown with a brighter color

### Theme Support
- Full light/dark mode support
- Colors adapt to the current theme automatically

## Settings

### Configuration Options

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `minimap_enabled` | bool | `true` | Whether the minimap is visible |
| `minimap_width` | f32 | `80.0` | Width of the minimap in pixels |

### Width Constraints
- Minimum: 40 pixels
- Maximum: 150 pixels

## Behavior

### Visibility
- Shown in **Raw** and **Split** view modes
- In Split mode, positioned between the editor and the preview pane
- Automatically hidden in **Zen Mode** for distraction-free editing
- Not shown in **Rendered** mode (no raw editor visible)
- Can be toggled via **Settings > Editor > Show Minimap**

### Performance
- Maximum 10,000 lines rendered for performance
- Uses efficient rectangle drawing instead of text rendering
- Scale factor adjusts automatically based on document length

## Technical Implementation

### Files
- `src/editor/minimap.rs` - Core minimap widget implementation
- `src/config/settings.rs` - Minimap settings (minimap_enabled, minimap_width)
- `src/app.rs` - Integration with editor layout

### Key Components

```rust
// Create and show the minimap
let minimap = Minimap::new(&content)
    .width(80.0)
    .scroll_offset(tab.scroll_offset)
    .viewport_height(tab.viewport_height)
    .content_height(tab.content_height)
    .line_height(tab.raw_line_height)
    .theme_colors(theme_colors)
    .search_highlights(&matches)
    .current_match(current_match_idx);

let output = minimap.show(ui);

// Handle navigation
if let Some(scroll_offset) = output.scroll_to_offset {
    tab.pending_scroll_offset = Some(scroll_offset);
}
```

### MinimapOutput
```rust
pub struct MinimapOutput {
    /// Target scroll offset (if user clicked/dragged)
    pub scroll_to_offset: Option<f32>,
    /// Whether the minimap was clicked
    pub clicked: bool,
    /// Whether the minimap is being dragged
    pub dragging: bool,
}
```

## Future Improvements

Potential enhancements for future versions:
1. **Highlight Current Line**: Show cursor position in the minimap
2. **Git Diff Indicators**: Show changed/added/deleted line markers
3. **Fold Region Indicators**: Visualize folded code regions
4. **Tooltip on Hover**: Show line number or content preview
5. ~~**Settings Panel Integration**: Add minimap toggle to settings UI~~ ✅ Done
6. ~~**Split View Support**: Add minimap to split view editor pane~~ ✅ Done

## Keyboard Shortcuts

Currently, the minimap has no dedicated keyboard shortcuts. Navigation is mouse-based:
- **Left Click**: Jump to position
- **Left Click + Drag**: Scroll to follow mouse

## Testing

The minimap includes unit tests for:
- Default settings
- Width clamping (min/max validation)
- Color theme detection
- Line type detection (headings, code, lists, etc.)
- Output defaults

Run tests with:
```bash
cargo test minimap
```
