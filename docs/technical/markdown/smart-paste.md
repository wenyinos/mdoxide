# Smart Paste for Links and Images

## Overview

Task 5 implements intelligent paste behavior that automatically creates markdown links and images when pasting URLs.

## Features

### 1. Link Creation with Selection
When text is selected and a URL is pasted:
- Select "Click here", paste `https://example.com`
- Result: `[Click here](https://example.com)`
- Cursor lands after the closing parenthesis

### 2. Image Insertion without Selection
When an image URL is pasted with no selection:
- Paste `https://example.com/pic.png`
- Result: `![](https://example.com/pic.png)`
- Cursor lands after the inserted image markdown

### 3. Regular URL Paste
When a regular URL (non-image) is pasted with no selection:
- Normal paste behavior (URL inserted as-is)

### 4. Non-URL Paste
When non-URL text is pasted:
- Normal paste behavior

## Implementation

### Key Files
- `src/app.rs` - Core smart paste logic

### Helper Functions

```rust
/// Check if a string looks like a URL
fn is_url(s: &str) -> bool

/// Check if a URL points to an image based on file extension
fn is_image_url(s: &str) -> bool
```

### URL Detection
URLs are detected by checking for:
- `http://` or `https://` prefix
- General scheme pattern: `[a-zA-Z][a-zA-Z0-9+.-]*://`

### Image URL Detection
Image URLs are detected by checking file extensions (case-insensitive):
- `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.svg`, `.bmp`, `.ico`, `.tiff`, `.tif`
- Query strings and fragments are stripped before extension check

### Architecture

The implementation uses a pre-render event consumption pattern:

**Pre-render Phase** (`consume_smart_paste`):
1. Scan egui input events for `Event::Paste(text)`
2. Check if pasted text is a URL
3. If URL + selection → create markdown link, consume event
4. If image URL + no selection → create markdown image, consume event
5. Otherwise → let normal paste proceed

This approach:
- Intercepts paste events before TextEdit processes them
- Only consumes events when smart behavior applies
- Falls through to default paste for normal cases

### Integration Points
The `consume_smart_paste` function is called in `update()`:
```rust
// IMPORTANT: Handle smart paste BEFORE rendering to intercept paste events
// and transform them into markdown links/images when appropriate.
self.consume_smart_paste(ctx);
```

## Testing

| Scenario | Expected Behavior |
|----------|-------------------|
| Select "Click here", paste URL | `[Click here](https://example.com)` |
| No selection, paste image URL | `![](https://example.com/pic.png)` |
| No selection, paste regular URL | Plain URL inserted |
| Paste non-URL text | Normal paste behavior |
| Undo after smart paste | Restores to before |
| URLs with query strings | Detection still works |
| URLs with fragments | Detection still works |

### Supported Image Extensions
- PNG: `.png`
- JPEG: `.jpg`, `.jpeg`
- GIF: `.gif`
- WebP: `.webp`
- SVG: `.svg`
- BMP: `.bmp`
- ICO: `.ico`
- TIFF: `.tiff`, `.tif`

### Edge Cases
- URLs with query strings: `https://example.com/pic.png?v=1` → detects as image
- URLs with fragments: `https://example.com/pic.png#section` → detects as image
- Mixed case extensions: `https://example.com/pic.PNG` → detects as image
- Non-standard schemes: `ftp://`, `file://` → detected as URLs

## Undo Behavior
- Smart link creation: Undo removes the entire link and restores selection
- Smart image insertion: Undo removes the image markdown
- Both operations are recorded as single edits for clean undo

## Future Enhancements
- Auto-fetch image titles for alt text
- Smart paste for other URL types (YouTube → embedded video placeholder)
- Configurable behavior per URL pattern
- Clipboard image detection (paste image data as file)
