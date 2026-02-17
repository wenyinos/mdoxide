# Image Rendering in Rendered View

## Overview

Markdown image syntax `![alt](path "title")` renders actual images in the Rendered and Split views. Images are loaded from disk, decoded, cached as egui textures, and displayed scaled to fit the available width while maintaining aspect ratio. Missing or unsupported images show a styled placeholder with alt text.

## Key Files

| File | Purpose |
|------|---------|
| `src/markdown/editor.rs` | `render_image()`, `resolve_image_path()`, `load_image_texture()`, `render_image_placeholder()` |
| `src/markdown/parser.rs` | `MarkdownNodeType::Image { url, title }` — comrak AST node |
| `Cargo.toml` | `image` crate with `png`, `jpeg`, `gif`, `webp` features |

## Implementation Details

### Path Resolution (`resolve_image_path`)

Resolution order for image URLs:

1. **Web URLs** (`http://`, `https://`, `data:`) — skipped (not supported), shows placeholder
2. **`file://` protocol** — stripped, treated as local path
3. **Absolute paths** — used directly if the file exists
4. **Relative to document directory** — resolved via `WikilinkContext.current_dir` (from `Tab.path`)
5. **Relative to workspace root** — fallback via `WikilinkContext.workspace_root`

The same `WikilinkContext` stored in egui memory for wikilink resolution is reused for image path resolution.

### Image Loading (`load_image_texture`)

1. Read raw bytes from disk with `std::fs::read()`
2. Decode with `image::load_from_memory()` (supports PNG, JPEG, GIF, WebP)
3. Convert to RGBA8 pixel buffer
4. Create `egui::ColorImage` from pixels
5. Load as named texture via `ctx.load_texture()` with linear filtering

### Caching

Textures are cached in egui's temp data keyed by resolved file path:

```rust
let cache_id = egui::Id::new("md_image_cache").with(&resolved_path);
```

On each frame, `render_image()` checks the cache first. On cache miss, it loads from disk and stores the result (success or failure). This avoids re-reading and re-decoding every frame.

Cache entries are `ImageLoadResult::Loaded(CachedImageTexture)` or `ImageLoadResult::Failed(String)`.

### Rendering

Images scale to fit available width while maintaining aspect ratio:

```rust
if orig_w > available_width {
    let scale = available_width / orig_w;
    (available_width, orig_h * scale)
} else {
    (orig_w, orig_h)
}
```

Hovering shows a tooltip with alt text, title, and file path.

### Inline Formatting Detection

Image nodes appear as children of `Paragraph` nodes in comrak's AST. The paragraph rendering code checks for "complex inline elements" to choose between a simple `TextEdit` path and a rich `horizontal_wrapped` path. `MarkdownNodeType::Image { .. }` is included in these checks so paragraphs containing images route through the formatting path where `render_inline_node` handles them.

This check exists in **4 locations** across the rendering pipeline:
- `render_paragraph_with_structural_keys` — `has_inline_elements`
- `render_paragraph` (legacy) — `has_inline_elements`
- List item rendering (structural keys) — `has_inline_formatting`
- List item rendering (legacy) — `has_inline_formatting`

### Error Handling

| Scenario | Behavior |
|----------|----------|
| Missing file | Styled placeholder frame with alt text and "Image not found" |
| Web URL | Placeholder with "Web images not supported" |
| Empty URL | Placeholder with "No image path" |
| Decode failure | Placeholder with error message, `log::warn!` |
| Unsaved file (no path) | Relative paths cannot resolve; absolute paths still work |

The placeholder shows a framed box with a picture icon, italicized alt text, and a hint message.

## Supported Formats

- PNG
- JPEG / JPG
- GIF (first frame only)
- WebP

## Dependencies

- `image` crate v0.25 with features: `png`, `jpeg`, `gif`, `webp`
- egui's `ColorImage`, `TextureHandle`, `TextureOptions` for texture management

## Usage

```markdown
![Photo](./images/photo.jpg)
![Logo](assets/logo.png "Company Logo")
![](screenshot.webp)
```

Images display in Rendered and Split views. In Raw view, the markdown syntax is shown as-is.

**Note:** The file must be saved to disk for relative image paths to resolve. Untitled (unsaved) documents can only display images with absolute paths.
