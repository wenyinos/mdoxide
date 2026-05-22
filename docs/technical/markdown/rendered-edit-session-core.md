# Rendered Edit Session — Core Types

Foundation types for the [Rendered Edit Session PRD](../../../ai-workflow/prds/prd-rendered-edit-session.md). **Start with the [architecture overview](./rendered-edit-session.md)** for motivation, commit policy, regression matrix, and design decisions. Block-type wiring is documented in linked pages below.

**Module:** `src/markdown/rendered_session.rs`

## Types

| Type | Purpose |
|------|---------|
| `BlockRef` | Stable block identity (line + kind); `widget_id` for egui TextEdit ids |
| `BlockEditState` | Per-block buffer, dirty flag, formatted edit mode, pending activation |
| `PendingActivation` | One-shot cursor/focus request for next frame |
| `CommitPolicy` | `SaveIfDirty` or `Discard` on close |
| `RenderedEditSession` | At most one `active` block; `HashMap` of buffers |

## Session API

| Method | Behavior |
|--------|----------|
| `switch_to` | Close previous (`SaveIfDirty`), set active, queue activation |
| `switch_to_ui` | Same + surrender previous block's egui focus |
| `close_active` / `close_active_ui` | Clear active; commit or discard per policy |
| `on_text_changed` | Update buffer, mark dirty (no source write) |
| `commit_active` | Force-commit active buffer via callback |
| `discard_active` | Reload buffer from source via callback |
| `invalidate_buffers` | Clear all state after `source_epoch` bump |
| `load_for_epoch` / `save_for_epoch` | Load/save with epoch tracking; auto-invalidate on mismatch |
| `open_formatted_display` | Set `formatted_editing = false` for a block |

Commit callbacks receive `(BlockRef, &BlockEditState)` so callers write to tab source without the session owning `Tab`. Undo is recorded at commit boundaries via [`rendered_commit_undo`](./rendered-edit-session-undo.md) (not per in-session keystroke).

## egui storage

Tab-scoped temp memory keyed by **editor id** (not global):

```rust
let session = rendered_session::load(ui, editor_id);
// ... mutate ...
rendered_session::save(ui, editor_id, session);
```

Storage id: `editor_id.with("rendered_edit_session")`.

## Widget ids

`BlockRef::widget_id(ui)` uses stable key paths under the rendered editor scope (e.g. `heading_text`, `para_text`, `list_item_text`, `table`/`cell`). Do not invent new suffixes when wiring render paths.

## Block wiring status

| Block type | Session-backed | Doc |
|------------|----------------|-----|
| Headings | Yes | [Headings](./rendered-edit-session-headings.md) |
| Plain paragraphs | Yes | [Paragraphs & lists](./rendered-edit-session-paragraphs-lists.md) |
| Simple list items | Yes | [Paragraphs & lists](./rendered-edit-session-paragraphs-lists.md) |
| Formatted blocks | Yes | [Formatted](./rendered-edit-session-formatted.md) |
| Tables | Yes (`BlockRef::TableCell` + force-commit signal) | [Tables](./rendered-edit-session-tables.md) |

**Widget identity:** `show_rendered_editor` uses `ui.push_id(editor_id)` + `ui.push_id(source_epoch)`. See [`rendered-widget-identity.md`](./rendered-widget-identity.md).

## Related

- [**Architecture overview**](./rendered-edit-session.md) — hub doc, RS-1…RS-7, open questions
- [Split view parity](./rendered-edit-session-split-view.md) — shared `rendered_editor_id` + epoch invalidation (RS-6)
- [Headings](./rendered-edit-session-headings.md) — first session-wired block type
- [Phase 0](./rendered-edit-session-phase0.md) — formatted blur hotfix + `source_epoch`
- [PRD](../../../ai-workflow/prds/prd-rendered-edit-session.md) — full architecture

## Tests

`cargo test rendered_session::` — switch/commit/discard invariants, `widget_id` suffix stability, storage roundtrip.
