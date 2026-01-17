# List Editing Bug Fixes (Tasks 64-68)

## Overview

This document describes the fixes implemented for the rendered-mode list editing bugs where clicking list items would select/edit the wrong content, plus verification and undo/redo integration fixes.

## Problems Identified

1. **Frontmatter Line Number Offset**: When parsing markdown files with YAML frontmatter (`---`), comrak returned line numbers as if the frontmatter didn't exist, causing an offset of ~12 lines.

2. **Edit State Persistence**: Simple text list items and headings lost focus after typing one character because the edit buffer was stored in `EditState` which was recreated every frame.

3. **Immediate Rebuilds**: Changes triggered immediate markdown rebuild on every keystroke, destroying TextEdit widget focus.

## Key Files Modified

- `src/markdown/parser.rs` - Frontmatter offset fix
- `src/markdown/editor.rs` - Edit buffer persistence and deferred commits

## Implementation Details

### 1. Frontmatter Line Number Offset Fix

**Location:** `src/markdown/parser.rs`

When comrak parses markdown with YAML frontmatter, it returns line numbers as if the frontmatter doesn't exist. For example, in a file where line 13 contains `# h1 Heading`, comrak reports it as line 1.

**Solution:**

```rust
/// Calculate the line offset caused by frontmatter.
fn calculate_frontmatter_offset(root: &MarkdownNode) -> usize {
    if let Some(first_child) = root.children.first() {
        if let MarkdownNodeType::FrontMatter(content) = &first_child.node_type {
            let content_lines = content.lines().count();
            // Handle delimiter lines based on content
            let delimiter_lines = match (has_start_delimiter, has_end_delimiter) {
                (true, true) => 0,
                (true, false) | (false, true) => 1,
                (false, false) => 2,
            };
            return content_lines + delimiter_lines;
        }
    }
    0
}

/// Recursively adjust all line numbers in the AST.
fn adjust_line_numbers(mut node: MarkdownNode, offset: usize) -> MarkdownNode {
    if !matches!(node.node_type, MarkdownNodeType::FrontMatter(_)) {
        if node.start_line > 0 { node.start_line += offset; }
        if node.end_line > 0 { node.end_line += offset; }
    }
    node.children = node.children.into_iter()
        .map(|child| adjust_line_numbers(child, offset))
        .collect();
    node
}
```

### 2. Edit Buffer Persistence

**Location:** `src/markdown/editor.rs`

**Problem:** `EditState` is recreated from source every frame. When we didn't commit changes immediately, typed characters were lost on the next frame.

**Solution:** Store the edit buffer in egui memory, which persists across frames:

```rust
// Get or initialize the edit buffer from egui memory
let edit_buffer_id = ui.id().with("list_item_edit_buffer").with(start_line);
let mut edit_buffer = ui.memory_mut(|mem| {
    mem.data
        .get_temp_mut_or_insert_with(edit_buffer_id, || editable.text.clone())
        .clone()
});

// Use edit_buffer for TextEdit instead of editable.text
let text_edit = TextEdit::singleline(&mut edit_buffer)
    .id(widget_id)
    // ...

// Update buffer in memory after editing
ui.memory_mut(|mem| {
    mem.data.insert_temp(edit_buffer_id, edit_buffer.clone());
});
```

### 3. Deferred Commits

**Problem:** Setting `editable.modified = true` on every keystroke triggered a full markdown rebuild, which recreated all widgets and caused focus loss.

**Solution:** Only commit changes when focus is lost:

```rust
let edit_tracking_id = ui.id().with("list_item_edit_tracking").with(start_line);

// Track previous focus state
let was_editing = ui.memory(|mem| {
    mem.data.get_temp::<bool>(edit_tracking_id).unwrap_or(false)
});

// Update tracking
ui.memory_mut(|mem| {
    mem.data.insert_temp(edit_tracking_id, has_focus);
});

// Only commit when focus is LOST
if was_editing && !has_focus {
    editable.modified = true;
    update_source_range(source, start_line, end_line, &edit_buffer);
    // Clear edit buffer for next edit
    ui.memory_mut(|mem| {
        mem.data.remove::<String>(edit_buffer_id);
    });
}
```

## Testing

1. Open a markdown file with YAML frontmatter
2. Click on list items - they should now edit the correct item
3. Type multiple characters - they should persist without focus loss
4. Click away to commit changes

## Task 67: Verification of List Editing Behavior

**Status:** Verified ✅

All aspects of the list editing flow were verified through code review:

1. **End-to-end editing flow:** Click → AST node resolution → structural key → `FormattedItemEditState` → editable widget ✅
2. **Single item editing:** Only one list item is in active editing mode at a time (egui focus management) ✅
3. **Nested list handling:** Recursive rendering with depth-aware paths, unique IDs via `para.start_line` + `item_number` ✅
4. **Header isolation:** Headers use different ID prefix (`"formatted_paragraph"`) vs list items (`"formatted_list_item"`) ✅
5. **Formatted spans:** Inline formatting (bold, italic, code) correctly routed, raw markdown shown during edit ✅

**Key Finding:** The structural-keys rendering path (`render_list_item_with_structural_keys`) is currently disabled. The active path (`render_list_item`) has all the fixes applied.

## Task 68: Undo/Redo Integration Fix

**Status:** Fixed ✅

### Problem Found

Rendered mode (MarkdownEditor) and TreeViewer edits were **NOT being recorded to the undo stack**. Ctrl+Z/Ctrl+Y did nothing after editing in rendered mode.

### Root Cause

- `EditorWidget` (raw mode) takes `&mut Tab` and calls `tab.record_edit()` internally
- `MarkdownEditor` and `TreeViewer` only take `&mut String` content reference, cannot access Tab's undo methods

### Solution

Capture content before editing in `app.rs`, call `tab.record_edit()` after if changed:

```rust
// In app.rs, for MarkdownEditor
let content_before = tab.content.clone();
let editor_output = MarkdownEditor::new(&mut tab.content)
    // ... configuration ...
    .show(ui);

if editor_output.changed {
    tab.record_edit(content_before);  // Now recorded for undo!
    debug!("Content modified in rendered editor, recorded for undo");
}

// Same pattern for TreeViewer
```

### Impact

| Mode | Before Fix | After Fix |
|------|------------|-----------|
| Raw (EditorWidget) | ✅ Worked | ✅ Works |
| Rendered (MarkdownEditor) | ❌ Not recorded | ✅ Now recorded |
| TreeViewer | ❌ Not recorded | ✅ Now recorded |

## Architecture Quick Reference

### How Click-to-Edit Works

1. **Click Detection**: egui detects click on rendered list item text
2. **ID Resolution**: Unique widget ID generated from `para.start_line` + `item_number`
3. **State Lookup**: `FormattedItemEditState` retrieved from egui memory using ID
4. **Edit Mode**: TextEdit widget shown with edit buffer from memory
5. **Commit**: On focus loss, changes written to source via `update_source_range()`

### Key Identifiers

| Component | Purpose |
|-----------|---------|
| `para.start_line` | Source line number (with frontmatter offset applied) |
| `item_number` | Position within parent list (0-indexed) |
| `formatted_item_id` | Composite ID: `ui.id().with("formatted_list_item").with(para.start_line).with(item_number)` |

### Common Pitfalls

1. **Off-by-one indexing**: Always use 0-indexed `item_number` within list, but 1-indexed line numbers for source
2. **Frontmatter offset**: Raw comrak line numbers are wrong when YAML frontmatter exists - always use adjusted numbers
3. **Edit state storage**: Never store edit state in `EditState` (recreated per frame) - use egui memory persistence
4. **Immediate rebuilds**: Don't set `modified = true` on every keystroke - defer until focus loss
5. **ID collisions**: Headers use `"formatted_paragraph"` prefix, list items use `"formatted_list_item"` - keep separate

### Related Documentation

- [Click-to-Edit Formatting](./click-to-edit-formatting.md) - `FormattedItemEditState` details
- [WYSIWYG Interactions](./wysiwyg-interactions.md) - Structural keys and editing flow
- [Undo/Redo System](./undo-redo.md) - History integration

## Known Remaining Issues

- First formatted list item (with inline code/bold) has cursor jump to end
- Some edge cases with edit commit timing

## Dependencies

- `comrak 0.22` - Markdown parsing (has frontmatter line number quirk)
- `egui 0.28` - UI framework with memory persistence
