# Paragraph Trailing Spaces Fix

## Problem

In rendered/split WYSIWYG mode, users could not add or keep trailing spaces at the end of lines in simple (plain text, no formatting) paragraphs. The spaces would disappear immediately. List items, headings, and click-to-edit formatted paragraphs were unaffected because they already used persistent edit buffers.

## Root Cause

The simple paragraph branch in `render_paragraph_with_structural_keys` called `node.text_content()` every frame, which reconstructs text from the comrak AST. The AST represents soft breaks as `SoftBreak` nodes (which map to a single space) and does not preserve trailing whitespace from the original source. Additionally, `edit_state.add_node()` was called every frame with this AST-derived text, overwriting the `TextEdit` buffer and preventing egui from preserving any trailing spaces the user typed.

## Fix

Applied the same persistent edit buffer pattern already used by headings and list items:

1. **Initialize from raw source**: Use `extract_paragraph_content(source, start_line, end_line)` instead of `node.text_content()`. This reads directly from the source string and preserves trailing whitespace.

2. **Persist in egui memory**: Store the edit buffer in egui temp memory keyed by `"para_edit_buffer"` + `node.start_line`, using `get_temp_mut_or_insert_with`. The buffer is only initialized on first access; subsequent frames reuse the persisted value.

3. **Commit on focus loss**: Track focus state with a `"para_edit_tracking"` temp bool. Only call `update_source_range` when focus transitions from active to lost (`was_editing && !has_focus`). This prevents the AST re-parse from overwriting in-progress edits.

4. **Clear buffer after commit**: Remove the temp buffer after committing so the next edit cycle starts fresh from source.

## Key Files

| File | Change |
|------|--------|
| `src/markdown/editor.rs` | `render_paragraph_with_structural_keys` simple-paragraph branch (~line 2269) |

## Pattern Reference

The heading edit buffer pattern (same file, `heading_edit_buffer_id`) and list item edit buffer pattern (`list_item_edit_buffer`) use the identical approach.
