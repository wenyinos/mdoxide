# Handover: Table Text Wrapping & Layout Fixes

## Rules (DO NOT UPDATE)
- Never auto-update this file - only update when explicitly requested
- Run `cargo build` after changes to verify code compiles
- Follow existing code patterns and conventions
- Use Context7 MCP tool to fetch library documentation when needed
- Document by feature (e.g., memory-optimization.md), not by task
- Update docs/index.md when adding new documentation
- **Branch**: `master`

---

## Current Task

**Fix Table Rendering — Text Wrapping, Backgrounds, and Cell Layout**
- **Priority**: High
- **Status**: Done — background alignment, cell padding, and column resize all implemented
- **Test file**: `test_md/test_table_wrapping.md`

### Goal (All Achieved)

Tables in the rendered/split markdown editor now:
1. **Fit within screen width** (or max_line_width when set, for zen mode centering)
2. **Wrap long text** inside cells gracefully
3. **Keep short text on one line** (e.g., "Alice", "QA", "ID" should never wrap)
4. Have **clean, aligned backgrounds** (header, striped rows) with no bleeding
5. Have **proper cell padding** — text does not overflow or offset from its row
6. **Columns are user-resizable** by dragging separators; double-click resets to auto

### What's Already Done

The table rendering was refactored from the original `TextEdit::singleline` (no wrapping) approach. Several iterations were tried:

1. ~~`egui_extras::TableBuilder`~~ — Removed. Required pre-measured row heights that never matched actual column widths. Caused "First use of Table ID" errors with multiple tables. Dependency (`egui_extras`) was removed from `Cargo.toml`.

2. ~~`egui::Grid`~~ — Removed. Grid determines column widths from natural content width and ignores parent `set_max_width` constraints, causing tables to overflow the viewport.

3. **Current approach: Manual layout** (`ui.horizontal()` + `ui.allocate_ui()`) — This is the current implementation. It correctly constrains width but has visual issues that need fixing.

### Current Architecture

**Width flow:**
- `MarkdownEditor` in `editor.rs` sets `ui.set_max_width(content_width)` based on `max_line_width` setting and zen mode
- `render_table()` in `editor.rs` (line ~4180) captures `ui.available_width()` BEFORE any layout changes, passes it to the table widget via `.max_width(table_avail_width)`
- `EditableTable::show()` in `widgets.rs` (line ~1400) uses `min(max_width, ui.available_width())` as the hard table width cap
- The `ui.horizontal()` wrapper was **removed** from `render_table` — it was causing width to be unbounded

**Column width calculation (widgets.rs, ~line 1490):**
- Measures each column's single-line natural width using `ui.fonts(|f| f.layout_no_wrap(...))`
- If all columns fit naturally: scales up proportionally
- If too wide: short columns (≤ fair_share) keep natural width; long columns share remaining space proportionally

**Row rendering (widgets.rs, ~line 1630):**
- Pre-measures row heights using `ui.fonts(|f| f.layout(..., wrap_w))` at exact column widths (used as minimum)
- Reserves a `Shape::Noop` placeholder in the paint list BEFORE content
- Each row is a `ui.horizontal()` with `set_min_width`/`set_max_width` = `table_width`
- Each cell uses `ui.allocate_ui_with_layout(cell_size, Layout::top_down)` with `set_min_width`/`set_max_width` (no max height constraint — cells grow to fit)
- `ui.add_space(cell_v_pad)` correctly adds vertical padding (top_down layout)
- Text uses `TextEdit::multiline` with a custom `LayoutJob::simple` layouter for word wrapping
- Newlines are stripped (Enter key navigates to next row instead)
- After each row renders: background painted into reserved slot using actual row rect (via `ui.painter().set(bg_idx, Shape::rect_filled(...))`)

**Column resizing (widgets.rs, after row loop):**
- 6px-wide invisible interaction zones centered on each vertical column separator
- Drag adjusts left column `+delta`, right column `-delta`, both clamped to `min_col_width` (40px)
- Custom widths stored in `TableEditState.custom_col_widths` (egui memory, per table)
- On each frame, custom widths are normalized to current `table_width` (proportional scaling)
- Double-click resets to auto-calculated widths
- Visual: resize cursor on hover, guide line during drag

**Cell editing:**
- Tab/Shift+Tab = next/prev cell
- Enter = move down one row
- Escape = deselect
- Focus tracking via `TableEditState` stored in egui memory

### Previously Reported Problems (All Fixed)

1. ~~**Background color bleeding/misalignment**~~ — Fixed. Backgrounds now use actual rendered row rect instead of pre-measured height.

2. ~~**Text vertical offset**~~ — Fixed. Cell layout changed from inherited `left_to_right` to explicit `top_down`, so `add_space(cell_v_pad)` correctly adds vertical padding.

3. ~~**Text rendering outside row bounds**~~ — Fixed. Removed `set_max_size` height constraint on cells; cells grow to fit content, and backgrounds match.

### How It Was Fixed

Used the **Shape::Noop placeholder technique** (same as egui's `Frame::show()` internals): reserve a paint slot before rendering content, then replace it with the actual background rect after the row renders. This guarantees backgrounds always match content dimensions. Combined with `top_down` cell layout for correct padding and no max-height constraint so cells expand naturally.

### Key Files

| File | Purpose | Key Lines |
|------|---------|-----------|
| `src/markdown/widgets.rs` | `EditableTable` struct + `show()` method — all table rendering | ~1327-1800 |
| `src/markdown/editor.rs` | `render_table()` — calls EditableTable, passes width | ~4180-4250 |
| `src/config/settings.rs` | `MaxLineWidth` enum — Off, Col80, Col100, Custom(u16) | ~1780 |
| `test_md/test_table_wrapping.md` | Test file with various table scenarios | all |

### What NOT to Do

- **Do NOT use `egui_extras::TableBuilder`** — it was tried and has fundamental issues with egui 0.28 (no `id_source`, row height mismatch)
- **Do NOT use `egui::Grid`** — it ignores parent width constraints
- **Do NOT wrap the table in `ui.horizontal()`** in `render_table()` — horizontal layouts in egui expand to fit content, defeating width constraints
- **Do NOT paint backgrounds AFTER content** without using a lower paint layer — it covers the text

---

## Recently Completed (This Session)

- **Table background & layout fix** — Shape::Noop placeholder technique for backgrounds, `top_down` cell layout, removed max-height constraint. All three visual issues (background bleeding, text offset, text outside bounds) resolved.
- **Column resizing** — Draggable column separators with proportional width persistence, min-width enforcement, double-click reset, visual feedback.

## Previously Completed

- **Table wrapping foundation** — manual layout with proportional column widths, text wrapping via `TextEdit::multiline` + `LayoutJob`, width respects `max_line_width`/zen mode
- **Task 29**: Always show view mode bar for all editor tabs (DONE)
- **Task 26**: Windows MSI installer overhaul (DONE)
- **Task 20 + 21**: Vim mode settings toggle, status bar indicator (DONE)
- **Task 19**: Lazy CSV row parsing with byte-offset indexing (DONE)
- **Task 30**: Light mode text readability fix (DONE)

---

## Environment

- **Project**: Ferrite (Markdown editor)
- **Language**: Rust
- **GUI Framework**: egui 0.28
- **Branch**: `master`
- **Build**: `cargo build`
- **Version**: v0.2.7 (in progress)
