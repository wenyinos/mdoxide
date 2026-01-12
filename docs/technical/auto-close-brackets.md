# Auto-close Brackets & Quotes

## Overview

Task 4 implements automatic insertion of closing brackets and quotes while typing, with selection wrapping and skip-over behavior.

## Features

### 1. Auto-pair Insertion
When typing an opener character without a selection, automatically insert the closing pair:
- `(` → `()`
- `[` → `[]`
- `{` → `{}`
- `"` → `""`
- `'` → `''`
- `` ` `` → ``` `` ```

Cursor is positioned between the pair.

### 2. Selection Wrapping
When text is selected and an opener is typed, wrap the selection:
- Select "hello", type `(` → `(hello)`
- Select "world", type `"` → `"world"`

Cursor moves to after the closing bracket.

### 3. Skip-over Behavior
When typing a closer and the next character is the same closer, move cursor forward instead of inserting a duplicate:
- `text|)` + type `)` → `text)|` (skipped over)

### 4. Smart Quote Handling
Quotes are not auto-closed after alphanumeric characters to avoid interference with contractions:
- `can't` won't trigger auto-close after the apostrophe

## Implementation

### Settings
- `Settings.auto_close_brackets: bool` - Default: `true`
- UI toggle in Settings → Editor → "Auto-close Brackets & Quotes"

### Key Files
- `src/config/settings.rs` - Setting definition
- `src/ui/settings.rs` - Settings UI checkbox
- `src/app.rs` - Core auto-close logic

### Architecture

The implementation uses a two-phase approach:

**Pre-render Phase** (`handle_auto_close_pre_render`):
1. Scan egui input events for bracket/quote characters
2. Handle skip-over: Consume closer events when next char matches
3. Handle selection wrapping: Consume opener events, modify content manually

**Post-render Phase** (`handle_auto_close_post_render`):
1. Detect if single opener character was just typed
2. Insert closing bracket at cursor position
3. Use `pending_cursor_restore` to keep cursor between brackets

### Helper Functions
```rust
fn get_closing_bracket(opener: char) -> Option<char>
fn is_closing_bracket(ch: char) -> bool
fn get_opening_bracket(closer: char) -> Option<char>
```

## Testing

| Scenario | Expected |
|----------|----------|
| Type `(` with no selection | `()` inserted, cursor between |
| Type `(` with "hello" selected | `(hello)` with cursor after `)` |
| Cursor before `)`, type `)` | Cursor jumps over `)` |
| Disable setting, type `(` | Only `(` inserted |
| Type `'` after "can" | Only `'` inserted (apostrophe) |
| Undo after auto-close | Restores to before |

## Undo Behavior

- Selection wrapping: Undo removes both brackets
- Auto-pair insertion: TextEdit's internal undo may handle this differently
- Skip-over: No undo needed (just cursor movement)

## Future Enhancements

- Multi-char pairs: `**`, `__` for markdown emphasis
- Triple backticks for code blocks
- Configurable pair mapping
