# Grapheme-Cluster-Aware Cursor Navigation

## Overview

Cursor movement (arrow keys), backspace, and delete now operate on **grapheme clusters** rather than individual Unicode code points. A grapheme cluster is the user-perceived "character" — it can span multiple code points (emoji ZWJ sequences, Bengali conjuncts, combining diacritics, Korean conjoining jamo).

**Dependency:** `unicode-segmentation` 1.11

## Problem

The editor's `Cursor.column` is a char (code point) offset. Before this change, every movement incremented/decremented by exactly 1 char. For multi-code-point grapheme clusters this meant:

- Arrow keys would land inside a cluster (e.g., between the ZWJ and the next emoji in 👨‍👩‍👧).
- Backspace would delete a single code point, leaving a broken cluster.
- Delete would partially remove a cluster.

## Solution

### Grapheme helper module (`src/editor/ferrite/grapheme.rs`)

Two core functions that map between char-column offsets and grapheme boundaries:

| Function | Purpose |
|----------|---------|
| `prev_grapheme_boundary(line, col)` | Returns the char-column of the grapheme boundary before `col`. Snaps backward if `col` is mid-cluster. |
| `next_grapheme_boundary(line, col)` | Returns the char-column just past the grapheme cluster at `col`. Snaps forward if `col` is mid-cluster. |
| `line_text_stripped(line)` | Strips `\n`/`\r\n` from `TextBuffer::get_line()` output before grapheme iteration. |

Both functions iterate grapheme clusters via `UnicodeSegmentation::graphemes(true)` (extended grapheme clusters, UAX #29) and track a running char-offset accumulator.

### Changed call sites

| File | Function | Change |
|------|----------|--------|
| `input/keyboard.rs` | `move_cursor_left` | `column -= 1` → `prev_grapheme_boundary()` |
| `input/keyboard.rs` | `move_cursor_right` | `column += 1` → `next_grapheme_boundary()` |
| `input/keyboard.rs` | `delete_backward` | `remove(pos-1, 1)` → `remove(pos-N, N)` where N = cluster size |
| `input/keyboard.rs` | `delete_forward` | `remove(pos, 1)` → `remove(pos, N)` |
| `editor.rs` | `backspace_at_all_cursors` | Computes per-cursor grapheme range instead of fixed 1-char |
| `editor.rs` | `delete_at_all_cursors` | Computes per-cursor grapheme range instead of fixed 1-char |
| `editor.rs` | `move_all_cursors` (ArrowLeft/Right) | Uses grapheme boundaries for multi-cursor navigation |
| `editor.rs` | `move_all_cursors_right` | Uses grapheme boundary for skip-over |

### Unchanged behaviour

- **Line joins** (backspace at column 0, delete at end of line): still remove exactly 1 char (the newline).
- **Word movement** (Ctrl+Arrow): operates on word boundaries, unchanged by this task.
- **Home/End/Page Up/Down**: position at line boundaries, unaffected.
- **Latin text**: each ASCII character is its own grapheme cluster, so movement is still by 1.

## Test coverage

### Unit tests (`grapheme.rs`, 16 tests)

Latin, emoji ZWJ, Bengali conjuncts, Korean precomposed and conjoining jamo, combining diacritics, edge cases (empty string, single char), line stripping.

### Integration tests (`keyboard.rs`, 12 new tests)

| Test | Script | Verifies |
|------|--------|----------|
| `test_arrow_right_emoji_zwj_family` | 👨‍👩‍👧 | Right arrow skips entire 5-char ZWJ sequence |
| `test_arrow_left_emoji_zwj_family` | 👨‍👩‍👧 | Left arrow jumps back over entire cluster |
| `test_backspace_deletes_entire_emoji_cluster` | a👨‍👩‍👧b | Backspace removes whole emoji, leaves "ab" |
| `test_delete_removes_entire_emoji_cluster` | a👨‍👩‍👧b | Delete removes whole emoji, leaves "ab" |
| `test_bengali_conjunct_single_arrow_press` | ক্ষ | Single right-arrow skips 3-char conjunct |
| `test_bengali_conjunct_backspace` | ক্ষ | Backspace removes entire conjunct |
| `test_korean_jamo_single_arrow_press` | 한 | Right arrow skips 3-char jamo block |
| `test_combining_diacritic_arrow_right` | é (decomposed) | Skips e + combining acute as one unit |
| `test_latin_unchanged_regression` | Hello | Latin still moves by 1 |
| `test_backspace_latin_unchanged_regression` | Hello | Latin backspace still deletes 1 char |
| `test_delete_at_end_of_line_joins` | Hello\nWorld | Delete at EOL still joins lines |
