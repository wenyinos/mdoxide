# Table cell focus and keyboard navigation

Rendered GFM tables use `EditableTable` in `src/markdown/widgets.rs`. This note covers **keyboard focus** during in-cell editing (Tab / Shift+Tab) and **hit-testing** for empty display-mode cells.

For general table behavior (deferred commits, toolbar, markdown sync), see [`editable-tables.md`](./editable-tables.md). For **session-triggered commit** when leaving a table for another block type, see [`rendered-edit-session-tables.md`](./rendered-edit-session-tables.md).

## Empty cells — display hit area ([issue #131](https://github.com/OlaProeis/Ferrite/issues/131))

While a cell is **not** focused, it draws in display mode using a formatted galley. Empty text yields a galley with no extent; an interactive `Label` had **zero click size**, so users could not click into new empty slots. The widget now allocates the padded inner rectangle with `allocate_exact_size(.., Sense::click())` and paints the galley at that rect’s origin so the visible cell accepts clicks like populated cells.

## Tab keeps focus inside the table

Plain `TextEdit` (default `lock_focus`) treats **Tab** as **move to the next widget** in egui’s global tab order, so focus left the grid before internal `pending_focus` navigation could behave like **Enter**.

Table cells use **`TextEdit::lock_focus(true)`** so Tab is reserved for grid navigation. **Tab / Shift+Tab** are **`consume_key`’d before `TextEdit::show`** so `\t` and TextEdit indentation helpers do not mutate cell text.

## Shift+Tab must be consumed before plain Tab

`InputState::consume_key` uses **`Modifiers::matches_logically`**. A pattern of **`Modifiers::NONE` matches Shift+Tab** (extra Shift is allowed). Always call **`consume_key(SHIFT, Tab)` before `consume_key(NONE, Tab)`**, then handle **`move_prev` before `move_next`** in the same frame.

## Manual regression

Manual cases **`TBLE-1` … `TBLE-3`** live in [`v0.3.0-regression-matrix.md`](../platform/v0.3.0-regression-matrix.md) §3.11. Rendered-session cross-block cases **`RS-1` … `RS-7`** (including RS-5 table → heading) are in §3.12 — see [`rendered-edit-session.md`](./rendered-edit-session.md).
