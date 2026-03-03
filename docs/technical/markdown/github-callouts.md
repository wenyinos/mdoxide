# GitHub-Style Callouts

## Overview

Support for GitHub-style admonition blocks (`> [!NOTE]`, `> [!TIP]`, etc.) with color-coded rendering, custom titles, and collapsible state.

## Key Files

- `src/markdown/parser.rs` - Callout detection, `CalloutType` enum, blockquote-to-callout conversion
- `src/markdown/editor.rs` - Styled rendering with colors, icons, collapse toggle
- `src/markdown/widgets.rs` - Serialization back to markdown syntax

## Syntax

```markdown
> [!NOTE]
> Basic note content

> [!TIP] Custom Title
> Tip with a custom title

> [!WARNING]-
> Collapsed by default, click to expand
```

## Supported Types

| Type | Color | Icon |
|------|-------|------|
| NOTE | Blue | ℹ |
| TIP | Green | 💡 |
| WARNING | Orange | ⚠ |
| CAUTION | Yellow | ◆ |
| IMPORTANT | Red | ❗ |

## Implementation Details

### Parser (`parser.rs`)

- **`CalloutType` enum** with `from_str` (case-insensitive), `display_name()`, and `icon()` methods.
- **`MarkdownNodeType::Callout`** AST node with `callout_type`, `title` (Option), and `collapsed` (bool) fields.
- **Processing order**: `convert_callout_blockquotes` runs *before* `merge_consecutive_blockquotes` to prevent distinct callouts from being merged into one blockquote.
- **`split_and_convert_blockquote`**: Handles a single `BlockQuote` containing multiple `[!TYPE]` markers by splitting it into separate `Callout` nodes.
- **`extract_callout_info`**: Parses the `[!TYPE]`, optional `-` suffix (collapsed), and optional custom title from the first paragraph's text content.

### Renderer (`editor.rs`)

- **`callout_colors()`**: Returns `(border_color, bg_color, title_color)` per type using `Color32::from_rgba_unmultiplied` with low alpha (20-25) for subtle backgrounds.
- **Collapse toggle**: Uses `ui.allocate_rect()` over the title row with `Sense::click()` for reliable click detection. State is persisted via `egui::Id` keyed on `(start_line, end_line)`.
- **Pointer cursor**: Shows `CursorIcon::PointingHand` on hover over the title.
- **ID uniqueness**: Each callout is wrapped in `ui.push_id(("callout_...", start_line, end_line))` to prevent egui widget ID collisions.
- Both `render_callout_with_structural_keys` and `render_callout` implement the same rendering logic for their respective render paths.

### Serializer (`widgets.rs`)

Reconstructs the original markdown syntax from the AST:
- `> [!TYPE]` for basic callouts
- `> [!TYPE]-` for collapsed
- `> [!TYPE] Custom Title` for custom titles

## Tests

8 parser tests in `parser.rs`:
- `test_callout_note_basic` - Basic NOTE callout
- `test_callout_custom_title` - Custom title parsing
- `test_callout_collapsed` - Collapsed state (`-` suffix)
- `test_callout_all_types` - All 5 types recognized
- `test_callout_case_insensitive` - Case-insensitive matching
- `test_regular_blockquote_not_callout` - Regular blockquotes unchanged
- `test_unknown_callout_type` - Unknown types stay as blockquotes
- `test_callout_multiline_content` - Multi-paragraph content

## Known Considerations

- Consecutive callouts (separated by blank lines) are correctly parsed as separate blocks.
- Unknown `[!TYPE]` values are left as regular blockquotes.
- Collapse state persists across frames via egui's persistent data store.
