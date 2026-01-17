# Search-in-Files Result Navigation Highlight

## Overview

When navigating to a search-in-files result, the editor now:
1. Opens the file (or switches to existing tab)
2. Switches to Raw mode (if in Rendered mode)
3. Scrolls to the exact match location
4. Applies a temporary visual highlight on the matched text
5. Positions the cursor at the match location

## Transient Highlight Behavior

The highlight is temporary and automatically disappears when:
- **Scroll**: User scrolls the editor (except the initial programmatic scroll)
- **Edit**: Any text edit occurs in the document
- **Click**: Any mouse click in the editor area

This prevents visual clutter and ensures the highlight is only visible when useful.

## Implementation Details

### Key Components

#### `SearchNavigationTarget` (src/ui/search.rs)
Contains all information needed for navigation:
- `path`: File path to open
- `line_number`: Line number (1-indexed) for scroll positioning
- `char_offset`: Absolute character offset from document start
- `match_len`: Length of the matched text

#### `TransientHighlight` (src/state.rs)
Manages the transient highlight state:
- `range: Option<(usize, usize)>`: Character range to highlight
- `ignore_next_scroll: bool`: Guard flag for programmatic scroll
- Methods: `set()`, `clear()`, `on_scroll()`, `on_edit()`, `on_click()`

#### `Tab` Methods (src/state.rs)
Helper methods on Tab struct:
- `set_transient_highlight(start, end)`
- `clear_transient_highlight()`
- `has_transient_highlight()`
- `transient_highlight_range()`
- `on_scroll_event()`, `on_edit_event()`, `on_click_event()`

#### `EditorWidget` (src/editor/widget.rs)
Renders the transient highlight:
- `.transient_highlight(range)` builder method
- Distinct amber/orange color (different from search and selection)
- Supports single-line and multi-line highlights

### Flow

1. User clicks search result in Search-in-Files panel
2. `SearchPanelOutput.navigate_to` is set with `SearchNavigationTarget`
3. `handle_search_navigation()` in app.rs:
   - Opens/switches to file
   - Switches to Raw mode if needed
   - Sets transient highlight via `tab.set_transient_highlight()`
   - Sets cursor position
   - Schedules scroll via `pending_scroll_to_line`
4. EditorWidget renders highlight with amber color
5. Highlight cleared on scroll/edit/click via event handlers

### Colors

The transient highlight uses a distinct amber/orange color to differentiate from:
- **Find/Replace highlights**: Yellow
- **Text selection**: Theme-dependent blue
- **Multi-cursor selections**: Light blue

| Mode | Color |
|------|-------|
| Dark | `rgba(255, 165, 50, 120)` - Amber |
| Light | `rgba(255, 180, 80, 150)` - Light orange |

## Testing

Manual test cases:
1. Click search result in open file → verify scroll + highlight
2. Click result in closed file → verify file opens, scrolls, highlights
3. Scroll after highlight → verify highlight disappears
4. Edit after highlight → verify highlight disappears
5. Click anywhere after highlight → verify highlight disappears
6. Click multiple results → verify only one highlight at a time
7. In Rendered mode, click result → verify switch to Raw mode + highlight
