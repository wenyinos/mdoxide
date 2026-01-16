# Split View Mode

**Status**: Implemented (Task 79)  
**Version**: v0.2.1

## Overview

Split view mode provides a side-by-side editing experience with the raw markdown editor on the left and a real-time rendered preview on the right. This allows users to see both the source markdown and the rendered output simultaneously while editing.

## Features

### Three View Modes

The editor now supports three view modes that cycle through with `Ctrl+E`:

1. **Raw Mode** (`📝`): Plain markdown text editor
2. **Split Mode** (`||`): Side-by-side raw editor + rendered preview
3. **Rendered Mode** (`👁`): WYSIWYG rendered editing

### Draggable Splitter

- The vertical splitter between panes can be dragged to resize
- Split ratio is preserved per-tab (0.2 to 0.8 range, default 0.5)
- Splitter shows visual grip handles for easy identification
- Resize cursor appears on hover

### Dual Editable Panes

- Both panes edit the same content - changes sync instantly between them
- Raw editor on the left for markdown source editing
- Rendered view on the right for WYSIWYG editing (same as full Rendered mode)
- Full undo/redo support for edits in either pane
- Each pane scrolls independently

### Independent Scrolling

- Each pane (raw and preview) scrolls independently
- Scroll sync between panes is planned for v0.3.0

### Per-Tab State

Each tab maintains its own:
- View mode (Raw, Split, or Rendered)
- Split ratio (pane width proportion)
- Scroll position per pane

### Session Persistence

- View mode and split ratio are saved per-tab in the session
- Restored when reopening the application
- Split ratio persists even when switching between view modes

## Usage

### Keyboard Shortcut

- **Ctrl+E**: Cycle through view modes (Raw → Split → Rendered → Raw)

### Toolbar

Click the view mode button in the View section of the ribbon:
- `📝` (Raw) → Click to enter Split view
- `||` (Split) → Click to enter Rendered view  
- `👁` (Rendered) → Click to enter Raw view

### Resizing Panes

1. Hover over the center splitter (cursor changes to resize icon)
2. Click and drag left/right to adjust the split ratio
3. Release to set the new ratio

## Integration with Other Features

### Zen Mode

In Zen Mode, split view shows only the raw editor at full width:
- Preview pane is hidden for distraction-free writing
- Split mode setting is preserved
- Exiting Zen Mode restores the split view

### Structured Files (JSON, YAML, TOML)

Split view is not available for structured data files:
- These files cycle directly between Raw and Tree View modes
- Attempting to enter Split mode on a structured file shows Raw mode instead

### Search Highlights

- Search results are highlighted in the raw editor pane
- Search navigation works in split view

### Code Folding

- Fold indicators appear in the raw editor gutter
- Folding works normally in split view

## Technical Implementation

### ViewMode Enum

```rust
pub enum ViewMode {
    Raw,      // Plain text editor
    Rendered, // WYSIWYG rendered view
    Split,    // Side-by-side (raw + preview)
}
```

### Tab State

```rust
pub struct Tab {
    pub view_mode: ViewMode,
    pub split_ratio: f32,  // 0.2 to 0.8, default 0.5
    // ...
}
```

### TabInfo for Persistence

```rust
pub struct TabInfo {
    pub view_mode: ViewMode,
    pub split_ratio: f32,
    // ...
}
```

## Limitations

- Split view not available for structured data files (JSON, YAML, TOML)
- In Zen Mode, only the raw editor is shown (preview hidden)
- No scroll synchronization between panes (planned for v0.3.0)

## Future Enhancements (v0.3.0+)

- Scroll sync between raw editor and preview panes
- Vertical split option (editor on top, preview on bottom)
- Keyboard shortcut to jump between panes

## Files Changed

| File | Changes |
|------|---------|
| `src/config/settings.rs` | Added `Split` to `ViewMode`, added `split_ratio` to `TabInfo` |
| `src/state.rs` | Added `split_ratio` to `Tab`, updated persistence methods |
| `src/app.rs` | Implemented split view layout, splitter drag, scroll sync |
| `src/ui/ribbon.rs` | Updated view mode toggle icons and tooltips |
