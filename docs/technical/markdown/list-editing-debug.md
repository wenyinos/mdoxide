# List Editing Debug Logging

## Overview

This document describes the debug logging added to diagnose the list item click-to-edit bug in rendered (WYSIWYG) mode. The bug causes clicking on list items to select the wrong content element.

## Bug Symptoms

1. **First list item**: Clicking to edit selects the HEADER above instead of the list item
2. **Other list items**: Visual selection appears correct, but edits go to a DIFFERENT item (off-by-one or wrong index)

## Enabling Debug Logging

Run Ferrite with the `RUST_LOG` environment variable set:

### Windows (PowerShell)
```powershell
$env:RUST_LOG="debug"; .\target\debug\ferrite.exe
```

### Windows (CMD)
```cmd
set RUST_LOG=debug && target\debug\ferrite.exe
```

### Linux/macOS
```bash
RUST_LOG=debug ./target/debug/ferrite
```

### Filter to Only List Debug Messages
```bash
RUST_LOG="ferrite::markdown::editor=debug" ./target/debug/ferrite
```

## Debug Log Prefixes

All list-related debug messages are prefixed with `[LIST_DEBUG]` for easy filtering:

```bash
# Filter output with grep/findstr
RUST_LOG=debug ./target/debug/ferrite 2>&1 | grep "\[LIST_DEBUG\]"
```

## Key Log Messages

### Document Structure (Frame Start)
```
[LIST_DEBUG] ========== RENDERED EDITOR FRAME START ==========
[LIST_DEBUG] Document has N top-level nodes
[LIST_DEBUG] Root child [0]: type=Heading, lines=1..1, children=1
[LIST_DEBUG] Root child [1]: type=List, lines=3..6, children=3
[LIST_DEBUG]   List item [0]: lines=3..3, text_preview='First item'
[LIST_DEBUG]   List item [1]: lines=4..4, text_preview='Second item'
```

### List Rendering
```
[LIST_DEBUG] render_list START: type=Bullet, indent=0, node_lines=3..6, child_count=3
[LIST_DEBUG] render_list: processing item child_idx=0, item_number=0, child_lines=3..3
```

### List Item Rendering
```
[LIST_DEBUG] render_list_item START: item_number=0, node_lines=3..3, indent=0, child_count=1
[LIST_DEBUG] render_list_item: para_node found, para_lines=3..3, text_preview='First item'
[LIST_DEBUG] render_list_item: formatted_item_id uses node.start_line=3, has_inline_formatting=false
```

### Click Detection
```
[LIST_DEBUG] CLICK DETECTED on list item: node.start_line=3, para.start_line=3, display_rect=..., item_number=0
[LIST_DEBUG] EDIT MODE ENTERED: formatted_item_id uses node.start_line=3, extracting from para.start_line=3, content='First item'
```

### Content Extraction
```
[LIST_DEBUG] extract_list_item_content: start_line=3, raw_line='- First item', prefix='- ', content='First item'
```

## Diagnosing the Bug

### Symptom: Clicking first list item selects header

Look for mismatched line numbers:
```
[LIST_DEBUG] CLICK DETECTED on list item: node.start_line=1, ...  # Should be 3!
```

If `node.start_line` shows the header's line number instead of the list item's, the ID generation is using the wrong node.

### Symptom: Off-by-one selection

Look for index vs item_number discrepancies:
```
[LIST_DEBUG] render_list: processing item child_idx=0, item_number=0, child_lines=3..3
[LIST_DEBUG] render_list: processing item child_idx=1, item_number=1, child_lines=4..4
```

If `child_idx` and the actual rendered position don't match, there may be an indexing bug.

### Symptom: Wrong content extracted

Check the extraction log:
```
[LIST_DEBUG] extract_list_item_content: start_line=4, raw_line='- Second item', content='Second item'
```

If `start_line` doesn't match the clicked item, the line number mapping is incorrect.

## Key Code Locations

| Function | File | Purpose |
|----------|------|---------|
| `render_list` | `src/markdown/editor.rs` | Entry point for list rendering |
| `render_list_item` | `src/markdown/editor.rs` | Individual item rendering |
| `render_list_with_structural_keys` | `src/markdown/editor.rs` | Alternative rendering path |
| `extract_list_item_content` | `src/markdown/editor.rs` | Content extraction for editing |
| `FormattedItemEditState` | `src/markdown/editor.rs` | Edit state tracking |

## Test File

Use `docs/test-list-editing-bug.md` to reproduce the bug. It contains:
- A header immediately followed by a list (triggers the first-item bug)
- List items with formatting (tests inline formatting path)
- Nested lists (tests nested rendering)
- Numbered lists (tests ordered list rendering)

## Fix Applied (Task 65)

The following issues were identified and fixed:

### Root Cause

The `formatted_item_id` was created using `node.start_line` (the Item node), but content extraction used `para.start_line` (the Paragraph node). If these differed, the ID and content would not match, causing edits to go to the wrong item.

### Fix Details

1. **Moved ID creation inside the formatting block**
   - Previously: `formatted_item_id` was created outside the `has_inline_formatting` block
   - Now: Created inside where `para` is available

2. **Use `para.start_line` consistently**
   - ID now uses `para.start_line` instead of `node.start_line`
   - This matches the line number used for content extraction

3. **Added item_number/item_index to ID**
   - Extra uniqueness guarantee: `ui.id().with("formatted_list_item").with(para.start_line).with(item_number)`
   - Prevents ID collision even if line numbers somehow match

4. **Added explicit IDs to heading TextEdits**
   - `heading_widget_id = ui.id().with("heading_text").with(node.start_line)`
   - Prevents any potential egui ID conflicts between headings and list items

### Changed Files

- `src/markdown/editor.rs`:
  - `render_list_item()` - Fixed ID generation
  - `render_list_item_with_structural_keys()` - Fixed ID generation
  - `render_heading()` - Added explicit TextEdit ID
  - `render_heading_with_structural_keys()` - Added explicit TextEdit ID

### Verification

After the fix, the debug logs should show:
```
[LIST_DEBUG] render_list_item FORMATTED: created ID with para.start_line=3, item_number=0, node.start_line=3
```

The `para.start_line` and the line used for content extraction should always match.
