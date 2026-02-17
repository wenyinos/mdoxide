# Wikilinks

## Overview

Wikilinks (`[[target]]` and `[[target|display text]]`) allow linking between Markdown files using a lightweight syntax. Clicking a wikilink in rendered or split view opens the target file in a new or existing tab. Broken links (target file not found) are visually distinguished with dimmed red styling and strikethrough.

## Key Files

- `src/markdown/parser.rs` - AST node type (`MarkdownNodeType::Wikilink`) and post-processing extraction from `Text` nodes
- `src/markdown/editor.rs` - `render_wikilink()` function, `WikilinkContext` struct, broken link detection, click handling via `egui::Memory`
- `src/markdown/mod.rs` - Re-exports `WikilinkContext`
- `src/app/file_ops.rs` - `navigate_wikilink()`, `resolve_wikilink_target()`, recursive file search with tie-breakers
- `src/app/central_panel.rs` - Passes `WikilinkContext` to `MarkdownEditor`, handles `wikilink_clicked` output events

## Implementation Details

### Parsing

Comrak does not natively support wikilinks, so they are handled as a **post-processing step** on the AST. After Comrak parses the Markdown into `MarkdownNode` trees, `extract_wikilinks()` recursively walks the AST and finds `Text` nodes containing `[[...]]` patterns. These are split into sequences of `Text` and `Wikilink` nodes.

The `Wikilink` AST variant:

```rust
Wikilink {
    target: String,          // e.g. "My Note"
    display: Option<String>, // e.g. Some("Click here")
}
```

Supported syntax:
- `[[target]]` - links to `target.md`, displays "target"
- `[[target|display text]]` - links to `target.md`, displays "display text"

Edge cases handled as plain text:
- `[[]]` (empty target)
- `[[not closed` (unclosed brackets)

### Rendering

Wikilinks are rendered as clickable `egui::Label` widgets in the rendered and split views:

- **Normal links**: Blue/accent colored text with hand cursor on hover
- **Broken links**: Dimmed red with strikethrough, indicating the target file was not found
- **Tooltip**: Shows target path and resolution status on hover
- **Click**: Stores target in `egui::Memory` under `Id::new("wikilink_clicked_target")`, which is read by `MarkdownEditorOutput.wikilink_clicked`

The `WikilinkContext` struct provides current directory and workspace root to the renderer via `egui` temporary memory, enabling lightweight file-exists checks at render time without threading filesystem state through the deep render call chain.

### File Resolution

`resolve_wikilink_target()` in `file_ops.rs` implements the resolution hierarchy:

1. **Relative to current file's directory** - `current_dir/target.md`
2. **Relative to workspace root** - `workspace_root/target.md`
3. **Recursive search** of workspace with tie-breakers:
   - Same-folder-first priority
   - Shortest path wins
   - First match returned if still ambiguous

Supports spaces in filenames (e.g., `[[My Note]]` resolves to `My Note.md`).

### Navigation

When a wikilink is clicked:
1. `central_panel.rs` captures the target from `MarkdownEditorOutput.wikilink_clicked`
2. After UI rendering completes (to avoid borrow conflicts), calls `FerriteApp::navigate_wikilink()`
3. The method resolves the target path and calls `state.open_file()` to open in a new or existing tab
4. If resolution fails, an error toast is displayed

## Dependencies Used

No new dependencies. Uses existing:
- `comrak` - Base Markdown parsing (wikilinks added as post-processing)
- `egui` - Rendering and `Memory` for cross-component communication

## Usage

Open `test_md/test_wikilinks.md` in Ferrite's rendered or split view to test all wikilink scenarios. The test file includes:
- Basic wikilinks and display text variants
- Spaces in filenames
- Broken/missing link indicators
- Edge cases (empty, unclosed)
- Wikilinks in lists, blockquotes, and formatted text

## Tests

7 unit tests in `src/markdown/parser.rs`:
- `test_parse_simple_wikilink`
- `test_parse_wikilink_with_display_text`
- `test_parse_wikilink_with_spaces`
- `test_parse_multiple_wikilinks`
- `test_parse_wikilink_text_content`
- `test_parse_unclosed_wikilink`
- `test_parse_empty_wikilink`

Run with: `cargo test parser::tests::test_parse`
