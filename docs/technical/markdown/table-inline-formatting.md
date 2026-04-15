# Table Inline Formatting

## Problem

Inline markdown formatting (bold, italic, strikethrough, code) was silently stripped from table cells. Two distinct issues:

1. **Serialization loss:** `text_content()` stripped all formatting markers during round-trip serialization, so `~~strikethrough~~` became `strikethrough` when switching views or editing.
2. **No rendered display:** Table cells rendered as plain `TextEdit` widgets with no inline formatting — bold, italic, and strikethrough markers appeared as literal text in the rendered view.

## Fix: Serialization Preservation

Replaced `cell.text_content()` with `serialize_inline_content(cell)` in two call sites in `src/markdown/widgets.rs`:

- **`serialize_table()`** — markdown serialization output
- **`TableData::from_node()`** — editable table data model population

The `serialize_inline_content()` function already existed and correctly handles all inline node types recursively.

## Fix: Rendered View Formatting

Added a click-to-edit display model for table cells:

- **Display mode** (cell not focused): Renders a rich text `Label` using `build_cell_layout_job()` which parses inline markdown and builds an egui `LayoutJob` with proper formatting.
- **Edit mode** (cell focused): Shows the raw `TextEdit` with markdown syntax visible, as before.

### Inline Markdown Parser

`parse_inline_markdown()` is a recursive descent parser that handles:

| Syntax | Rendering | Notes |
|--------|-----------|-------|
| `***text***` | Bold + italic | Checked before `**` to avoid ambiguity |
| `**text**` | Bold | Uses `get_styled_font_family()` for correct font variant |
| `*text*` | Italic | `TextFormat.italics = true` |
| `~~text~~` | Strikethrough | `TextFormat.strikethrough` stroke |
| `` `text` `` | Inline code | Monospace font with background color |

Nesting is fully supported (e.g., `**~~bold struck~~**`, `***~~all three~~***`).

Header row cells render with bold as the base style.

### Editor Font Threading

`editor_font: &EditorFont` is threaded from `render_node` → `render_table()` → `EditableTable.editor_font()` so the table widget uses the correct bold/italic font variants (Inter-Bold, JetBrains-Bold, etc.).

## Files Changed

| File | Change |
|------|--------|
| `src/markdown/widgets.rs` | `serialize_table()` and `TableData::from_node()`: use `serialize_inline_content()` |
| `src/markdown/widgets.rs` | New: `build_cell_layout_job()`, `parse_inline_markdown()`, helper functions |
| `src/markdown/widgets.rs` | `EditableTable`: added `editor_font` field, click-to-edit cell rendering |
| `src/markdown/editor.rs` | `render_table()`: added `editor_font` parameter, passed to `EditableTable` |

## Verification

Tables with inline formatting now render correctly and round-trip:

```markdown
| Style | Example |
|-------|---------|
| Bold | **strong** |
| Italic | *slanted* |
| Strikethrough | ~~removed~~ |
| Bold + italic | ***both*** |
| Nested | **~~bold struck~~** |
| Triple | ***~~all three~~*** |
```
