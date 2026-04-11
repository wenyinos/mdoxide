# Image Viewer Tab

Dedicated image viewer tabs for opening image files (PNG, JPEG, GIF, WebP, BMP) directly in Ferrite, intercepting them before the binary file error path.

## Architecture

### FileType Detection

`FileType::Image` variant in `src/state.rs` detects extensions: `png`, `jpg`, `jpeg`, `gif`, `webp`, `bmp` (case-insensitive via `from_extension()`).

### Tab Kind

`TabKind::ImageViewer(ImageViewerState)` holds per-tab viewer state:

| Field | Type | Purpose |
|-------|------|---------|
| `zoom` | `f32` | Current zoom level (default 1.0, auto-fitted on first render) |
| `dimensions` | `Option<(u32, u32)>` | Image width/height, populated after first load |
| `file_size` | `u64` | File size in bytes |
| `format_label` | `String` | Uppercase extension (e.g., "PNG", "JPEG") |
| `fitted` | `bool` | Whether initial fit-to-window has been applied |

### File Opening Flow

In `AppState::open_file_with_focus()`, image files are intercepted **before** the binary content check:

```
open_file_with_focus()
  → check already open (dedup by path)
  → FileType::from_path().is_image()? → open_image_tab()  ← early return
  → is_binary_content()? → error
  → normal text tab
```

`open_image_tab()` creates a lightweight tab without reading file content as text — only metadata (file size, extension) is captured. The image bytes are loaded lazily on first render.

### Texture Loading & Caching

Image textures are loaded via `load_viewer_image()` in `central_panel.rs`, reusing the same `image` crate infrastructure as markdown image rendering. Textures are cached in egui's temp data store keyed by file path (`egui::Id::new("image_viewer_texture").with(&path)`).

### Rendering

`render_image_viewer_tab()` in `central_panel.rs`:

1. **Fit-to-window**: On first render, computes scale to fit available area without upscaling beyond 1:1
2. **Ctrl+Scroll zoom**: Multiplicative zoom (0.1x to 10.0x range), applied via `smooth_scroll_delta.y`
3. **Centered display**: Image centered in a `ScrollArea::both()` with computed padding
4. **Metadata bar**: Bottom separator with dimensions, format, file size, and zoom percentage

### Tab Behavior

- Tab title shows framed picture emoji + filename
- `should_prompt_to_save()` returns `false` (read-only, no modifications)
- Image viewer tabs skip editor focus (`needs_focus = false`)
- Invalid images show an error message in the tab instead of crashing

## Supported Formats

PNG, JPEG, GIF, WebP, BMP — handled by the `image` crate with corresponding feature flags in `Cargo.toml`.

## Key Files

| File | Changes |
|------|---------|
| `src/state.rs` | `FileType::Image`, `ImageViewerState`, `TabKind::ImageViewer`, `open_image_tab()` |
| `src/app/central_panel.rs` | `render_image_viewer_tab()`, `load_viewer_image()`, `ImageViewerTexture` |
| `src/editor/folding.rs` | `FileType::Image` added to no-fold branch |
| `Cargo.toml` | Added `bmp` feature to `image` crate |
