# Search Highlight Positioning in Rendered View & Z-Order Stacking

## Overview

Search highlights now appear in the **rendered** markdown view (including table cells), and all floating overlay panels use consistent z-order stacking via `egui::Order::Foreground`.

## Part A: Rendered View Search Highlights

### Problem

Search highlights (from Ctrl+F) only appeared in the **raw** editor (FerriteEditor). The **rendered** WYSIWYG view and **split preview** pane showed no search match highlighting, making it difficult to locate matches when viewing formatted content — especially in tables where cell geometry differs from raw source lines.

### Solution

`MarkdownEditor` now accepts search match byte ranges and paints semi-transparent highlight overlays during the viewport rendering pass.

**Key changes:**

| File | Change |
|------|--------|
| `src/markdown/editor.rs` | Added `search_highlights` / `current_search_match` fields to `MarkdownEditor`, builder method `.search_highlights(matches, current)`, and `paint_rendered_search_highlights()` function |
| `src/app/central_panel.rs` | Wire `FindState` matches into `MarkdownEditor` for both full rendered view and split preview pane |

### How it works

1. `central_panel.rs` passes `FindState.matches` (byte position pairs) and `current_match` index to `MarkdownEditor` via the builder.
2. Inside `show_rendered_editor`, after each visible block renders, `paint_rendered_search_highlights()` checks if any match byte ranges overlap the block's source line range.
3. For **table blocks**: highlights are subdivided into per-row strips. The separator line (e.g., `|---|---|`) is skipped. Each data row gets its own highlight rect.
4. For **non-table blocks** (paragraphs, headings, lists): a proportional strip is painted at the approximate Y position within the block.
5. The current match uses a brighter color (yellow/orange); other matches use a dimmer semi-transparent yellow.

### Coordinate mapping

- `line_start_byte_offsets()` (already in codebase) provides a byte offset → line number lookup via binary search.
- Each rendered block tracks `y_before` / `y_after` from `ui.cursor().top()`.
- For tables, row positions are approximated by dividing the block height evenly across visual rows (header + data, minus the separator line).

## Part B: Panel Z-Order Stacking

### Problem

Floating overlay panels (Find/Replace, Go to Line, Search in Files, file operation dialogs) used `egui::Window` with **no explicit ordering**. Their stacking depended on paint order within the frame, causing panels to sometimes appear behind the editor or other chrome — especially after window resize, tab switch, or theme change.

### Solution

All floating overlay panels now use `.order(egui::Order::Foreground)`, ensuring they consistently render above the main editor content.

| Panel | File | Change |
|-------|------|--------|
| Find/Replace | `src/editor/find_replace.rs` | Added `.order(egui::Order::Foreground)` |
| Go to Line | `src/ui/dialogs.rs` | Added `.order(egui::Order::Foreground)` |
| Search in Files | `src/ui/search.rs` | Added `.order(egui::Order::Foreground)` |
| File Create/Rename/Delete | `src/ui/dialogs.rs` | Added `.order(egui::Order::Foreground)` |
| Unsaved Changes | `src/app/dialogs.rs` | Added `.order(egui::Order::Foreground)` |
| Error Modal | `src/app/dialogs.rs` | Added `.order(egui::Order::Foreground)` |
| Portal Error | `src/app/dialogs.rs` | Added `.order(egui::Order::Foreground)` |
| Quick Switcher | `src/ui/quick_switcher.rs` | Already used `Order::Foreground` (no change) |

### Layer ordering

Within `Order::Foreground`, panels painted later in the frame appear on top. The composition order in `central_panel.rs` is:

1. `render_dialogs()` — Find/Replace, unsaved changes, error modals
2. Quick Switcher overlay
3. File operation dialogs
4. Go to Line dialog
5. Search in Files panel

This means modal-like panels (Quick Switcher, Go to Line) naturally stack above utility panels (Find/Replace), which is the intended UX.
