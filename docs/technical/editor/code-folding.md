# Code Folding (Indicators Only)

## Overview

Code folding infrastructure with gutter fold indicators. Currently implements fold region detection and visual indicators in the gutter, but **text hiding is deferred to v0.3.0** as it requires deep integration with egui's TextEdit widget.

**Status:** Partial implementation - indicators work, text hiding deferred.

## Key Files

| File | Purpose |
|------|---------|
| `src/state.rs` | `FoldKind`, `FoldRegion`, `FoldState` data structures |
| `src/editor/folding.rs` | Fold region detection algorithms |
| `src/editor/widget.rs` | Gutter fold indicator rendering |
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

## What's Deferred (v0.3.0)

- ❌ **Text hiding** - Collapsed regions don't hide content
- ❌ **Placeholder lines** - No "... (X lines folded)" display
- ❌ **Cursor interaction** - Auto-expand when cursor enters folded region

### Why Deferred

Actual text hiding requires one of:
1. Custom text layouter that skips folded lines
2. Content filtering with cursor position mapping
3. Virtual document model (view separate from content)

All approaches require significant changes to how egui's TextEdit is used, which would also benefit multi-cursor editing (also deferred to v0.3.0).

## Usage

1. Open any file (fold detection works for Markdown, JSON, YAML, etc.)
2. Look for fold indicators (▼) in the gutter next to foldable regions
3. Click an indicator to toggle collapsed state (turns orange ▶ when collapsed)
4. Use keyboard shortcuts for bulk operations

## Future Improvements (v0.3.0)

- Implement custom text editor widget with line-level control
- Support for hiding folded content
- Placeholder line rendering with fold preview
- Auto-expand on cursor navigation
- Integration with search (skip hidden content or auto-expand matches)
