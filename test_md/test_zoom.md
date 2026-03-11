# Zoom Feature Test

Test Ctrl+Scroll Wheel zoom and keyboard zoom (v0.2.7).

## Zoom Controls

| Action | Shortcut | Expected |
|--------|----------|----------|
| Zoom In | Ctrl + Mouse Wheel Up | UI and text scale up |
| Zoom Out | Ctrl + Mouse Wheel Down | UI and text scale down |
| Zoom In | Ctrl + = (plus) | Same as scroll up |
| Zoom Out | Ctrl + - (minus) | Same as scroll down |
| Reset Zoom | Ctrl + 0 | Return to 100% zoom |

## Test Checklist

- [ ] Ctrl+Scroll Up increases zoom level
- [ ] Ctrl+Scroll Down decreases zoom level
- [ ] Ctrl++ (keyboard) increases zoom
- [ ] Ctrl+- (keyboard) decreases zoom
- [ ] Ctrl+0 resets to default zoom
- [ ] Zoom affects both editor text and rendered preview
- [ ] Zoom persists when switching between tabs
- [ ] Scroll without Ctrl scrolls normally (no zoom)
- [ ] Zoom level is visually smooth (no jarring jumps)

## Visual Verification Content

Small text to check at various zoom levels. Zoom in and verify readability improves, zoom out and verify more content is visible.

### Mixed Content

Here is **bold text**, *italic text*, and `inline code` that should all scale proportionally.

```rust
fn main() {
    println!("Code blocks should zoom too");
    let x = 42;
}
```

> Blockquotes should scale with the rest of the content.

| Col A | Col B | Col C |
|-------|-------|-------|
| 1 | 2 | 3 |
| 4 | 5 | 6 |

### Image (if supported)
Tables, code blocks, and all other rendered elements should zoom uniformly.
