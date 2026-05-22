## Rendered edit session — tables

Status: implemented (task 101)

Tables now participate in the same [`RenderedEditSession`](./rendered-edit-session-core.md) state machine as headings, paragraphs, and (formatted) list items, while preserving the deferred-commit ergonomics that make in-cell editing feel responsive.

For the broader session design (BlockRef, BlockEditState, switch_to_ui, epoch invalidation) see [`rendered-edit-session-core.md`](./rendered-edit-session-core.md). For the underlying widget mechanics (focus, Tab navigation, empty-cell clicks) see [`editable-tables.md`](./editable-tables.md) and [`table-cell-focus-navigation.md`](./table-cell-focus-navigation.md).

### Goals

1. A single click on a heading / paragraph / list item after editing a cell must commit the table (RS-5 flow).
2. Tabbing between cells inside the same table must **not** commit the table — buffered edits stay buffered (existing TBLE-1…TBLE-3 behaviours).
3. Click-outside dismissal (no widget hit) must commit the active table too.
4. Cross-table switches commit the source table once and activate the new cell.

### Model

The `BlockRef::TableCell { table_line, row, col }` variant from `rendered_session::BlockRef` is now actively populated: while a cell is focused, `session.active = Some(TableCell { … })`. Unlike other block kinds, the cell's text is **not** stored in `BlockEditState.text` — it lives in `TableData` inside the widget. The session entry exists solely so cross-block transitions go through the same `switch_to_ui` path everything else uses.

### Force-commit signal

Because `BlockEditState` doesn't carry cell text, `commit_session_block` for a `TableCell` cannot write to source directly. Instead it sets a one-shot egui `temp` flag scoped per table:

```rust
// widgets.rs
pub fn signal_table_force_commit(ctx: &egui::Context, table_line: usize);
fn take_table_force_commit(ui: &mut Ui, table_line: usize) -> bool;
```

`EditableTable::show()` consumes the flag at the start of its frame. If `content_modified` was set, it commits unconditionally (bypassing the `focus_lost` + `defer_commit_age >= 2` cycle) and reports `output.changed = true` so `render_table` writes the table back to source the same frame. If the table is clean, the flag clears without effect.

### Active block sync (`sync_table_cell_session_active`)

`EditableTable::show()` exposes the user's interaction target via `WidgetOutput.focused_cell`. The value is **gated** on actual focus or focus intent this frame:

```rust
let focused_cell_out =
    if any_cell_has_focus || edit_state.pending_focus.is_some() {
        edit_state.pending_focus.or(edit_state.focused_cell)
    } else {
        None
    };
```

This gate is critical. `edit_state.focused_cell` is sticky inside the widget (only assigned when a cell reports `response.has_focus()`, never cleared on focus loss). Reporting it unconditionally would let `sync_table_cell_session_active` see `Some(stale)` the frame **after** the user clicked a heading, falsely re-enter the cell into the session, and ping-pong focus back to the table on every frame — exactly the "flicker and lose focus" symptom from the first integration pass.

After `.show()`, `render_table` calls `sync_table_cell_session_active`, which reconciles `session.active` against the gated `focused_cell`:

| Previous `session.active`                       | New `focused_cell` | Action                                                                                |
|-------------------------------------------------|--------------------|---------------------------------------------------------------------------------------|
| `Some(target)` (same cell)                      | `Some(target)`     | No-op (just mark dismiss flag if clicked).                                            |
| `Some(TableCell { table_line: T, … })` (intra-T)| `Some(other in T)` | **Direct assign** — bypass `switch_to_ui` so no commit_fn fires. Preserves defer.     |
| Anything else (heading, paragraph, …, table B)  | `Some(cell in T)`  | `switch_to_ui` with `PendingActivation::default()`. Commits the previous block.       |
| any                                             | `None`             | Leave `session.active` untouched; widget-level `focus_lost` + dismiss path handles it.|

Direct assign for intra-table movement is the key correctness piece: routing every Tab/Shift+Tab through `switch_to_ui` would call `commit_fn` on the leaving cell, fire `signal_table_force_commit`, and flush the table on every cell move. The widget already commits exactly once when focus leaves the table; the session must mirror that.

### Dismiss-on-click-outside

`session_dismiss_if_clicked_outside` reuses the existing `session_active_clicked_key` egui temp flag. `sync_table_cell_session_active` (and the early-return same-cell branch) calls `mark_session_active_clicked_if_clicked` whenever the user clicks while focused on any cell of this table — that keeps dismiss from spuriously firing during in-table interaction. When the user clicks empty space outside any block, the flag stays false; dismiss runs `close_active`, the commit callback writes the force-commit signal, and the table flushes on its next render.

### Leaving a cell for a non-table block

End-to-end flow when the active block is `TableCell { table_line: T, row, col }` and the user clicks a heading:

1. Heading widget detects the click and calls `session.switch_to_ui(ctx, heading_ref, …)`.
2. `switch_to_ui` runs `close_active(SaveIfDirty)` on the previous active (the cell). The commit callback (defined inside `render_table`, `render_heading`, etc.) invokes `commit_session_block(ctx, TableCell { T, … }, …)`, which calls `signal_table_force_commit(ctx, T)`.
3. `session.active = Some(heading_ref)`. Heading takes focus.
4. Next frame, the table at `start_line == T` enters `EditableTable::show()`; `take_table_force_commit` returns `true`. With `content_modified`, the table writes its current markdown back to source via `update_table_in_source` and clears its dirty flag.

The same path covers cross-table switches: clicking a cell in table B calls `switch_to_ui` with the new `TableCell` target, fires the force-commit signal for table A, and B becomes active. A's next render flushes.

### Within-table Tab / Shift+Tab

The widget continues to manage cell focus via `TextEdit::lock_focus(true)` + manual `consume_key(SHIFT, Tab)` then `consume_key(NONE, Tab)`. The only change for the session is that after each frame, `WidgetOutput.focused_cell` reflects the user's intended next cell (via `pending_focus.or(focused_cell)`), and `sync_table_cell_session_active` updates `session.active` directly — no `switch_to_ui`, no commit_fn, no signal.

### Why a signal instead of writing cell text from `BlockEditState`

Two alternatives were considered and rejected:

- **Mirror cell text into `BlockEditState`.** Would require updating `BlockEditState.text` on every keystroke and serializing the whole table when committing a single cell — duplicate state with no benefit, since `TableData` already owns the canonical buffer.
- **Pass `session` into `EditableTable::show()`.** Couples the widget to session internals and forces every widget caller (HTML export, future read-only renderers) to construct a session. The signal/output decoupling keeps `widgets.rs` orthogonal to session lifecycle.

### Files touched

| File                                | Change                                                                                       |
|-------------------------------------|----------------------------------------------------------------------------------------------|
| `src/markdown/widgets.rs`           | `WidgetOutput.focused_cell` + `with_focused_cell`; `signal_table_force_commit` helpers; `EditableTable::show` consumes the signal and populates `focused_cell`. |
| `src/markdown/editor.rs`            | `commit_session_block` takes `ctx`, signals on `TableCell`; `render_table` takes `&mut RenderedEditSession` and runs `sync_table_cell_session_active`; legacy `render_node` call site updated. |
| `src/markdown/rendered_session.rs`  | Tests covering cross-block enter, cross-block exit, intra-table direct assign, cross-table switch, epoch invalidation for TableCell. |

### Regression coverage

Existing manual cases TBLE-1…TBLE-3 in [`v0.3.0-regression-matrix.md`](../platform/v0.3.0-regression-matrix.md) §3.11 are preserved. Cross-block table scenarios (RS-5) are in §3.12 — see [`rendered-edit-session.md`](./rendered-edit-session.md).

- **TBLE-1 / TBLE-2** (click empty cell after Add column / Add row): still routed through `EditableTable`'s display-mode hit-rect; session sync runs after and updates `session.active` from `output.focused_cell` once the cell becomes focused.
- **TBLE-3** (Tab through consecutive empties): per-cell buffer logic unchanged; session sees intra-table movement and never fires commit_fn during Tab traversal.

New scenarios the session enables, covered by unit tests in `src/markdown/rendered_session.rs`:

- `switch_from_heading_to_table_cell_commits_heading`
- `switch_from_table_cell_to_heading_invokes_commit_for_cell`
- `intra_table_direct_assign_does_not_fire_commit`
- `cross_table_switch_invokes_commit_for_source_cell`
- `invalidate_buffers_clears_active_table_cell`

Plus signal-mechanism tests in `src/markdown/widgets.rs`:

- `test_widget_output_with_focused_cell`
- `test_table_force_commit_signal_roundtrip`
- `test_table_force_commit_take_is_one_shot`
