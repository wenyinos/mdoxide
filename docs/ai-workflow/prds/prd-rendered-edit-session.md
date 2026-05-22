# PRD: Rendered Edit Session — Consolidated WYSIWYG Editing Architecture

## Version Target

**v0.3.x** (architectural refactor; may ship incrementally across patch releases)

## Priority

**HIGH** — Rendered and split-view editing is a core product differentiator. Current patchwork focus/commit logic is fragile, causes regressions (stuck edit state, cursor flash, double-click to switch), and blocks further WYSIWYG work (ROADMAP: cursor drift, large-file rendered editing, shaped-text WYSIWYG).

## Overview

Replace the fragmented per-widget focus and commit hacks in rendered mode with a **single edit session coordinator**, **stable widget identity**, and a **unified buffer-then-commit policy**. The goal is reliable one-click block switching, correct cursor placement, and predictable exit from edit mode — without fighting egui’s focus model or remapping every widget id on each keystroke.

This PRD describes **Path 2**: keep rendered mode as an editable view, but consolidate architecture. It does **not** pivot to preview-only or raw-only formatting.

---

## Problem Statement

### User-visible symptoms (current)

| Symptom | Example |
|---------|---------|
| Double-click to switch blocks | Edit heading A, click heading B — first click only defocuses A |
| Cursor flash then disappear | Click between letters in rendered mode; caret appears briefly then vanishes |
| Stuck in edit mode | Formatted list/paragraph stays in raw `**bold**` TextEdit after clicking away |
| Stuck in display mode | Opposite failure mode after botched defer logic |
| Cross-widget inconsistency | Tables behave differently from headings, lists, formatted paragraphs |

### Root causes (architectural)

1. **`content_hash` id scope** — `ui.push_id(content_hash, …)` remaps **all** inner widget ids whenever `self.content` changes. Any commit during a focus transition destroys egui focus/cursor state.

2. **Two editing paradigms** — Always-on `TextEdit` (headings, plain paragraphs) vs click-to-edit toggle (formatted inline content). Each has different exit paths and state storage.

3. **Focus-driven commits** — Source is updated on `lost_focus()` edges. egui often consumes the first click to defocus; timing between commit → re-parse → id remap → new focus is racy.

4. **Per-widget patches** — Tables use deferred commit + `TableGlobalFocus`. Headings use `rendered_focus` defer. Formatted items use `FormattedItemEditState` + broken defer-on-blur. No single owner of “which block is active”.

5. **Broken formatted defer** — `formatted_exit_should_save` defers on the `lost_focus()` frame only; caller never re-invokes on later frames, so `editing = false` never runs.

---

## Goals

### Must have

1. **One active block** — At most one rendered block (heading, paragraph, list item, formatted item, or table cell) is “open” for editing at a time.
2. **Explicit block switch** — Clicking block B **programmatically closes** block A (save buffer → source if dirty), then opens B with correct cursor — **one click**, including after typing.
3. **Stable widget ids during rendered editing** — Commits from rendered mode do not invalidate egui widget ids mid-session.
4. **Unified commit policy** — Edit in temp buffers; write to source only via session `close_block()` (or equivalent), not scattered `lost_focus()` handlers.
5. **Formatted click-to-edit preserved** — Display styled text; click enters raw edit with galley-based cursor placement (existing algorithm, session-owned lifecycle).
6. **Table integration** — Tables plug into the same session model; existing keyboard nav (Tab, Enter) and deferred table commit behavior preserved or improved.
7. **External invalidation** — Raw mode edits, file reload, tab switch, and split-sync still force widgets to pick up new source (via epoch bump, not per-keystroke hash).

### Should have

8. **Split view parity** — Same session semantics in split (rendered pane).
9. **Undo integration** — Block commit produces one logical undo step (or documented interaction with existing tab undo).
10. **Diagnostics** — Debug logging behind `log` trace for session transitions (close/open/switch/epoch bump).

### Non-goals (this PRD)

- True rich-text WYSIWYG (edit bold without seeing `**`) — future / Phase 4 shaping work
- Replacing egui `TextEdit` with FerriteEditor in rendered mode
- Rendered-mode undo stack separate from tab undo
- Multi-cursor in rendered mode
- Preview-only rendered view (Path 3 pivot)

---

## Proposed Architecture

### High-level diagram

```
┌─────────────────────────────────────────────────────────────┐
│  TabState / MarkdownEditor                                   │
│  ┌─────────────────────┐    ┌──────────────────────────────┐ │
│  │ source_epoch        │    │ RenderedEditSession          │ │
│  │ (external changes)  │    │  active: Option<BlockRef>    │ │
│  └──────────┬──────────┘    │  buffers: HashMap<BlockRef>  │ │
│             │               │  switch_to / close / open    │ │
│             v               └──────────────┬───────────────┘ │
│  ui.push_id(tab_id)                        │                 │
│    ui.push_id(source_epoch)  ◄─────────────┘                 │
│      render blocks → ask session before focus/click            │
└─────────────────────────────────────────────────────────────┘
```

### Component 1: `source_epoch` (stable id scope)

**Replace** `content_hash` in `ui.push_id` for editable widget identity.

| Event | Bump epoch? |
|-------|-------------|
| Rendered block commit (session `close_block`) | **No** |
| Raw mode edit in same tab | **Yes** |
| File reload / external content replace | **Yes** |
| Tab switch (use tab-scoped session; epoch per tab) | N/A (different memory) |
| Undo/redo restoring content | **Yes** |
| Split sync applying remote pane change | **Yes** (if content replaced from other pane) |

**Storage:** `source_epoch: u64` on tab state (or egui temp keyed by tab id). Increment with saturating add.

**Id hierarchy:**

```rust
ui.push_id(editor_id, |ui| {           // per tab / editor instance
    ui.push_id(source_epoch, |ui| {    // bumps only on external invalidation
        // Block widgets: ui.id().with("heading_text").with(line)
    });
});
```

**Viewport culling** may continue to use content hash for height-cache invalidation — that is independent of widget focus ids.

**Reference:** `show_rendered_editor` ~L1101–1171 in `src/markdown/editor.rs`.

---

### Component 2: `BlockRef` (stable block identity)

Line-based identity survives epoch and matches existing `StickyFocus` concept; unify into one type.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockRef {
    Heading { line: usize, structural: bool },
    Paragraph { line: usize },
    ListItem { line: usize, item: u32 },
    FormattedParagraph { line: usize, structural: bool },
    FormattedListItem { line: usize, item: u32, structural: bool },
    TableCell { table_line: usize, row: usize, col: usize },
}
```

**Rules:**

- `line` is **1-indexed** source line (consistent with existing code).
- List `item` is index within parent list node (consistent with formatted list ids today).
- `structural` distinguishes structural-key code paths (`*_sk` widgets) from plain paths.

**Widget id:** `BlockRef::widget_id(ui) -> Id` — same key scheme as current `StickyFocus` in `rendered_focus.rs` (migrate/rename).

---

### Component 3: `RenderedEditSession`

Single coordinator stored in egui temp memory keyed by **tab/editor id** (not global `Id::new("ferrite_rendered_edit_focus")`).

```rust
pub struct RenderedEditSession {
    /// Currently open block, if any.
    pub active: Option<BlockRef>,
    /// Per-block edit buffers (text + mode flags).
    pub blocks: HashMap<BlockRef, BlockEditState>,
}

pub struct BlockEditState {
    /// TextEdit buffer (raw markdown for formatted blocks).
    pub text: String,
    /// For formatted blocks: false = display mode, true = raw edit mode.
    pub formatted_editing: bool,
    pub dirty: bool,
    /// One-shot: apply focus + cursor next frame.
    pub pending_activation: Option<PendingActivation>,
}

pub struct PendingActivation {
    pub cursor_char_index: Option<usize>,
    pub request_focus: bool,
}
```

#### Session API (conceptual)

| Method | Behavior |
|--------|----------|
| `switch_to(ctx, block, activation)` | If `active != block`: `close_active(CommitPolicy::SaveIfDirty)`; set active; merge/create `BlockEditState`; set `pending_activation` |
| `close_active(policy)` | If formatted: set `formatted_editing = false`; if dirty and policy saves: flush buffer → source; surrender egui focus; clear `active` |
| `open_formatted_display(block)` | Ensure block exists in map; `formatted_editing = false` |
| `on_text_changed(block)` | Mark dirty; update buffer; **do not** write source |
| `commit_active()` | Force save active buffer to source; mark clean |
| `discard_active()` | Reload buffer from source; mark clean (Escape on formatted) |

**Activation entry points** (all call `switch_to`, never raw egui focus hacks alone):

- Click on always-on TextEdit (heading/plain paragraph/list)
- Click on formatted display area (with galley cursor mapping)
- Table cell click / Tab navigation (delegates within table or switches from non-table block)
- Keyboard focus tab order (if applicable)

**Close triggers:**

- `switch_to` another block
- Click outside document / on non-editable chrome (optional: close with save)
- Enter/Escape on formatted blocks (existing UX)
- Tab close / mode switch to raw (commit all pending)

---

### Component 4: Unified commit policy

**Principle:** Source string mutates only inside session-controlled commit functions.

| Block type | While editing | On close/switch |
|------------|---------------|-----------------|
| Heading | Buffer in session or existing temp buffer | `update_source_line` |
| Plain paragraph / list item | Buffer in session | `update_source_range` |
| Formatted paragraph / list item | Raw text in buffer; display when `formatted_editing == false` | `update_source_range`; exit formatted edit mode |
| Table | Keep existing cell buffer model | Commit table when leaving table **or** on session switch out of table |

**Remove:**

- Per-widget `focus_loss_should_commit` defer frames
- `formatted_exit_should_save` defer
- `restore_switch_focus` per-frame focus repair
- `after_text_edit` cross-widget `primary_pressed` steal

**Replace with:** `session.switch_to(target, activation)` at click/focus boundaries.

---

### Component 5: Formatted click-to-edit (session-owned)

Preserve existing display ↔ edit UX documented in [`click-to-edit-formatting.md`](../../technical/markdown/click-to-edit-formatting.md).

**Flow (unchanged UX, new owner):**

1. Display mode: render inline nodes styled; `Sense::click()` on block rect.
2. On click: `session.switch_to(block, PendingActivation { cursor, request_focus: true })`.
3. Session sets `formatted_editing = true`, loads raw text from source if buffer cold.
4. Next frame: TextEdit shown; apply `pending_activation` (focus + `CCursor` from galley mapping).
5. On close: save raw → source, `formatted_editing = false`.

**Cursor mapping:** Reuse `compute_displayed_cursor_index`, `map_displayed_to_raw` — no algorithm change in phase 1.

---

### Component 6: Table integration

Tables already buffer edits and defer source commit until leaving the table ([`table-editing-focus.md`](../../technical/markdown/table-editing-focus.md)).

**Integration plan:**

- `BlockRef::TableCell` participates in `switch_to`.
- Switching **from** table cell **to** non-table: table commits if `content_modified` (existing), then session opens new block.
- Switching **between** tables or cells: table internal logic handles; session `active` updates but table commit deferred until leaving table entirely.
- Remove duplicate cross-widget checks in `widgets.rs` that mirror `rendered_focus` (`has_other_focus`, etc.) — session is authoritative.

**Keep:** `TextEdit::lock_focus(true)`, Tab/Shift+Tab consumption, empty-cell hit testing ([#131](https://github.com/OlaProeis/Ferrite/issues/131)).

---

## User Experience

### Personas

- **Everyday writer** — Uses rendered or split mode for notes; expects Word-like click-and-type.
- **Power user** — Switches raw ↔ rendered; expects no lost edits or focus traps.

### Key flows

| Flow | Expected behavior |
|------|-------------------|
| Click between letters in heading | Cursor lands at click position; stays visible |
| Edit heading A, click heading B | One click; A saved; B focused at click |
| Click formatted list item | Styled display → raw edit; cursor near click |
| Edit formatted item, click plain paragraph | Formatted saves; paragraph opens |
| Edit table cell, click heading | Table commits; heading opens |
| Edit in raw pane (split) | Rendered epoch bumps; rendered buffers invalidated/reloaded |
| Escape in formatted edit | Discard buffer; return to display mode |
| Click away (no other block) | Active block saves and closes |

### UX invariants

- Hover shows text cursor over editable blocks (unchanged).
- No block left in raw formatted edit mode after switching away.
- No double-click requirement for block switching after editing.

---

## Development Roadmap (Phases)

Phases are ordered by dependency. Each phase should be shippable without breaking raw mode.

### Phase 0 — Stop the bleeding (pre-requisite)

**Scope:** Revert or disable broken `rendered_focus` defer paths that cause stuck edit state; minimal regression to “double-click sometimes” acceptable for one patch.

- Remove `formatted_exit_should_save` defer; immediate save + `editing = false` on blur **or** route through session stub.
- Document known regressions in CHANGELOG.

**Exit criteria:** Formatted items never stuck in edit mode.

---

### Phase 1 — Foundation: epoch + session skeleton

**Scope:**

- Add `source_epoch` to tab state; bump on external invalidation only.
- Replace `ui.push_id(content_hash, …)` with `ui.push_id(source_epoch, …)` for widget tree.
- Introduce `src/markdown/rendered_session.rs` with `BlockRef`, `RenderedEditSession`, `switch_to` / `close_active` (heading + plain paragraph only).
- Wire heading click/focus through session; remove heading-specific `rendered_focus` defer.

**Exit criteria:**

- Heading ↔ heading one-click switch after edit works.
- Raw mode edit bumps epoch; rendered picks up new text.
- Rendered heading edit does not bump epoch.

**Files (primary):**

- `src/markdown/rendered_session.rs` (new)
- `src/markdown/editor.rs`
- `src/state.rs` (epoch on tab)
- Deprecate much of `src/markdown/rendered_focus.rs`

---

### Phase 2 — Plain blocks + list items

**Scope:**

- Extend session to `Paragraph`, `ListItem` (non-formatted).
- Migrate plain TextEdit buffers into session or keep temp buffers synced with session dirty flag.
- Remove `focus_loss_should_commit` for these block types.

**Exit criteria:**

- Cross-block switching among headings, paragraphs, list items stable.
- List item index correctness preserved (see [`prd-v0.2.0-list-editing-bug.md`](./prd-v0.2.0-list-editing-bug.md)).

---

### Phase 3 — Formatted click-to-edit

**Scope:**

- Move `FormattedItemEditState` fields into `BlockEditState`.
- Display/edit rendering reads session state, not scattered temp `edit_state` keys.
- Galley cursor activation via `PendingActivation` only.
- Delete `formatted_exit_should_save` and display-mode `prepare_switch` hacks.

**Exit criteria:**

- Formatted paragraph/list: click → cursor → edit → switch away → display mode restored.
- ROADMAP cursor drift cases no worse than today; target improvement in follow-up.

---

### Phase 4 — Table integration

**Scope:**

- Connect `EditableTable` / `TableGlobalFocus` to session.
- Single code path for “leave table for non-table block”.
- Remove redundant `rendered_focus` usage from `widgets.rs`.

**Exit criteria:**

- All Phase 1–3 flows work with tables in document.
- TBLE-1…TBLE-3 regression matrix passes ([`v0.3.0-regression-matrix.md`](../../technical/platform/v0.3.0-regression-matrix.md)).

---

### Phase 5 — Split view + undo + cleanup

**Scope:**

- Split pane: shared epoch/session per tab (rendered pane owns session).
- Undo: on `commit_active`, integrate with tab edit history (single snapshot per block close — detail in implementation).
- Delete `rendered_focus.rs` if fully superseded.
- Update technical docs; add [`rendered-edit-session.md`](../../technical/markdown/rendered-edit-session.md).

**Exit criteria:**

- Split view switching matches rendered-only behavior.
- No dead focus/commit code paths.

---

## Logical Dependency Chain

```
Phase 0 (hotfix)
    ↓
Phase 1 (epoch + session + headings)     ← first usable vertical slice
    ↓
Phase 2 (paragraphs + lists)
    ↓
Phase 3 (formatted)                       ← highest UX risk; depends on stable epoch
    ↓
Phase 4 (tables)
    ↓
Phase 5 (split, undo, docs, delete legacy)
```

**First demo milestone:** Phase 1 complete — heading-only documents prove epoch + session.

---

## Success Criteria (acceptance)

### Functional

- [ ] **RS-1:** Edit heading A, single-click heading B → B focused, A persisted, no double-click.
- [ ] **RS-2:** Click between characters in heading → cursor stable for 3+ seconds while typing.
- [ ] **RS-3:** Formatted list item: click → edit → click elsewhere → returns to styled display (not stuck raw).
- [ ] **RS-4:** Formatted → formatted switch in one click after edit.
- [ ] **RS-5:** Table cell → heading in one click after edit.
- [ ] **RS-6:** Raw edit in split left → rendered right reflects change after epoch bump.
- [ ] **RS-7:** Rendered edit does not change `source_epoch` (widget ids stable same session).

### Non-functional

- [ ] No new per-frame O(n) work beyond current rendered render path.
- [ ] `cargo test` / existing markdown editor tests pass.
- [ ] Manual regression matrix RS-1…RS-7 documented in technical doc.

---

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Stale buffer after external epoch bump | Wrong text shown | On epoch bump, clear session buffers or reload from source |
| Viewport culling + line numbers drift | Wrong block at line | BlockRef uses source line numbers; culling only skips render, not identity |
| Undo granularity | Too many/few undo steps | Phase 5 design: one commit = one undo snapshot |
| Table special cases | Regression in Tab nav | Phase 4 dedicated; keep lock_focus and consume_key order |
| Large documents | Buffer duplication | Session buffers only for active + recently active blocks; cap cache size optional |
| Split sync race | Epoch bump during edit | Define policy: external change prompts reload or auto-merge (document in Phase 5) |

---

## Testing Strategy

### Unit tests (`rendered_session.rs`)

- `switch_to` closes previous block with save when dirty
- `switch_to` same block preserves buffer
- `close_active(Discard)` restores source text
- Epoch bump clears or invalidates buffers

### Integration tests

- Extend `markdown::editor` tests for BlockRef id stability across simulated commits

### Manual matrix (minimum)

| ID | Steps |
|----|-------|
| RS-1 | Two headings, edit A, click B |
| RS-2 | Click mid-word in heading, type |
| RS-3 | Bold list item full cycle |
| RS-4 | Two bold items, edit first, click second |
| RS-5 | Table + heading in same doc |
| RS-6 | Split: edit raw, verify rendered |
| RS-7 | Debug log epoch before/after rendered keystroke commit on switch |

---

## Migration / Deprecation

| Current | Disposition |
|---------|-------------|
| `src/markdown/rendered_focus.rs` | Remove after Phase 4 |
| `content_hash` in `push_id` | Keep for culling cache key only; not for widget ids |
| `FormattedItemEditState` | Absorb into `BlockEditState` |
| `focus_loss_should_commit` | Remove |
| `formatted_exit_should_save` | Remove |
| Per-widget defer commit temp keys | Remove |

---

## Related Documentation

| Document | Relevance |
|----------|-----------|
| [`wysiwyg-editor.md`](../../technical/markdown/wysiwyg-editor.md) | Current mode overview — update after Phase 5 |
| [`click-to-edit-formatting.md`](../../technical/markdown/click-to-edit-formatting.md) | Formatted UX — session becomes owner |
| [`table-editing-focus.md`](../../technical/markdown/table-editing-focus.md) | Table defer model — integrate in Phase 4 |
| [`table-cell-focus-navigation.md`](../../technical/markdown/table-cell-focus-navigation.md) | Tab/empty cell — preserve |
| [`galley-cursor-positioning.md`](../../technical/editor/galley-cursor-positioning.md) | Cursor mapping — reuse |
| [`prd-v0.2.5.1-cursor-positioning.md`](./prd-v0.2.5.1-cursor-positioning.md) | Prior cursor fix PRD |
| [`prd-v0.2.0-list-editing-bug.md`](./prd-v0.2.0-list-editing-bug.md) | List index correctness |

---

## Open Questions (resolve in Phase 1 design review)

1. **Click outside document** — Save and close, or keep last block “warm” with focus lost?
2. **Recently closed block buffers** — Evict immediately or keep for quick undo-back?
3. **Code blocks / mermaid** — In scope for session (separate block types) or later?
4. **Split conflict** — If both panes edit simultaneously, epoch + session merge policy?

---

## Appendix A: Current vs proposed (focus switch)

**Current (broken path):**

```
User clicks B while A focused
  → egui defocuses A (first click)
  → A: lost_focus → defer commit → eventually content_hash changes
  → all widget ids remap
  → B: request_focus / restore_switch_focus fights egui
  → cursor lost; formatted: editing flag stuck
```

**Proposed:**

```
User clicks B while A active
  → session.switch_to(B, activation)
      → close A: save buffer, surrender focus (no epoch bump)
      → open B: set pending_activation(cursor)
  → next frame: B TextEdit applies focus + cursor
  → stable ids throughout
```

---

## Appendix B: Suggested task breakdown (for Taskmaster)

1. Phase 0 hotfix — revert formatted defer
2. Add `source_epoch` to tab state + bump sites
3. Create `rendered_session.rs` types + tests
4. Replace `push_id(content_hash)` with `push_id(source_epoch)` for widgets
5. Migrate headings to session API
6. Migrate plain paragraphs and list items
7. Migrate formatted blocks + remove FormattedItemEditState
8. Integrate tables with session
9. Split view epoch sharing
10. Undo on commit + delete `rendered_focus.rs`
11. Technical doc + regression matrix

---

*Created: May 2026*  
*Status: Draft — Path 2 architecture*  
*Authors: AI-assisted; review by maintainers before parse into tasks*
