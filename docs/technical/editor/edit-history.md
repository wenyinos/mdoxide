# EditHistory Module

## Overview

The `EditHistory` module provides an operation-based undo/redo system for FerriteEditor. Unlike the snapshot-based system (which stored full content copies), this approach stores discrete edit operations, making it memory-efficient for large files.

**Status:** Fully integrated in FerriteEditor (v0.2.6). Ctrl+Z/Y work for undo/redo.

## Architecture

### Operation-Based vs Snapshot-Based

| Approach | Memory Usage | Complexity | Use Case |
|----------|--------------|------------|----------|
| **Snapshot** (current Tab) | O(n × history_size) | Simple | Small files (<100KB) |
| **Operation** (EditHistory) | O(ops × avg_op_size) | Moderate | Large files (>1MB) |

For a 4MB file with 100 edits averaging 20 characters each, the operation-based approach uses ~2KB vs ~400MB for snapshots.

### Key Components

```rust
// Edit operation - stores position and text
pub enum EditOperation {
    Insert { pos: usize, text: String },
    Delete { pos: usize, text: String },
}

// History manager - maintains undo/redo stacks
pub struct EditHistory {
    undo_stack: Vec<OperationGroup>,
    redo_stack: Vec<OperationGroup>,
    last_edit_time: Option<Instant>,
}
```

## Edit Operations

### Insert Operation

Records text insertion at a character position:

```rust
let op = EditOperation::Insert {
    pos: 5,
    text: " World".to_string(),
};
```

**Undo**: Delete `text.len()` characters starting at `pos`  
**Redo**: Insert `text` at `pos`

### Delete Operation

Records text deletion, storing the removed text for undo:

```rust
let op = EditOperation::Delete {
    pos: 5,
    text: " World".to_string(),
};
```

**Undo**: Insert `text` at `pos`  
**Redo**: Delete `text.len()` characters starting at `pos`

### Operation Inverse

Each operation type is the inverse of the other:

```rust
let insert = EditOperation::Insert { pos: 5, text: "X".to_string() };
let delete = insert.inverse();  // EditOperation::Delete { pos: 5, text: "X" }
```

## Operation Grouping

### Time-Based Grouping

Consecutive operations within 500ms are grouped into a single undo unit. This means rapid typing is undone as a single action rather than character-by-character.

```rust
const GROUP_THRESHOLD: Duration = Duration::from_millis(500);

// Rapid typing groups together
history.record_operation(EditOperation::Insert { pos: 0, text: "H".to_string() });
history.record_operation(EditOperation::Insert { pos: 1, text: "i".to_string() });
// Both operations in single undo group

// After 500ms pause, new group starts
std::thread::sleep(Duration::from_millis(550));
history.record_operation(EditOperation::Insert { pos: 2, text: "!".to_string() });
// New undo group
```

### Manual Group Breaking

Force a new group at specific points (e.g., after save):

```rust
history.break_group();
```

## Integration with TextBuffer

The EditHistory works with the `TextBuffer` module (also part of FerriteEditor):

```rust
use crate::editor::{TextBuffer, EditHistory, EditOperation};

let mut buffer = TextBuffer::from_string("Hello");
let mut history = EditHistory::new();

// Insert and record
let text = " World";
let pos = 5;
buffer.insert(pos, text);
history.record_operation(EditOperation::Insert {
    pos,
    text: text.to_string(),
});

// Undo (applies inverse automatically)
history.undo(&mut buffer);  // buffer is now "Hello"

// Redo (reapplies operation)
history.redo(&mut buffer);  // buffer is now "Hello World"
```

## API Reference

### EditHistory Methods

| Method | Description |
|--------|-------------|
| `new()` | Create empty history |
| `record_operation(op)` | Record operation, groups if <500ms |
| `undo(buffer)` → `bool` | Apply inverse of last group, returns success |
| `redo(buffer)` → `bool` | Reapply last undone group, returns success |
| `can_undo()` → `bool` | Check if undo available |
| `can_redo()` → `bool` | Check if redo available |
| `clear()` | Clear all history |
| `break_group()` | Force end of current group |
| `undo_count()` → `usize` | Number of undo groups |
| `redo_count()` → `usize` | Number of redo groups |

### EditOperation Methods

| Method | Description |
|--------|-------------|
| `inverse()` | Returns the reverse operation |
| `apply(buffer)` | Applies operation to buffer |

## Behavior

### Redo Stack Clearing

Recording a new operation clears the redo stack:

```
Initial: "Hello"
Insert " World" → undo: [Insert], redo: []
Undo            → undo: [], redo: [Insert]
Insert "!"      → undo: [Insert "!"], redo: [] (cleared!)
```

### Grouped Undo

When operations are grouped, a single undo reverses all operations in the group (in reverse order):

```
Type "Hi" quickly (grouped):
  undo: [Group([Insert "H", Insert "i"])]

Single undo:
  1. Remove "i" (inverse of last)
  2. Remove "H" (inverse of first)
  Result: empty string
```

### Unicode Support

Operations correctly handle Unicode text:

```rust
buffer.insert(0, "こんにちは");  // Japanese "Hello"
history.record_operation(EditOperation::Insert {
    pos: 0,
    text: "こんにちは".to_string(),
});

// Character count, not byte count
assert_eq!(buffer.len(), 5);  // 5 characters (15 bytes)
```

## Memory Efficiency

### Comparison for 4MB File

| Scenario | Snapshot System | Operation System |
|----------|-----------------|------------------|
| 100 character inserts | 100 × 4MB = 400MB | 100 × ~20B = ~2KB |
| 50 line deletions | 50 × 4MB = 200MB | 50 × ~80B = ~4KB |
| Mixed 100 operations | ~300MB | ~5KB |

### Why Operations Win

1. **Small edit operations**: Most edits are small (few characters)
2. **No full copies**: Only changed text is stored
3. **Grouping reduces entries**: Rapid edits share one group

## Integration

EditHistory is integrated into FerriteEditor (v0.2.6):

- **FerriteEditor widget** - Main integration point, history stored per-editor
- **Keyboard input** - All edits automatically record operations
- **Keyboard shortcuts** - Ctrl+Z/Y trigger undo/redo
- **Large file mode** - Reduced undo stack (10 vs 100 entries) for memory efficiency

## Related Documentation

- [TextBuffer](./text-buffer.md) - Rope-based text storage
- [Undo/Redo System](./undo-redo.md) - Current snapshot-based system
- [Custom Editor Widget Plan](../planning/custom-editor-widget-plan.md) - v0.3.0 roadmap

## Testing

The module includes comprehensive tests:

```bash
cargo test history
```

Tests cover:
- Basic undo/redo operations
- Operation grouping (time-based)
- Unicode and emoji handling
- Extensive operation sequences (100 ops)
- Large buffer performance (1MB, ignored by default)
