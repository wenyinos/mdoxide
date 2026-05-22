# Rendered Edit Session — Split View Parity

Split view and rendered-only mode share one [`RenderedEditSession`](./rendered-edit-session-core.md) and the same `source_epoch` invalidation path per tab.

**PRD:** [Rendered Edit Session](../../../ai-workflow/prds/prd-rendered-edit-session.md) (RS-6, split view parity)

## Problem

Split view previously used a separate egui editor id (`split_preview_rendered`) while rendered-only mode used `main_editor_rendered`. That created **two** session instances per tab in egui temp memory — switching modes or editing across panes could leave stale block buffers or lose active edit state.

## Unified editor id

Both panes now call `rendered_editor_id(tab.id)` from `src/markdown/rendered_session.rs`:

```rust
pub fn rendered_editor_id(tab_id: usize) -> Id {
    Id::new("main_editor_rendered").with(tab_id)
}
```

Wired in `src/app/central_panel.rs` for:

- `ViewMode::Rendered` — full rendered editor
- `ViewMode::Split` — right preview pane (left pane is raw `EditorWidget`)

Same id ⇒ same `RenderedEditSession`, viewport-culling cache, and widget-id scope when toggling R/S/V or switching between rendered and split.

## Shared `source_epoch`

Both panes read `Tab::source_epoch()` via `MarkdownEditor::source_epoch(...)`.

| Edit source | Epoch bump? | Undo path |
|-------------|-------------|-----------|
| Raw pane (split left) | Yes — `EditorWidget` → `Tab::record_external_edit_from_snapshot()` | Operation diff via FerriteEditor snapshot |
| Rendered block commit | No — `Tab::record_edit_from_snapshot()` | One logical undo step per block commit — see [undo granularity](./rendered-edit-session-undo.md) |
| Undo / redo | Yes — `Tab::undo()` / `redo()` | Clears session buffers via epoch |

On epoch mismatch, `load_for_epoch` calls `RenderedEditSession::invalidate_buffers()` (trace log includes stored vs current epoch and editor id). Raw edits while a rendered block is active are **authoritative**: buffers clear; the user reloads from source on next block activation.

## Rendered → raw consistency

Rendered commits write directly to `Tab::content`. The raw pane reads the same string on the next frame — no extra sync step. Do not bump `source_epoch` on rendered commits (stable TextEdit widget ids within a session).

## Related modules

| File | Role |
|------|------|
| `src/markdown/rendered_session.rs` | `rendered_editor_id`, `load_for_epoch`, `save_for_epoch` |
| `src/app/central_panel.rs` | Passes `rendered_editor_id(tab.id)` + `source_epoch` to both rendered paths |
| `src/editor/widget.rs` | Raw split pane; bumps epoch on content change |
| `src/state.rs` | `source_epoch`, `record_external_edit_from_snapshot`, undo/redo epoch bumps |
| `src/markdown/editor.rs` | `show_rendered_editor` — session load/save inside `push_id(editor_id)` + `push_id(source_epoch)` |

## Tests

`cargo test rendered_session::` (when test suite compiles):

- `rendered_editor_id_unifies_single_and_split_view`
- `session_persists_across_rendered_view_mode_switch`
- `rs6_raw_edit_epoch_bump_invalidates_session_buffers`
- `load_for_epoch_invalidates_when_epoch_changes`

## Manual verification (RS-6)

1. Open a markdown doc; switch to **Split** (R → S or view toggle).
2. Click a heading in the rendered pane; type several characters (do not blur yet).
3. Click the raw pane; append a line of text.
4. Confirm the rendered pane shows the new raw content on the next frame.
5. Click the heading again — it should reflect source (prior in-buffer edit discarded unless committed before raw edit).
6. Toggle Rendered ↔ Split — active session state (if any committed block) should feel consistent; no duplicate focus traps.
