# Bracket and Markdown Emphasis Matching

This document describes the bracket and emphasis pair matching feature in Ferrite's editor.

## Overview

When the cursor is adjacent to a supported delimiter, the editor highlights both the delimiter and its matching counterpart with a subtle theme-aware highlight. This helps users visually track matching pairs while editing code or markdown.

## Supported Delimiters

### Brackets
- **Parentheses**: `(` and `)`
- **Square brackets**: `[` and `]`
- **Curly braces**: `{` and `}`
- **Angle brackets**: `<` and `>`

### Markdown Emphasis
- **Bold asterisks**: `**`
- **Bold underscores**: `__`

## How It Works

1. **Cursor Position Detection**: The feature monitors the primary cursor position (in multi-cursor mode, only the primary cursor drives highlighting)

2. **Delimiter Detection**: When the cursor moves, the matcher checks if it's adjacent to (immediately before or after) a delimiter character or emphasis marker

3. **Stack-Based Matching**: For brackets, a stack-based algorithm scans in the appropriate direction to find the matching pair, correctly handling nested structures

4. **Emphasis Matching**: For `**` and `__` markers, the matcher finds the next/previous occurrence of the same marker that forms a valid pair

5. **Visual Highlighting**: Both the source delimiter (at cursor) and target delimiter (matching pair) are highlighted with:
   - A subtle background fill
   - A thin border for better visibility

## Theme Integration

The highlight colors are theme-aware:

| Theme | Background | Border |
|-------|------------|--------|
| Light | Gold/yellow tint (rgba 255, 220, 100, 80) | rgb(200, 170, 50) |
| Dark | Cyan/blue tint (rgba 80, 180, 220, 60) | rgb(100, 180, 220) |

Colors are defined in `src/theme/mod.rs` under `UiColors`:
- `matching_bracket_bg`: Background fill color
- `matching_bracket_border`: Border color

## Settings

The feature can be toggled in Settings → Editor:

- **Setting name**: `highlight_matching_pairs`
- **Default**: `true` (enabled)
- **UI**: "Highlight Matching Brackets" checkbox

## Files

| File | Purpose |
|------|---------|
| `src/editor/matching.rs` | Core matching algorithm and delimiter types |
| `src/editor/widget.rs` | Integration with EditorWidget, rendering |
| `src/config/settings.rs` | Settings for the feature |
| `src/theme/mod.rs` | Theme colors for highlights |
| `src/ui/settings.rs` | Settings panel UI toggle |
| `src/app.rs` | Passing setting to EditorWidget |

## Architecture

### DelimiterKind Enum
Defines the types of delimiters:
```rust
pub enum DelimiterKind {
    Paren,                    // ()
    Bracket,                  // []
    Brace,                    // {}
    Angle,                    // <>
    EmphasisBoldAsterisk,     // **
    EmphasisBoldUnderscore,   // __
}
```

### DelimiterToken Struct
Represents a found delimiter:
```rust
pub struct DelimiterToken {
    pub kind: DelimiterKind,
    pub is_open: bool,       // Opening vs closing
    pub start: usize,        // Byte position start
    pub end: usize,          // Byte position end
}
```

### MatchingPair Struct
Result of a successful match:
```rust
pub struct MatchingPair {
    pub source: DelimiterToken,  // At cursor
    pub target: DelimiterToken,  // Matching pair
}
```

### DelimiterMatcher
Main service for finding matches:
```rust
let matcher = DelimiterMatcher::new(text);
if let Some(pair) = matcher.find_match(cursor_pos) {
    // Highlight pair.source and pair.target
}
```

## Performance

- Matching is computed on cursor movement, not every frame
- Stack-based algorithm is O(k) where k is the distance to the matching bracket
- For very large files, the algorithm scans only until a match is found
- Emphasis matching uses string `find()` which is optimized

## Edge Cases

1. **Unmatched brackets**: No highlight shown
2. **Nested brackets**: Innermost matching pair is highlighted
3. **Mixed bracket types**: Each type is tracked independently
4. **Multi-cursor**: Only primary cursor drives highlighting
5. **Unicode text**: Byte-to-char position conversion handles UTF-8

## Testing

The matching module includes comprehensive unit tests covering:
- Simple bracket matching
- Nested brackets
- Closing bracket matching
- Square brackets
- Angle brackets
- Emphasis markers (`**` and `__`)
- Unmatched delimiters
- Empty text
- Code blocks
- Unicode handling

Run tests with:
```bash
cargo test editor::matching
```

## Future Enhancements

Potential improvements for future versions:
- Syntax-aware matching (skip brackets in comments/strings)
- Configurable highlight style (background vs border-only)
- Single emphasis markers (`*` and `_` for italics)
- Rainbow brackets (nested depth coloring)
- Mismatched bracket error highlighting
