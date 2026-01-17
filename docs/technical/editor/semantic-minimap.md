# Semantic Minimap

## Overview

The semantic minimap provides a clickable list of document structure for markdown files, replacing the pixel-based minimap. It shows headings (H1-H6) plus content type indicators for code blocks, Mermaid diagrams, tables, images, and blockquotes. For non-markdown files (code, CSV, JSON, etc.), the original pixel minimap is used instead.

This provides a meaningful document navigation experience for markdown documents, allowing users to quickly jump to any section or content block.

## Features

- **Header Labels**: Displays all H1-H6 headings from the document
- **Content Type Indicators**: Shows visual markers for code blocks, Mermaid diagrams, tables, images, and blockquotes
- **Density Visualization**: Horizontal bars between items showing relative content density (darker/wider = more content)
- **Visual Hierarchy**: Font size and indentation vary by heading level (H1 largest, deeper levels smaller/indented)
- **Current Section Highlighting**: The section containing the cursor is visually highlighted
- **Click Navigation**: Click any item to scroll the editor to that position
- **Hover Tooltips**: Shows line number and context-specific information on hover
- **Theme Integration**: Full support for light and dark themes with distinct colors for each content type
- **Scrollable List**: Handles documents with many items via scroll area
- **File Type Aware**: Automatically uses semantic minimap for `.md`/`.markdown` files, pixel minimap for others

## Implementation

### Core Components

**`SemanticMinimap`** (`src/editor/minimap.rs`)
- Main widget that renders the heading list
- Takes `&[OutlineItem]` from the existing outline extraction system
- Builder pattern for configuration (width, scroll offset, theme, etc.)

**`SemanticMinimapOutput`**
- Output struct containing navigation requests
- `scroll_to_line`: Target line number for navigation
- `scroll_to_char`: Character offset for precise positioning
- `clicked`: Whether a heading was clicked

### Visual Design

| Heading Level | Font Size | Indent |
|---------------|-----------|--------|
| H1 | 8.0 (bold) | 0px |
| H2 | 7.7 | 4px |
| H3 | 7.4 | 8px |
| H4 | 7.1 | 12px |
| H5 | 6.8 | 16px |
| H6 | 6.5 | 20px |

Content blocks use a fixed font size (7.0) with no indentation.

Item height: 11px per row for compact display.

### Color Scheme

Each heading level has a distinct color indicator:
- **H1**: Blue
- **H2**: Green
- **H3**: Orange
- **H4**: Purple
- **H5+**: Gray

Content type indicators have their own color scheme:
- **Code blocks** (`</>`): Purple - for fenced code blocks (```)
- **Mermaid diagrams** (`◇`): Teal - for ```mermaid blocks
- **Tables** (`⊞`): Gold - for markdown tables with | separators
- **Images** (`▣`): Pink - for ![alt](url) syntax
- **Blockquotes** (`❝`): Slate blue - for > quoted text

The current section is highlighted with a background color and white/bold text.

### Content Type Detection

The outline extraction system now detects the following content types:

| Type | Detection Pattern | Icon | Example Title |
|------|------------------|------|---------------|
| Heading | `# `, `## `, etc. | H1-H6 | Actual heading text |
| Code Block | Opening ` ``` ` | `</>` | "Code block (line N)" |
| Mermaid | ` ```mermaid ` | `◇` | "Mermaid diagram" |
| Table | Lines with `\|` separators | `⊞` | "Table (line N)" |
| Image | `![alt](url)` | `▣` | Alt text or filename |
| Blockquote | Lines starting with `>` | `❝` | "Blockquote (line N)" |

**Important**: Content inside code blocks is not extracted. This prevents false detection of headings, tables, or images within code examples.

### Header Truncation

Long headings are truncated at 20 characters with an ellipsis (…) to maintain readability in the compact view.

## Settings

| Setting | Default | Range | Description |
|---------|---------|-------|-------------|
| `minimap_enabled` | `true` | bool | Toggle minimap visibility |
| `minimap_width` | `120.0` | 80-200 | Width in pixels |
| `minimap_mode` | `Auto` | Auto/Semantic/Pixel | Minimap display mode |

### Minimap Mode

The `minimap_mode` setting controls which minimap variant is displayed:

| Mode | Description | Use Case |
|------|-------------|----------|
| **Auto** (default) | Semantic for `.md`/`.markdown`, pixel for all other files | Best of both worlds |
| **Semantic** | Always show structure-based minimap with headings | Prefer document navigation |
| **Pixel** | Always show code overview minimap | Prefer code visualization |

The mode can be changed in Settings → Editor → Minimap Mode. Changes take effect immediately without restart.

Settings are stored in `src/config/settings.rs` and can be toggled in the Settings panel.

## Integration Points

### Editor Integration

The semantic minimap is integrated in two view modes:

1. **Raw Mode** (`ViewMode::Raw`): Shows minimap on the right side of the editor
2. **Split Mode** (`ViewMode::Split`): Shows minimap between editor and preview panes

Both modes use the same `extract_outline_for_file()` function to extract headings.

### Outline Reuse

The semantic minimap leverages the `OutlineItem` structure from `src/editor/outline.rs`, which provides:
- `content_type: ContentType` - The type of content (Heading, CodeBlock, MermaidDiagram, Table, Image, Blockquote)
- `level: u8` - Heading level (1-6) for headings, 0 for content blocks
- `title: String` - Display text (heading text or content description)
- `line: usize` - Line number (1-indexed)
- `char_offset: usize` - Character offset (for precise navigation)

The `ContentType` enum provides helper methods:
- `is_heading()` - Returns true for heading types
- `heading_level()` - Returns `Option<u8>` for heading level
- `label()` - Returns display label (e.g., "H1", "</>", "◇")

### Navigation Implementation

When a heading is clicked, `navigate_to_heading()` performs:

1. **Text Search**: Searches for the exact markdown pattern (e.g., `## Section Title`)
2. **Offset Conversion**: Converts byte offsets from `String::find()` to character offsets for egui
3. **Transient Highlight**: Applies a temporary highlight to the heading line
4. **Scroll Positioning**: Centers the heading in the viewport (positioned at 1/3 from top)

The offset conversion is critical because Rust's `String::find()` returns byte offsets, but egui's text system uses character offsets. The `byte_to_char_offset()` helper handles this conversion for UTF-8 content.

## Density Visualization

Horizontal bars between items show the relative amount of content between sections. This gives users a visual sense of document structure at a glance.

### How It Works

1. **Line Count Calculation**: For each item, the line count to the next item is calculated
2. **Normalization**: Line counts are normalized to a [0.0, 1.0] range based on min/max density in the document
3. **Visual Mapping**: The normalized value maps to bar width and opacity

### Visual Parameters

| Parameter | Min | Max | Description |
|-----------|-----|-----|-------------|
| Bar Width | 2px | 80% of width | Wider = more content |
| Bar Opacity | 0.15 | 0.6 | Darker = more content |
| Bar Height | 3px | 3px | Fixed height for subtle appearance |

### Interpretation

- **Wide, dark bar**: Large section with lots of content
- **Narrow, light bar**: Small section with minimal content
- **All equal bars**: Evenly distributed content across sections
- **No bar above first item**: First item starts at document top

### Configuration

Density visualization is enabled by default. It can be toggled via the `show_density` builder method:

```rust
let minimap = SemanticMinimap::new(&headers)
    .show_density(true)  // Enable density bars (default)
    .total_lines(1000);  // Total lines for last section calculation
```

## Performance

- Header extraction runs on each render when minimap is enabled
- The `extract_outline()` function uses efficient line-by-line parsing
- For typical markdown documents (<10k lines), extraction completes in <5ms
- Density calculation adds negligible overhead (O(n) where n = number of items)
- Large documents benefit from the same caching as the outline panel

## File Type Detection

When `minimap_mode` is set to `Auto` (default), the minimap automatically selects the appropriate variant based on file type:

| File Type | Extension | Minimap Used (Auto Mode) |
|-----------|-----------|--------------------------|
| Markdown | `.md`, `.markdown` | Semantic Minimap |
| Unsaved files | (none) | Semantic Minimap |
| JSON | `.json` | Pixel Minimap |
| YAML | `.yaml`, `.yml` | Pixel Minimap |
| TOML | `.toml` | Pixel Minimap |
| CSV | `.csv` | Pixel Minimap |
| TSV | `.tsv` | Pixel Minimap |
| Unknown | `.rs`, `.js`, `.py`, etc. | Pixel Minimap |

When `minimap_mode` is set to `Semantic` or `Pixel`, the user's preference overrides the automatic file type detection.

The mode selection logic uses the `MinimapMode::use_semantic()` method:

```rust
// Get minimap mode from settings
let minimap_mode = self.state.settings.minimap_mode;

// Check if file is markdown (for auto mode)
let is_markdown_file = self.state.active_tab()
    .map(|tab| {
        match &tab.path {
            Some(path) => {
                path.extension()
                    .and_then(|e| e.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("md") 
                        || ext.eq_ignore_ascii_case("markdown"))
                    .unwrap_or(false)
            }
            None => true, // Unsaved files default to markdown
        }
    })
    .unwrap_or(true);

// Determine which minimap to use based on mode setting
let use_semantic_minimap = minimap_mode.use_semantic(is_markdown_file);
```

Both Regular and Split view modes support dual minimap rendering:
- **Regular Mode**: Mode check at data collection, conditional rendering
- **Split Mode**: Separate `semantic_minimap_data_split` and `pixel_minimap_data_split` variables

## Related Files

- `src/editor/minimap.rs` - Both minimap widgets (Semantic and Pixel)
- `src/editor/outline.rs` - Header extraction logic
- `src/editor/mod.rs` - Module exports
- `src/app.rs` - Integration with view modes, file type detection
- `src/config/settings.rs` - Settings definitions

## Future Improvements

- [x] **Task 17**: Add semantic minimap content type indicators (v0.2.5)
- [x] **Task 18**: Implement semantic minimap density visualization (v0.2.5)
- [x] **Task 19**: Add toggle between semantic and pixel minimap in settings (v0.2.5)
- [ ] Sync scroll position between minimap and editor scrollbar
- [ ] Keyboard navigation (up/down arrows in minimap)
- [ ] Collapse/expand sections in minimap (matching outline panel)
