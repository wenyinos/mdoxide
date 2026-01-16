# Document Statistics Panel

The Document Statistics panel provides comprehensive metrics about markdown documents, displayed as a tab within the Outline panel.

## Overview

When viewing markdown documents, the Outline panel now includes two tabs:
- **Outline**: The existing heading navigation view
- **Stats**: Document statistics and metrics

## Statistics Displayed

### Reading Time
- Prominent display at the top
- Calculated at 250 words per minute (WPM) average reading speed
- Minimum reading time is 1 minute

### Text Statistics
- **Words**: Total word count
- **Characters**: Total character count (including spaces)
- **Chars (no spaces)**: Character count excluding whitespace
- **Lines**: Total line count
- **Paragraphs**: Paragraph count (text blocks separated by blank lines)

### Structure
- **Headings**: Total heading count with per-level breakdown (H1-H6)
- **List items**: Count of both ordered and unordered list items
- **Horizontal rules**: Count of `---`, `***`, or `___` separators

### Media & Links
- **Links**: Count of markdown links `[text](url)` (excludes images)
- **Images**: Count of markdown images `![alt](url)`

### Code & Diagrams
- **Code blocks**: Count of fenced code blocks (excluding Mermaid)
- **Mermaid diagrams**: Count of ````mermaid` blocks specifically
- **Tables**: Count of markdown tables
- **Blockquotes**: Count of `>` quote blocks

## Implementation Details

### Data Structures

The `DocumentStats` struct in `src/editor/stats.rs`:

```rust
pub struct DocumentStats {
    pub text: TextStats,              // words, chars, lines, paragraphs
    pub headings_by_level: [usize; 6], // H1-H6 counts
    pub heading_count: usize,
    pub link_count: usize,
    pub image_count: usize,
    pub code_block_count: usize,
    pub mermaid_count: usize,
    pub table_count: usize,
    pub blockquote_count: usize,
    pub reading_time_minutes: u32,
    pub list_item_count: usize,
    pub horizontal_rule_count: usize,
}
```

### Calculation

Statistics are calculated efficiently by:
1. Single-pass text parsing for basic text stats
2. Line-by-line markdown element detection
3. Code block content is excluded from heading/link/image counts

### Performance

- Statistics are cached alongside the document outline
- Only recalculated when document content changes (hash-based change detection)
- Parsing is efficient (< 50ms for typical documents)

## Tab Interface

The tab bar uses custom-painted buttons:
- Active tab has highlighted underline
- Non-active tabs use muted colors
- Both tabs available for markdown files
- Structured files (JSON/YAML/TOML) continue showing structure stats without tabs

## Localization

All UI strings are localized via `locales/en.yaml`:
- `outline.tab_outline` - "Outline"
- `outline.tab_statistics` - "Stats"
- `stats.*` - Individual statistic labels

## Related Files

| File | Purpose |
|------|---------|
| `src/editor/stats.rs` | `DocumentStats` struct and parsing |
| `src/ui/outline_panel.rs` | Tab UI and statistics rendering |
| `locales/en.yaml` | Localization strings |
| `src/app.rs` | Integration and caching |
