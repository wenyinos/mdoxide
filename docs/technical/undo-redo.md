# Undo/Redo System

## Overview

Ferrite implements a per-tab undo/redo system that tracks content changes and cursor positions, allowing users to navigate through their editing history using keyboard shortcuts with full state restoration.

## Architecture

### Per-Tab State

Each `Tab` maintains its own independent undo/redo history:

```rust
/// An entry in the undo/redo stack.
/// Stores both content state and cursor position.
pub struct UndoEntry {
    pub content: String,
    pub cursor_position: usize,
}

struct Tab {
    // ... other fields
    undo_stack: Vec<UndoEntry>,  // Stack of previous states
    redo_stack: Vec<UndoEntry>,  // Stack of undone states for redo
    max_undo_size: usize,        // Maximum history size (default: 100)
}
```

### Storage Model

The system uses **full content snapshots** with cursor position rather than incremental deltas:
- **Pros**: Simple, reliable, works with any edit type, perfect cursor restoration
- **Cons**: Higher memory usage for large documents
- **Trade-off**: For typical markdown documents (<100KB), memory impact is minimal

### Maximum History

The `max_undo_size` limits the undo stack to 100 entries by default. When exceeded, the oldest entries are removed (FIFO).

## Implementation Details

### Recording Edits

Because egui's `TextEdit` modifies content directly (bypassing `Tab::set_content()`), a separate method records edits:

```rust
impl Tab {
    /// Record an edit after TextEdit modifies content directly.
    /// Call with the OLD content and OLD cursor position (before the edit).
    pub fn record_edit(&mut self, old_content: String, old_cursor: usize) {
        if old_content != self.content {
            self.undo_stack.push(UndoEntry::new(old_content, old_cursor));
            if self.undo_stack.len() > self.max_undo_size {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();  // New edits invalidate redo
        }
    }
}
```

### EditorWidget Integration (Raw Mode)

The `EditorWidget` captures content AND cursor before showing `TextEdit`, then records if changed:

```rust
// Before TextEdit
let original_content = self.tab.content.clone();
let original_cursor = self.tab.cursors.primary().head;

// Show TextEdit (may modify content)
let text_output = text_edit.show(ui);

// After TextEdit - record for undo if changed
if self.tab.content != original_content {
    self.tab.record_edit(original_content, original_cursor);
}
```

### MarkdownEditor and TreeViewer Integration (Rendered Mode)

Unlike `EditorWidget`, the `MarkdownEditor` (WYSIWYG mode) and `TreeViewer` (JSON/YAML/TOML) only receive `&mut String` content references, not the full `Tab`. Recording must be done at the app level:

```rust
// In app.rs - for MarkdownEditor
let content_before = tab.content.clone();
let cursor_before = tab.cursors.primary().head;
let editor_output = MarkdownEditor::new(&mut tab.content)
    // ... configuration ...
    .show(ui);

if editor_output.changed {
    tab.record_edit(content_before, cursor_before);
}

// Same pattern for TreeViewer
let content_before = tab.content.clone();
let cursor_before = tab.cursors.primary().head;
let output = TreeViewer::new(&mut tab.content, file_type, tree_state)
    .show(ui);

if output.changed {
    tab.record_edit(content_before, cursor_before);
}
```

### Content Version for External Changes

egui's `TextEdit` maintains internal state keyed by widget ID. When content changes externally (via undo/redo), the `TextEdit` doesn't automatically detect this. The solution is a `content_version` counter:

```rust
struct Tab {
    // ... other fields
    content_version: u64,  // Incremented on undo/redo
}
```

The `EditorWidget` includes this version in the TextEdit's ID (but NOT in the ScrollArea's ID):

```rust
let base_id = self.id.unwrap_or_else(|| ui.id().with("editor"));
let id = base_id.with(self.tab.content_version());

// TextEdit uses `id` (with content_version) - forces re-read on undo/redo
let text_edit = TextEdit::multiline(content).id(id)...

// ScrollArea uses `base_id` (stable) - preserves scroll position on undo/redo
let scroll_area = ScrollArea::vertical().id_source(base_id.with("scroll"))...
```

When `undo()` or `redo()` is called, the version increments, causing egui to treat the TextEdit as a new widget and re-read the content from the source string. The ScrollArea uses a stable ID to preserve scroll position.

### Undo/Redo Operations

The operations now return the cursor position from the stored entry:

```rust
impl Tab {
    /// Undo the last edit.
    /// Returns `Some(cursor_position)` if undo was performed.
    pub fn undo(&mut self) -> Option<usize> {
        if let Some(entry) = self.undo_stack.pop() {
            // Save current state to redo stack
            let current_cursor = self.cursors.primary().head;
            self.redo_stack.push(UndoEntry::new(self.content.clone(), current_cursor));
            // Restore previous state
            self.content = entry.content;
            self.content_version = self.content_version.wrapping_add(1);
            Some(entry.cursor_position)
        } else {
            None
        }
    }

    /// Redo the last undone edit.
    /// Returns `Some(cursor_position)` if redo was performed.
    pub fn redo(&mut self) -> Option<usize> {
        if let Some(entry) = self.redo_stack.pop() {
            // Save current state to undo stack
            let current_cursor = self.cursors.primary().head;
            self.undo_stack.push(UndoEntry::new(self.content.clone(), current_cursor));
            // Restore next state
            self.content = entry.content;
            self.content_version = self.content_version.wrapping_add(1);
            Some(entry.cursor_position)
        } else {
            None
        }
    }
}
```

### Scroll, Focus, and Cursor Preservation

When undo/redo is triggered, the handler preserves and restores the user's editing context:

```rust
fn handle_undo(&mut self) {
    if let Some(tab) = self.state.active_tab_mut() {
        // Preserve scroll position before undo
        let current_scroll = tab.scroll_offset;
        
        // Perform undo - returns cursor from the undo entry
        if let Some(restored_cursor) = tab.undo() {
            // Restore scroll position
            tab.pending_scroll_offset = Some(current_scroll);
            // Request focus on the new widget (ID changed due to content_version)
            tab.needs_focus = true;
            // Restore cursor to the position from the undo entry
            let new_len = tab.content.len();
            tab.pending_cursor_restore = Some(restored_cursor.min(new_len));
        }
    }
}
```

The `EditorWidget` restores the cursor position after showing the TextEdit:

```rust
// Capture pending cursor before the closure
let pending_cursor = self.tab.pending_cursor_restore.take();

// ... show TextEdit ...

// After TextEdit, restore cursor if pending
if let Some(cursor_pos) = pending_cursor {
    let ccursor = egui::text::CCursor::new(cursor_pos);
    let cursor_range = egui::text::CCursorRange::one(ccursor);
    text_output.state.cursor.set_char_range(Some(cursor_range));
    text_output.state.store(ui.ctx(), id);
    
    // Update internal cursor tracking
    self.tab.cursors.set_single(Selection::cursor(cursor_pos));
    self.tab.sync_cursor_from_primary();
}
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Z` | Undo last edit |
| `Ctrl+Y` | Redo undone edit |
| `Ctrl+Shift+Z` | Redo undone edit (alternative) |

### Event Consumption (Critical)

egui's `TextEdit` widget has **built-in undo/redo functionality**. To prevent conflicts between our custom undo system and TextEdit's internal undo, we must:

1. **Consume events BEFORE rendering** - Call `consume_key()` before `render_ui()`
2. **Use `consume_key()` not `key_pressed()`** - Prevents TextEdit from seeing the event

```rust
// In update() - BEFORE render_ui()
fn consume_undo_redo_keys(&mut self, ctx: &egui::Context) {
    ctx.input_mut(|i| {
        // Ctrl+Shift+Z: Redo (check first - more specific)
        if i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::Z) {
            // Handle redo
        }
        // Ctrl+Z: Undo
        if i.consume_key(egui::Modifiers::CTRL, egui::Key::Z) {
            // Handle undo
        }
        // Ctrl+Y: Redo
        if i.consume_key(egui::Modifiers::CTRL, egui::Key::Y) {
            // Handle redo
        }
    });
}

// Call order in update():
self.consume_undo_redo_keys(ctx);  // Consume BEFORE render
let deferred_format = self.render_ui(ctx);  // TextEdit never sees Ctrl+Z
self.handle_keyboard_shortcuts(ctx);  // Other shortcuts
```

Without proper timing and event consumption, pressing Ctrl+Z would trigger BOTH our undo AND TextEdit's internal undo, causing unpredictable behavior like:
- Double-undoing
- Change appearing to "blink" (undo then immediately redo)
- Focus loss
- Cursor jumping to end of document

## User Feedback

The system provides visual feedback via toast notifications:
- **Successful undo**: "Undo (N remaining)" where N is remaining undo count
- **Successful redo**: "Redo (N remaining)" where N is remaining redo count
- **Empty stack**: "Nothing to undo" or "Nothing to redo"

Toast messages display for 1.5 seconds.

## Behavior Notes

### Redo Stack Clearing

The redo stack is **cleared** whenever a new edit is made. This is standard behavior - you cannot redo after making new changes:

```
Initial: "Hello"
Edit:    "Hello World"  → undo_stack: [("Hello", 5)]
Undo:    "Hello"        → redo_stack: [("Hello World", 11)], cursor at 5
Edit:    "Hello!"       → redo_stack: CLEARED
```

### Tab Independence

Each tab maintains completely independent undo/redo history:
- Switching tabs preserves each tab's history
- Closing a tab discards its history
- Opening a file starts with empty history

### Save Interaction

Saving a file does **not** clear the undo history. You can still undo after saving.

## Testing

Unit tests cover:
- Basic undo/redo operations (`test_tab_undo_redo`)
- `record_edit` for external modifications (`test_tab_record_edit`)
- Cursor position restoration on undo
- Redo clearing on new edit (`test_tab_undo_clears_redo_on_edit`, `test_tab_record_edit_clears_redo`)
- Stack count tracking (`test_tab_undo_redo_counts`)
- Maximum size enforcement (`test_tab_max_undo_size`)
- No-op for unchanged content (`test_tab_record_edit_no_change`)

## Future Considerations

Potential improvements for future versions:
1. **Delta-based storage**: Store diffs instead of full content for memory efficiency
2. **Word-boundary grouping**: Group character-by-character edits into word-level undo entries
3. **UI buttons**: Add undo/redo buttons to toolbar (planned for Ribbon UI - Task 41)
4. **Persistent undo**: Save undo history to disk for session restoration
