# Task List Checkbox Rendering

Interactive task list checkboxes in the rendered markdown preview.

## Overview

Task lists (GitHub-style `- [ ]` and `- [x]` syntax) are rendered as interactive egui Checkbox widgets in the rendered and split views. Clicking a checkbox toggles the task state and updates the underlying markdown source.

## Key Files

| File | Purpose |
|------|---------|
| `src/markdown/editor.rs` | Checkbox rendering in `render_list_item()` and `render_list_item_with_structural_keys()` |
| `src/markdown/parser.rs` | Task item detection in AST (`MarkdownNodeType::TaskItem`) |

## Implementation Details

### Rendering

Task list items are detected during AST traversal. When a task item is found:

1. **Checkbox UI**: Renders `egui::Checkbox` instead of ASCII `[ ]`/`[x]` text
2. **Click Handling**: Captures checkbox response with `ui.checkbox(&mut checked, "")`
3. **Source Toggle**: On click, toggles the source line between `[ ]` and `[x]`
4. **State Tracking**: Changes are marked in `edit_state` for markdown rebuild

### Code Pattern

```rust
if is_task {
    let mut checked = task_checked;
    let checkbox_response = ui.checkbox(&mut checked, "");

    if checkbox_response.changed() {
        // Toggle source line
        if let Some(source_line) = source.lines().nth(node.start_line.saturating_sub(1)) {
            let new_line = if task_checked {
                source_line.replace("[x]", "[ ]").replace("[X]", "[ ]")
            } else {
                source_line.replace("[ ]", "[x]")
            };
            update_source_line(source, node.start_line, &new_line);
            
            // Mark modified
            let node_id = edit_state.add_node(...);
            if let Some(editable) = edit_state.get_node_mut(node_id) {
                editable.modified = true;
            }
        }
    }
}
```

### Supported Syntax

- `- [ ]` Unchecked task
- `- [x]` Checked task (lowercase)
- `- [X]` Checked task (uppercase)
- `* [ ]` Alternative bullet style (asterisk)

### Visual Design

- No bullet marker shown for task items (checkbox replaces bullet)
- 2.0px spacing after checkbox for alignment
- Checkbox state syncs bidirectionally with source

## Usage

1. Open a markdown file with task list syntax in rendered or split view
2. Click any checkbox to toggle its state
3. Source markdown updates automatically
4. Raw editor view reflects the change

## Edge Cases Handled

- Mixed lists (tasks and regular bullets) render correctly
- Nested task lists preserve indentation
- Malformed syntax falls back to text rendering
- Case-insensitive `[x]`/`[X]` handling
