# Table of Contents Generation

This document describes the Table of Contents (TOC) generation feature in Ferrite.

## Overview

Ferrite can automatically generate a navigable Table of Contents from markdown headings. The TOC is inserted as markdown links that correspond to the document's heading structure.

## Usage

### Keyboard Shortcut

- **Ctrl+Shift+U** (Windows/Linux) or **Cmd+Shift+U** (macOS): Insert or update Table of Contents

### Ribbon Button

Click the **☰** (hamburger menu) icon in the Format section of the ribbon when editing a markdown file.

## TOC Block Format

The TOC is wrapped in HTML comment markers for easy identification and updating:

```markdown
<!-- TOC -->
- [Introduction](#introduction)
  - [Getting Started](#getting-started)
- [Features](#features)
  - [Drag & Drop](#drag--drop)
<!-- /TOC -->
```

## Behavior

### Insert vs Update

- **Existing TOC**: If a `<!-- TOC -->...<!-- /TOC -->` block exists in the document, it will be replaced with the newly generated TOC.
- **No existing TOC**: A new TOC block will be inserted at the current cursor position.

### Heading Levels

By default, the TOC includes headings H1 through H3. The indentation reflects the heading hierarchy:

- H1 headings appear at the root level
- H2 headings are indented one level
- H3 headings are indented two levels

### Anchor Generation

Anchors are generated using the `slug` crate, which creates URL-safe identifiers:

| Heading | Generated Anchor |
|---------|-----------------|
| `Hello World` | `#hello-world` |
| `Getting Started` | `#getting-started` |
| `Drag & Drop` | `#drag-drop` |
| `What's New?` | `#what-s-new` |

### Inline Formatting

Inline markdown formatting (bold, italic, code, links) is stripped from heading text in the TOC for cleaner display:

| Heading | TOC Entry |
|---------|-----------|
| `# **Bold** Title` | `- [Bold Title](#bold-title)` |
| `## \`Code\` Section` | `- [Code Section](#code-section)` |
| `### [Linked](url) Text` | `- [Linked Text](#linked-text)` |

## Implementation Details

### Module Structure

The TOC functionality is implemented in `src/markdown/toc.rs` and consists of:

- `TocOptions` - Configuration for TOC generation (heading levels, bullet style)
- `TocHeading` - Represents a single heading for TOC inclusion
- `TocResult` - Result of TOC insertion/update operation
- `extract_toc_headings()` - Extracts headings from markdown text
- `generate_toc_content()` - Generates the TOC markdown content
- `generate_toc_block()` - Generates complete TOC block with markers
- `find_toc_block()` - Finds existing TOC block in document
- `insert_or_update_toc()` - Main entry point for TOC operations
- `remove_toc()` - Removes TOC block from document

### Reusing Outline Extraction

The TOC generation reuses the existing heading extraction logic from `src/editor/outline.rs`, which provides:

- Accurate heading level detection (H1-H6)
- Inline formatting stripping (bold, italic, code, links are cleaned)
- Proper handling of code blocks (headings inside code blocks are ignored)

## Configuration

### Default Options

| Option | Default | Description |
|--------|---------|-------------|
| `min_level` | 1 | Minimum heading level to include |
| `max_level` | 3 | Maximum heading level to include |
| `use_bullets` | true | Use `-` bullets (vs `1.` numbered) |
| `indent` | `"  "` | Two-space indentation per level |

### Future Enhancements

- Settings panel option for default max heading level
- Auto-update TOC on save (configurable)
- Custom TOC title/header support

## Edge Cases

### Empty Document

If the document is empty or contains no headings, an empty TOC block is inserted:

```markdown
<!-- TOC -->
<!-- /TOC -->
```

### Non-Markdown Files

TOC generation is only available for markdown files (`.md`, `.markdown`). Attempting to use it on other file types shows a toast message: "TOC only available for Markdown files".

### Headings in Code Blocks

Headings inside fenced code blocks are correctly ignored:

```markdown
# Real Heading

\`\`\`markdown
# This is NOT included in TOC
\`\`\`
```

Only "Real Heading" appears in the TOC.

## Related Features

- **Outline Panel** (`Ctrl+Shift+O`): Visual navigation through document headings
- **Semantic Minimap**: Shows heading structure in the minimap
