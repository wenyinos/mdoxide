# Setext Heading Detection & Single-Dash Fix

## Overview

Improved the robustness of `fix_false_setext_headings()` in the markdown parser. The function corrects a single `-` under text from being treated as a setext H2 heading (comrak's default per CommonMark) to a Paragraph + List(Item), since in an editor context a lone `-` is almost always the start of a list item.

During investigation, confirmed that `--` and `---` under text are **legitimate setext H2 headings per the CommonMark specification** and should not be overridden.

## Key Files

- `src/markdown/parser.rs` — `fix_false_setext_headings()` function and test suite

## CommonMark Setext Heading Rules

| Source | Result | Standard? |
|--------|--------|-----------|
| `Text\n=` or `Text\n===` | H1 heading | Yes (CommonMark) |
| `Text\n-` | Paragraph + List (editor override) | No — editor UX decision |
| `Text\n--` | H2 heading | Yes (CommonMark) |
| `Text\n---` | H2 heading | Yes (CommonMark) |
| `(blank)\n---\n(blank)` | Thematic break (horizontal rule) | Yes (CommonMark) |
| `---` at document start/end | YAML frontmatter delimiter | Yes (with frontmatter extension) |

The spec requires only **one or more** `-` or `=` characters for a setext underline. The distinction between `---` as a heading underline vs. thematic break depends on whether there is text directly above it (heading) or blank lines around it (thematic break).

## Implementation Details

### Problem: comrak `end_line` overshoot

Comrak's AST node for a setext heading may set `end_line` to include trailing blank lines or even the start of the next block element. The original code assumed the underline was always at `source_lines[end_line - 1]`, which pointed at the wrong line in multi-paragraph documents.

Example: for `"Text\n-\n\nMore text"`, comrak reports the heading as `start_line=1, end_line=3`, but the underline (`-`) is on line 2, not line 3.

### Fix: backwards scan for underline

Instead of blindly using `end_line`, the code now scans backwards through the heading's source line range to find the last line consisting entirely of `-` characters — that's the actual setext underline:

```rust
let mut underline_info: Option<(&str, usize)> = None;
for idx in (start_idx..end_idx).rev() {
    if let Some(line) = source_lines.get(idx) {
        let t = line.trim();
        if !t.is_empty() && t.chars().all(|c| c == '-') {
            underline_info = Some((t, idx + 1)); // 1-indexed
            break;
        }
    }
}
```

Only a single `-` triggers the false-setext correction; `--` and longer are left as valid headings.

## Tests

| Test | Validates |
|------|-----------|
| `test_setext_h2_double_dash_is_valid` | `Text\n--` → H2 (CommonMark compliant) |
| `test_setext_h2_triple_dash_is_valid` | `Text\n---` → H2 (CommonMark compliant) |
| `test_triple_dash_horizontal_rule_with_blank_lines` | `\n---\n` → ThematicBreak |
| `test_yaml_frontmatter_preserved` | YAML `---` delimiters parse correctly |
| `test_single_dash_false_setext_still_works` | `Text\n-` → Paragraph + List |
| `test_single_dash_false_setext_in_longer_doc` | Single-dash fix works when comrak's end_line overshoots |
| `test_multiline_setext_h2` | Multi-line text + `--` → valid H2 |
