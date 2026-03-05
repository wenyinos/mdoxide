# Header Spacing Setting

Adjustable vertical spacing between markdown headers (H1-H6) in the rendered view.

## Overview

Users can customize how much space appears before and after headings in Rendered and Split view modes. This affects visual density and readability of structured documents.

## Setting Options

| Value | Top Margin | Bottom Margin | Use Case |
|-------|------------|---------------|----------|
| **Compact** | 0.5× base | 0 | Dense outlines, long documents |
| **Normal** | 1.0× base | 0 | Default, balanced layout |
| **Relaxed** | 1.5× base | 2–6px | Spacious, print-like layout |

Base values for Normal: H1 (8px top), H2 (6px top), H3–H6 (4px top). Bottom margin is only applied in Relaxed mode (H1: 6px, H2: 4px, others: 2px).

## Location

- **Settings → Editor** (Markdown Rendering section, after Paragraph Indentation)
- **Config key**: `header_spacing` (serialized as `"compact"`, `"normal"`, or `"relaxed"`)

## Implementation

- **Config**: `HeaderSpacing` enum in `src/config/settings.rs`, `margins_for_level(level: u8)` helper
- **Editor**: `MarkdownEditor::header_spacing()` builder, `header_margins()` in `src/markdown/editor.rs`
- **Rendering**: `render_heading()` and `render_heading_with_structural_keys()` apply top/bottom margins via `ui.add_space()`
- **Raw view**: Unaffected (spacing applies only to rendered markdown)

## Persistence

Setting is stored in `config.json` via serde with `#[serde(default)]` for backward compatibility. Default is `normal`.
