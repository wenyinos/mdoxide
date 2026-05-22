# Rendered Widget Identity (`source_epoch`)

Rendered WYSIWYG blocks use a stable egui id hierarchy so commits during editing do not remap TextEdit focus/cursor state.

**PRD:** [Rendered Edit Session](../../../ai-workflow/prds/prd-rendered-edit-session.md)

## Id hierarchy

Inside `show_rendered_editor` (`src/markdown/editor.rs`):

```rust
ui.push_id(editor_id, |ui| {           // per tab (e.g. main_editor_rendered + tab.id)
    ui.push_id(source_epoch, |ui| {    // bumps only on external invalidation
        // Block widgets: ui.id().with("heading_text").with(line), etc.
    });
});
```

- **`editor_id`** — `rendered_editor_id(tab.id)` from `rendered_session.rs` (set in `central_panel.rs` for both rendered-only and split preview). See [split view parity](./rendered-edit-session-split-view.md).
- **`source_epoch`** — from `Tab::source_epoch()` via `MarkdownEditor::source_epoch(...)`.

Block-level suffixes are defined by `BlockRef::widget_id_in_scope` in `rendered_session.rs`.

## `content_hash` (culling only)

A hash of `self.content` remains for **viewport culling** and block height caches (`ViewportCullingState`). It is **not** used in `ui.push_id` for widget identity.

| Concern | Key |
|---------|-----|
| Widget focus / TextEdit ids | `editor_id` + `source_epoch` |
| Scroll culling / height cache | `content_hash` + available width |

## When `source_epoch` bumps

See [Phase 0 doc](./rendered-edit-session-phase0.md). Rendered WYSIWYG commits use `Tab::record_edit_from_snapshot()` and do **not** bump epoch.

## Related modules

| File | Role |
|------|------|
| `src/markdown/editor.rs` | `push_id` hierarchy in rendered view |
| `src/app/central_panel.rs` | Passes `tab.source_epoch()` into `MarkdownEditor` |
| `src/state.rs` | `Tab::source_epoch()`, bump helpers |
| `src/markdown/rendered_session.rs` | `BlockRef::widget_id_in_scope`, `rendered_widget_scope_id` |

## Tests

- `rendered_session::tests` — scope id stability vs epoch bump vs content_hash scope
- `markdown::editor::tests` — `source_epoch` builder; egui `push_id` integration for heading ids

## Manual verification

1. Open a doc with two headings in rendered mode.
2. Edit heading A — cursor should stay visible (no flash/disappear each keystroke).
3. Switch to raw, edit, return to rendered — content updates; epoch bumped (trace log on `Tab`).
4. Rendered-only edit — `source_epoch` unchanged in logs.
