# Frontmatter Panel

## Overview

Visual editor for YAML frontmatter in markdown files. Displayed as the **"FM" tab** in the right-side outline panel, alongside Outline, Statistics, Links, and Hub tabs. Renders frontmatter key-value pairs as a form with type-aware widgets, supporting bidirectional sync with the raw editor.

## Key Files

- `src/ui/frontmatter_panel.rs` - Content widget, YAML parsing/serialization, form rendering
- `src/ui/outline_panel.rs` - Hosts the FM tab via `OutlinePanelTab::Frontmatter`
- `src/app/mod.rs` - `FrontmatterPanel` field on `FerriteApp`, content update + output handling
- `src/app/types.rs` - `KeyboardAction::ToggleFrontmatter`
- `src/app/keyboard.rs` - Shortcut wiring (`Ctrl+Shift+M` opens outline panel and switches to FM tab)
- `src/ui/ribbon.rs` - `RibbonAction::ToggleFrontmatter`

## Implementation Details

### Architecture

The frontmatter panel follows the same pattern as `BacklinksPanel` — a content widget rendered inside the outline panel's tab system:

- `FrontmatterPanel` struct holds cached state (parsed fields, content hash, input buffers)
- `show_content(ui, is_dark)` renders inside a parent `Ui` (the outline panel tab area)
- `FrontmatterPanelOutput` carries edit results (`new_content`) back to the caller
- `OutlinePanelOutput.frontmatter_new_content` propagates edits to `app/mod.rs`, which applies them to the active tab's content
- `update_from_content(content)` is called before rendering to re-parse if the raw content changed

The FM tab is always visible in the outline panel tab bar. When the active file is not markdown, the tab shows a "not available" message. The `Ctrl+Shift+M` shortcut opens the outline panel (if closed) and switches to the FM tab.

### Frontmatter Extraction

Uses a custom `extract_frontmatter()` function (not the full markdown parser) for efficiency:
- Finds `---\n` at document start
- Scans for closing `\n---\n`
- Returns YAML body and byte offset of the closing delimiter
- Handles BOM, CRLF, and edge cases

### Value Types

`FrontmatterValue` enum maps YAML types to appropriate egui widgets:

| YAML Type | Enum Variant | Widget |
|-----------|-------------|--------|
| String | `String(String)` | `TextEdit::singleline` |
| Boolean | `Bool(bool)` | `egui::Checkbox` |
| Number | `Number(String)` | `TextEdit::singleline` |
| Sequence | `List(Vec<String>)` | Pill/chip tags with add/remove |
| Mapping | `Mapping(String)` | `TextEdit::multiline` (code editor) |
| Null | `Null` | Italic "null" label |

### Bidirectional Sync

- **Raw → Panel**: Content hash comparison each frame; re-parses only when hash changes
- **Panel → Raw**: On any field edit, serializes all fields back to YAML and replaces the frontmatter block in the document content via `replace_frontmatter_in_content()`

### Caching

Content hash (`DefaultHasher`) prevents re-parsing on every frame. The hash is reset to 0 after panel edits to force a re-parse on the next frame (keeps hash in sync with the new content).

## Dependencies Used

- `serde_yaml` 0.9 - YAML parsing and serialization (already in Cargo.toml)
- `chrono` 0.4 - Date formatting for "Add frontmatter" template

## Usage

- **Access**: Click the "FM" tab in the right-side outline/document panel
- **Shortcut**: `Ctrl+Shift+M` opens the outline panel and switches to the FM tab
- **Visibility**: FM tab always visible in tab bar; content only renders for markdown files
- **Empty state**: Shows "Add frontmatter" button to insert a default template (title, date, tags)
- **Test file**: `test_md/test_frontmatter.md`
