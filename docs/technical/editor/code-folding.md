# Code Folding

## Overview

Code folding with gutter fold indicators and content hiding. Supports hierarchical folding for Markdown files (headings, code blocks, lists) and indentation-based folding for code/data files.

**Status:** Fully functional with FerriteEditor.

## Key Files

| File | Purpose |
|------|---------|
| `src/state.rs` | `FoldKind`, `FoldRegion`, `FoldState` data structures |
| `src/editor/folding.rs` | Fold region detection algorithms |
| `src/editor/widget.rs` | Fold state sync between Tab and FerriteEditor |
| `src/editor/ferrite/editor.rs` | Click handling, y-position calculation for hidden lines |
| `src/editor/ferrite/rendering/gutter.rs` | Fold indicator rendering (▶/▼) |
| `src/config/settings.rs` | Folding configuration settings |

## Data Structures

### FoldKind

```rust
pub enum FoldKind {
    Heading(u8),    // Markdown heading level 1-6
    CodeBlock,      // Fenced code blocks (```)
    List,           // List hierarchies
    Indentation,    // Indentation-based (JSON/YAML)
}
```

### FoldRegion

```rust
pub struct FoldRegion {
    pub id: FoldId,
    pub start_line: usize,    // 0-indexed
    pub end_line: usize,      // 0-indexed, inclusive
    pub kind: FoldKind,
    pub collapsed: bool,
    pub preview_text: String, // First ~50 chars for display
}
```

### FoldState

Manages all fold regions for a document:
- `regions: Vec<FoldRegion>` - All detected fold regions
- `dirty: bool` - Whether regions need recomputation
- Methods for toggling, querying, and bulk operations

## Fold Detection

Detection algorithms in `src/editor/folding.rs`:

1. **Markdown Headings** - Headings fold until next heading of same/higher level
2. **Code Blocks** - Fenced code blocks (``` ... ```)
3. **List Hierarchies** - Nested list items based on indentation
4. **Indentation-based** - For JSON/YAML/structured files

Detection is triggered when content changes (dirty flag) and preserves collapsed state across re-detection.

## Gutter Indicators

Visual indicators in the editor gutter:
- **Expanded (▼)** - Down-pointing triangle, default color
- **Collapsed (▶)** - Right-pointing triangle, orange highlight

Click detection on indicators toggles fold state.

## Settings

In `Settings` struct:
```rust
pub folding_enabled: bool,          // Master toggle
pub folding_show_indicators: bool,  // Show gutter indicators
pub fold_headings: bool,            // Detect heading folds
pub fold_code_blocks: bool,         // Detect code block folds
pub fold_lists: bool,               // Detect list folds
pub fold_indentation: bool,         // Detect indentation folds
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+[` | Fold all regions |
| `Ctrl+Shift+]` | Unfold all regions |
| `Ctrl+Shift+.` | Toggle fold at cursor |

## What Works

- ✅ Fold region detection for all types
- ✅ Gutter fold indicators with click toggle
- ✅ Visual state change (triangle direction + color)
- ✅ Fold state persistence across content changes
- ✅ Keyboard shortcuts for fold operations
- ✅ Settings UI for enabling/disabling fold types
- ✅ **Text hiding** - Collapsed regions hide content and space collapses
- ✅ Fold state synced between Tab and FerriteEditor

## Implementation Notes

### Fold Toggle Flow

1. User clicks fold indicator in gutter
2. `FerriteEditor.ui()` detects click in fold indicator area
3. `y_to_line()` converts click position to line number
4. `fold_state.toggle_at_line()` toggles the fold in FerriteEditor
5. `widget.rs` syncs fold state back to Tab: `tab.fold_state = editor.fold_state().clone()`

**Important:** The toggle happens in FerriteEditor and is synced to Tab. Do NOT toggle again in app.rs.

### Hidden Line Handling

When rendering, hidden lines (inside collapsed folds) are handled by:
1. Y-position calculation skips hidden lines (they don't add height)
2. Render loop skips hidden lines with `continue`
3. Hidden lines still get y-positions (for vector indexing) but don't occupy space

```rust
// In y-position calculation
if !self.fold_state.is_line_hidden(line_idx) {
    y += self.view.get_line_height(line_idx);
}

// In render loop
if self.fold_state.is_line_hidden(line_idx) {
    continue;
}
```

### Hierarchical Folding

Folds are hierarchical - outer folds contain inner folds:
- Folding an outer region hides all nested content
- Inner folds can be toggled independently when outer is expanded

Example for Rust code with indentation-based folding:
- `impl` block (indent 0) contains entire implementation
- `fn` (indent 4) contains function body  
- `match` (indent 8) contains match arms
- Folding the `impl` hides everything inside

## Future Improvements

- ⏳ **Placeholder lines** - "... (X lines folded)" display
- ⏳ **Cursor interaction** - Auto-expand when cursor enters folded region
- ⏳ **Scroll accounting** - Scroll position doesn't fully account for hidden lines yet
- ⏳ **Bracket-based folding** - More intuitive for code files than pure indentation

## Usage

1. Open any file (fold detection works for Markdown, JSON, YAML, etc.)
2. Enable fold indicators: Settings > Editor > Code Folding > Show Fold Indicators
3. Look for fold indicators (▼) in the gutter next to foldable regions
4. Click an indicator to toggle collapsed state (turns ▶ when collapsed, content hides)
5. Use keyboard shortcuts for bulk operations

## Testing

```bash
cargo build
cargo run
```

1. Open a Markdown file with headings - fold indicators appear
2. Click a fold indicator - content should collapse and space should shrink
3. Click again - content should expand
4. Test keyboard shortcuts (Ctrl+Shift+[/]/.)
