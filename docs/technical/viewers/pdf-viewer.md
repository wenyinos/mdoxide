# PDF Viewer

Read-only PDF viewer tabs using the [hayro](https://crates.io/crates/hayro) crate for pure-Rust, CPU-based rendering.

## Architecture

Follows the same viewer infrastructure pattern as the Image Viewer (Task 39):

| Component | Location | Purpose |
|-----------|----------|---------|
| `FileType::Pdf` | `src/state.rs` | Extension detection (`.pdf`, case-insensitive) |
| `PdfViewerState` | `src/state.rs` | Per-tab state: current page, zoom, page count, error |
| `TabKind::PdfViewer` | `src/state.rs` | Tab kind variant carrying `PdfViewerState` |
| `open_pdf_tab()` | `src/state.rs` | Opens PDF, reads page count, creates tab |
| `render_pdf_viewer_tab()` | `src/app/central_panel.rs` | Page rendering, navigation, zoom UI |
| `PdfPageTexture` | `src/app/central_panel.rs` | Cached rendered page texture |
| `render_pdf_page()` | `src/app/central_panel.rs` | Renders a single page via hayro to egui texture |

## File Opening Flow

1. `open_file_with_focus()` checks `FileType::from_path()` — PDF intercepted before binary detection
2. `open_pdf_tab()` reads the file, creates `hayro_syntax::Pdf`, extracts page count
3. If PDF parsing fails (corrupted/encrypted), stores error in `PdfViewerState::error`
4. Creates `TabKind::PdfViewer(PdfViewerState)` tab

## Rendering Pipeline

```
PDF bytes → hayro_syntax::Pdf → page[i] → hayro::render(page, settings) → Pixmap
→ RGBA u8 slice → egui::ColorImage → egui::TextureHandle → egui::Image
```

- Pixmap data is premultiplied RGBA8, loaded via `from_rgba_premultiplied`
- Texture cached in egui temp data, keyed by `(path, page_index, zoom)`
- Cache automatically invalidated when page or zoom changes

## Page Navigation

| Action | Trigger |
|--------|---------|
| Previous page | `◀` button, `ArrowLeft`, `PageUp` |
| Next page | `▶` button, `ArrowRight`, `PageDown` |
| Zoom | `Ctrl+Scroll` (0.5x–4.0x range) |

## Metadata Bar

Bottom bar shows: `◀ 1 / N ▶ | PDF | 1.2 MB | 100%`

## Error Handling

- Corrupted PDF → error message overlay (no crash)
- Encrypted/password-protected → graceful error via `LoadPdfError`
- Page out of range → descriptive error
- File read failure → error overlay

## Performance

- Page rendered once per (page, zoom) combination; texture cached in egui temp storage
- Fit-to-window on first render (no upscale beyond 1:1)
- Re-render only on page change or zoom change

## Limitations

- hayro does not support encrypted/password-protected PDFs
- No text selection or search within PDF content
- Rendering is synchronous (blocking); large/complex pages may cause brief UI stalls
