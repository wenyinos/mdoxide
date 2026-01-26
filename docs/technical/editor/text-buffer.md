# TextBuffer Module

A memory-efficient text buffer backed by the `ropey` rope data structure, designed for handling large files with O(log n) editing operations.

## Overview

`TextBuffer` wraps `ropey::Rope` to provide:
- O(log n) insert and delete operations
- O(log n) line/character index conversions
- O(1) length and line count queries
- Memory-efficient storage (target: <50MB for 4MB file)
- Unicode-correct text handling

## Module Location

```
src/editor/ferrite/buffer.rs
```

## API Reference

### Construction

```rust
use crate::editor::TextBuffer;

// Create empty buffer
let buffer = TextBuffer::new();

// Create from string
let buffer = TextBuffer::from_string("Hello\nWorld");

// From traits
let buffer: TextBuffer = "Hello".into();
let buffer: TextBuffer = String::from("World").into();
```

### Core Operations

#### Insert

```rust
// Insert text at character position
buffer.insert(5, " Beautiful");
// "Hello World" → "Hello Beautiful World"
```

- **Complexity**: O(M + log N) where N = buffer length, M = inserted text length
- **Panics**: If position is out of bounds

#### Remove

```rust
// Remove 6 characters starting at position 5
buffer.remove(5, 6);
// "Hello World" → "HelloWorld"
```

- **Complexity**: O(M + log N) where N = buffer length, M = removed text length
- **Panics**: If range is out of bounds

### Line Operations

#### Get Line Content

```rust
let line: Cow<str> = buffer.line(0);
let line_trimmed = buffer.line(0).trim_end();

// Non-panicking version
if let Some(line) = buffer.get_line(100) {
    println!("Line 100: {}", line);
}
```

- Returns `Cow<str>` - borrowed if contiguous in memory, owned otherwise
- Line content includes trailing newline if present
- **Complexity**: O(log N)

#### Line Count

```rust
let count = buffer.line_count();
// "Hello\nWorld" → 2 lines
// "Hello\nWorld\n" → 3 lines (trailing newline adds empty line)
```

- **Complexity**: O(1)

### Index Conversions

#### Line to Character Offset

```rust
let char_offset = buffer.line_to_char(1);
// For "Hello\nWorld": line 1 starts at char 6

// Non-panicking version
let offset = buffer.try_line_to_char(line_idx);
```

- **Complexity**: O(log N)

#### Character to Line Index

```rust
let line_idx = buffer.char_to_line(6);
// For "Hello\nWorld": char 6 ('W') is on line 1

// Non-panicking version
let line = buffer.try_char_to_line(char_idx);
```

- **Complexity**: O(log N)

### Length Queries

```rust
let char_count = buffer.len();      // Character count (O(1))
let byte_count = buffer.len_bytes(); // Byte count (O(1))
let is_empty = buffer.is_empty();   // Check if empty (O(1))
```

### String Conversion

```rust
// Via Display trait (ToString auto-implemented)
let content = buffer.to_string();

// Or use format!
let content = format!("{}", buffer);
```

### Advanced: Direct Rope Access

```rust
// Read-only access
let rope: &Rope = buffer.rope();

// Mutable access (use with caution - bypasses tracking)
let rope_mut: &mut Rope = buffer.rope_mut();
```

## Unicode Handling

`TextBuffer` correctly handles Unicode text:

```rust
let buffer = TextBuffer::from_string("Hello 世界 🌍");

// Character count (not byte count)
assert_eq!(buffer.len(), 10);      // 10 Unicode characters
assert_eq!(buffer.len_bytes(), 16); // 16 UTF-8 bytes

// Japanese text
let buffer = TextBuffer::from_string("こんにちは\n世界");
assert_eq!(buffer.line_count(), 2);
assert_eq!(buffer.line(0).trim_end(), "こんにちは");
```

## Performance Characteristics

| Operation | Complexity |
|-----------|------------|
| `new()`, `from_string()` | O(N) |
| `insert()` | O(M + log N) |
| `remove()` | O(M + log N) |
| `line()` | O(log N) |
| `line_count()` | O(1) |
| `line_to_char()` | O(log N) |
| `char_to_line()` | O(log N) |
| `len()`, `len_bytes()` | O(1) |
| `to_string()` | O(N) |

Where N = buffer length, M = operation length.

## Memory Efficiency

The rope data structure provides memory-efficient storage:
- **Target**: <50MB for 4MB file
- Compared to egui's TextEdit which can use ~500MB for a 4MB file

This is achieved through:
- Tree-based storage (B-tree of text chunks)
- No need for contiguous memory allocation
- Efficient structural sharing for undo/redo (planned)

## Usage in FerriteEditor

`TextBuffer` is the foundation of the custom FerriteEditor widget:

```
TextBuffer (this module)
    ↓
EditHistory (Task 3) - undo/redo using buffer operations
    ↓
ViewState (Task 4) - virtual scrolling with line indexing
    ↓
FerriteEditor (Task 6) - custom editor widget
```

## Testing

Run tests with:

```bash
cargo test buffer

# Include large file performance test
cargo test test_large_file -- --ignored
```

## Related Documentation

- [Editor Widget](./editor-widget.md) - Current egui TextEdit wrapper
- [Custom Editor Plan](../planning/custom-editor-widget-plan.md) - FerriteEditor architecture
- [Memory Optimization Plan](../planning/memory-optimization.md) - Memory reduction strategies

## Dependencies

- `ropey` 1.6 - Rope data structure
