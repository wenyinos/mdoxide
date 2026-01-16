# Quick Switcher Mouse Support

## Overview

Fixed mouse hover and click support in the quick switcher (Ctrl+P). The original implementation had unreliable hit detection due to incorrect interaction rect sizing, and the hover state was not synchronized with keyboard selection.

## Key Files

- `src/ui/quick_switcher.rs` - Quick file switcher implementation

## Problem

The quick switcher had several mouse interaction issues:

1. **Unreliable hover detection** - Using `ui.available_rect_before_wrap()` at the start of a horizontal layout gave an incorrect/small rect
2. **Click only worked between words** - Text labels were blocking clicks from reaching the interaction layer
3. **Hover-selection desync** - Mouse hover showed visual highlight but didn't update `selected_index`, so pressing Enter after hovering opened the wrong item
4. **Background covering text** - When background was painted after content, it obscured the text

## Solution

### 1. Content-First Layout

Draw content first with `ui.horizontal()` to get the actual row rect:

```rust
let row_response = ui
    .horizontal(|ui| {
        ui.add_space(16.0);
        ui.label(RichText::new(icon).size(14.0));
        ui.add_space(8.0);
        ui.label(RichText::new(&result.display_name).color(text_color).strong());
        // ... more content
    })
    .response;
```

### 2. Overlay Interaction Layer

Create a clickable interaction AFTER content so it captures all clicks (including on text):

```rust
let row_rect = row_response.rect.expand2(egui::vec2(8.0, 2.0));
let response = ui.interact(
    row_rect,
    ui.id().with(("row_click", idx)),
    Sense::click(),
);
```

### 3. Hover-Selection Sync

Update `selected_index` when hovering so Enter key opens the correct item:

```rust
if response.hovered() {
    self.selected_index = idx;
}
```

### 4. Background Layer Painting

Use `Order::Background` layer to paint highlight behind text:

```rust
let bg_layer = LayerId::new(
    Order::Background,
    ui.id().with(("row_bg", idx)),
);
ui.ctx().layer_painter(bg_layer).rect_filled(
    row_rect,
    4.0,
    if is_selected { selected_bg } else { hover_bg },
);
```

## Key egui Concepts

| Concept | Usage |
|---------|-------|
| `ui.horizontal()` | Layout content in a row, returns `InnerResponse` with `.response.rect` |
| `ui.interact()` | Create invisible clickable/hoverable region over existing content |
| `LayerId` + `Order::Background` | Paint to a layer behind other widgets |
| `layer_painter()` | Get a painter that draws to a specific layer |

## Behavior After Fix

- **Hover highlights correctly** - Full row highlights on mouse hover
- **Click opens files** - Clicking anywhere on a row (including on text) opens the file
- **Selection syncs with hover** - Moving mouse updates `selected_index` immediately
- **Enter works after hover** - Pressing Enter opens the hovered item
- **Keyboard still works** - Arrow keys navigate, transitions between mouse/keyboard are seamless

## Test Strategy

1. Open quick switcher (Ctrl+P)
2. Move mouse over items - verify highlight follows cursor
3. Click on file name text - verify file opens
4. Click between words - verify file opens
5. Hover over item, press Enter - verify hovered item opens (not previously keyboard-selected)
6. Use arrow keys after hovering - verify keyboard navigation still works
