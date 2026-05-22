# Rendered Edit Session — Phase 0 (Hotfix + source_epoch)

> **Note:** Legacy `rendered_focus.rs` was removed in task 104. Formatted blur hotfix and defer logic described below were superseded by the full [`RenderedEditSession`](./rendered-edit-session.md) coordinator.

Foundation work for the [Rendered Edit Session PRD](../../../ai-workflow/prds/prd-rendered-edit-session.md): stop formatted-block stuck states and introduce per-tab external-invalidation tracking before the full session coordinator lands.

## Problem (pre-Phase 0)

Rendered WYSIWYG editing used fragmented focus/defer logic:

- `ui.push_id(content_hash, …)` remapped all widget ids on every keystroke commit.
- Formatted click-to-edit used `formatted_exit_should_save`, which could defer exit on blur but callers only checked it on the `lost_focus()` frame — `editing` never returned to `false`, leaving raw `**bold**` TextEdit visible.

## Phase 0 — Formatted blur hotfix

**Module:** `src/markdown/rendered_focus.rs`

`formatted_exit_should_save` now saves and exits **immediately** on blur (`focus_lost` → return true). Deferred exit heuristics for formatted blocks were removed. Cross-widget defer for always-on TextEdit paths remains via `focus_loss_should_commit`.

| Path | Behavior |
|------|----------|
| Formatted paragraph / list item blur | Immediate commit + `editing = false` |
| Heading / plain paragraph blur | Still uses `focus_loss_should_commit` defer |

**Call sites:** four branches in `src/markdown/editor.rs` (structural + non-structural formatted paragraphs and list items).

**Tests:** `rendered_focus::tests` — blur saves immediately, no defer when cross-widget switch pending, stale defer state cleared.

**Known trade-off:** Switching between formatted blocks may require two clicks again until the session coordinator (task 96+) handles programmatic block switch.

## Phase 0 — `source_epoch` on `Tab`

**Module:** `src/state.rs` (`Tab`)

Per-tab counter for **external** content invalidation only. Future rendered widget ids will use `ui.push_id(source_epoch, …)` instead of `content_hash`. Viewport culling continues to use content hash independently.

### API

| Method | Purpose |
|--------|---------|
| `Tab::source_epoch()` | Read current epoch |
| `Tab::bump_source_epoch()` | Saturating +1 (trace log) |
| `Tab::notify_external_content_change()` | After direct `content` assignment: bump `content_version` + `source_epoch` |
| `Tab::record_edit_from_snapshot()` | Rendered edits — **does not** bump epoch |
| `Tab::record_external_edit_from_snapshot()` | Raw / external edits — bumps epoch when content changes |

### When epoch bumps

| Event | Bumps? |
|-------|--------|
| Raw FerriteEditor sync | Yes (`record_external_edit_from_snapshot`) |
| `set_content`, undo, redo, `record_edit` | Yes |
| File reload, find/replace, frontmatter, formatting (Ferrite) | Yes (`notify_external_content_change`) |
| `finish_loading` (background open) | Yes |
| Tree viewer edits | Yes (`record_external_edit_from_snapshot`) |
| Rendered / split rendered WYSIWYG commit | **No** (`record_edit_from_snapshot`) |

### Widget identity (task 97)

`show_rendered_editor` uses `ui.push_id(editor_id)` + `ui.push_id(source_epoch)` for widget scope. See [`rendered-widget-identity.md`](./rendered-widget-identity.md).

## Related modules

| File | Role |
|------|------|
| `src/markdown/rendered_focus.rs` | Cross-widget focus, defer commit, formatted blur |
| `src/state.rs` | `Tab::source_epoch`, invalidation helpers |
| `src/editor/widget.rs` | Raw pane → `record_external_edit_from_snapshot` |
| `src/app/central_panel.rs` | Split/rendered → `record_edit_from_snapshot` |

## Next steps (PRD roadmap)

1. ~~**Task 96:** `BlockRef`, `RenderedEditSession` in `src/markdown/rendered_session.rs`~~
2. ~~**Task 97:** Replace `ui.push_id(content_hash)` with `source_epoch` widget scope~~
3. Wire session into heading/paragraph paths; migrate `StickyFocus` → `BlockRef`

## Manual verification

- Formatted list/paragraph: click to edit, click away → returns to styled display (not stuck raw markdown).
- Raw mode typing → `source_epoch` increases (trace log or debugger on `Tab`).
- Rendered heading edit → `source_epoch` unchanged.
- Undo/redo / file reload → `source_epoch` increases.
