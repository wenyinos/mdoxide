# Sync Scrolling

Ferrite keeps document position aligned between **Raw**, **Rendered**, and **Split** (raw + preview side-by-side) views. Two mechanisms share the same hybrid top/bottom/middle strategy but run at different times.

| Mechanism | When | Setting |
|-----------|------|---------|
| **Mode-toggle sync** | Ctrl+E (Raw ↔ Rendered, not involving Split) | `sync_scroll_enabled` |
| **Split-view live sync** | While scrolling in Split on markdown files | Same setting + optional **2-way** |

Default: `sync_scroll_enabled = false`, `sync_scroll_bidirectional = true` (when sync is turned on).

## User controls

- **Settings → Preview → Sync scroll** — global preference (also ribbon **Sync Scroll** toggle).
- **Split view** — footer under the **semantic minimap** (between editor and preview):
  - **Sync** — master on/off for live split sync (persists to settings).
  - **2-way** — when Sync is on: bidirectional (default) vs raw → rendered only.

i18n keys: `settings.preview.sync_scroll`, `split_sync_scroll`, `split_sync_bidirectional` in `locales/en.yaml`.

## Hybrid algorithm (top / middle / bottom)

Used for **mode toggle** and **split idle snap**:

```text
Within 5px of top     → scroll offset 0 (both panes / target mode)
Within 5px of bottom  → max scroll (or ratio 1.0 on mode toggle)
Otherwise (middle)    → content-based mapping (line + fraction), not %
```

**Why not percentage-only?** Code blocks and Mermaid are much taller in preview than in raw line count; a 50% scroll ratio shows different content. Line-based mapping with **interpolation inside blocks** keeps the same source line visible across modes.

Constants: `SCROLL_BOUNDARY_PX = 5.0` in `src/preview/sync_scroll.rs`.

## Mode-toggle sync (Ctrl+E)

**Entry:** `FerriteApp::handle_toggle_view_mode()` in `src/app/navigation.rs`.

Only runs when `sync_scroll_enabled` and switching between **Raw** and **Rendered** (not Split).

### Tab / app pending fields (`src/state.rs`)

| Field | Purpose |
|-------|---------|
| `pending_scroll_offset` | Pixel offset to apply next frame |
| `pending_scroll_ratio` | 0.0–1.0 (e.g. bottom = 1.0), converted after layout |
| `pending_scroll_to_line` | 1-based line for Raw→Rendered lookup |
| `pending_scroll_anchor` | `(line, fraction)` for split / rendered→raw |
| `rendered_line_mappings` | `(start_line, end_line, rendered_y)` per block |
| `raw_line_height` | From Ferrite editor for raw scroll math |

`Tab::clear_sync_pending_scroll()` clears offset, anchor, ratio, and `pending_scroll_to_line` (not app-level outline `pending_scroll_to_line`).

### Two-frame application (Rendered target)

1. Render, build `rendered_line_mappings`, convert `pending_scroll_to_line` → `pending_scroll_offset` via interpolation.
2. Apply `pending_scroll_offset` on `ScrollArea::vertical_scroll_offset`.

### Interpolation

`find_rendered_y_for_line_interpolated` / `find_source_line_for_rendered_y_interpolated` in `central_panel.rs` — within a block spanning lines 100–200, line 150 maps to ~50% of that block’s rendered height, not only the block top.

## Split-view live sync

**Entry:** `src/app/central_panel.rs` (markdown split layout).

Gated: `settings.sync_scroll_enabled &&` markdown file in split.

### Flow

```text
User scrolls (wheel, scrollbar drag, or keys)
    → note_scroll_activity() detects offset delta (≥2px), not wheel-only
    → master pane: Raw or Rendered (mouse hover + larger delta)
    → store_raw_anchor(line, fraction) or store_preview_y(y)

~120ms idle (SPLIT_SCROLL_IDLE)
    → single programmatic snap to other pane
    → top/bottom: snap to 0 or max scroll
    → middle: source_anchor_to_preview_y / preview_y_to_source_anchor
    → mark_programmatic() (~80ms) to ignore echo

Sync OFF
    → clear tab pending scroll fields each frame
    → reset SyncScrollState on toggle off
    → no idle snap loop
```

### Anchors

- **Raw:** `EditorOutput.scroll_anchor_line` + `scroll_anchor_fraction` from Ferrite `ViewState` (wrap-aware).
- **Preview:** `LineMapping` list from rendered pass; `SyncScrollState::source_anchor_to_preview_y` / `preview_y_to_source_anchor` in `src/preview/sync_scroll.rs`.

### Bidirectional

`sync_scroll_bidirectional` (default `true`):

- **On:** scrolling preview can move raw (see **Split-view scroll delivery** below).
- **Off:** only raw → preview (`tab.pending_scroll_offset`). Preview-only scroll sets `ScrollOrigin::Rendered` and clears any stale raw anchor so idle snap does **not** re-align the preview to an old raw position.

### Split-view scroll delivery

Each pane has its own programmatic scroll channel. Using the wrong field causes visible jumps (e.g. applying raw max scroll to the preview snaps the preview upward on code-block-heavy docs).

| Direction | Region | Mechanism | Applied by |
|-----------|--------|-----------|------------|
| Raw → preview | top / bottom / middle | `tab.pending_scroll_offset` | `MarkdownEditor::pending_scroll_offset` (start of next frame) |
| Preview → raw | top / bottom | `SyncScrollState::set_raw_target(y)` | `EditorWidget::pending_sync_scroll_offset` via `get_animated_raw_offset()` |
| Preview → raw | middle | `tab.pending_scroll_anchor` `(line, fraction)` | `EditorWidget` → `scroll_to_absolute` on anchor line |

**Implementation:** idle snap in `src/app/central_panel.rs` after both panes render; constants and helpers in `src/preview/sync_scroll.rs`.

**Regression:** with **Sync** + **2-way** on, scroll the preview to the bottom — preview stays at bottom and raw follows; raw → preview bottom unchanged.

## Rendered-only mode with sync off

When sync is disabled, rendered mode must **not** apply stale `pending_scroll_*` from a prior split session or mode switch:

- `sync_on` captured before `active_tab_mut()` in `central_panel.rs`.
- Pending offset/ratio/line conversion only when `sync_on`.
- Ribbon or minimap **Sync** off → `clear_sync_pending_scroll()` + `SyncScrollState::reset_on_disable()`.

## Viewport culling and scroll stability

Rendered preview uses **viewport culling** (`ViewportCullingState` in `src/markdown/editor.rs`). Block heights (especially code blocks) are measured as they enter view; `total_height` can change after scroll stops.

**Height fixup** (sync-independent): when `total_height` changes by >1px and the user is not actively scrolling, the next frame preserves scroll **ratio** (`cur/max_old → ratio*max_new`) via temp memory `rendered_height_fixup` so the viewport does not jump when egui keeps a fixed pixel offset. Fixup is suppressed for **200ms** after user scroll input so stopping a wheel/scrollbar drag does not immediately nudge the viewport.

**Active scroll input** is detected via `is_active_scroll_input()` — mouse wheel (`smooth_scroll_delta`) or a decided pointer **drag** (scrollbar thumb). Simple clicks (e.g. task list checkboxes, links) do **not** count as scrolling and do not start the cooldown.

**Inline-only content edits** (task checkbox `[ ]` ↔ `[x]`, etc.) reuse existing culling layout when top-level block `(start_line, end_line)` ranges are unchanged, even if `content_hash` differs. That avoids a bootstrap remeasure frame that would change `total_height` and shift scroll. See [`rendered-view-viewport-culling.md`](./rendered-view-viewport-culling.md) and [`task-list-checkbox.md`](./task-list-checkbox.md).

This is separate from sync; it fixes layout remeasure “nudges” mid-document.

## Settings (`src/config/settings.rs`)

| Field | Default | Description |
|-------|---------|-------------|
| `sync_scroll_enabled` | `false` | Mode toggle + split live sync |
| `sync_scroll_bidirectional` | `true` | Preview → raw when split sync on |

## Key files

| File | Role |
|------|------|
| `src/app/central_panel.rs` | Split layout, idle sync, minimap footer, rendered pending guards |
| `src/app/navigation.rs` | Mode-toggle hybrid sync |
| `src/app/mod.rs` | Ribbon toggle, `sync_scroll_states` cleanup on tab close |
| `src/preview/sync_scroll.rs` | `SyncScrollState`, anchors, boundaries, idle/programmatic |
| `src/editor/minimap.rs` | `show_split_sync_footer`, `SPLIT_SYNC_FOOTER_HEIGHT` |
| `src/editor/widget.rs` | `pending_scroll_anchor`, `pending_sync_scroll_offset`, `EditorOutput` scroll metrics |
| `src/markdown/editor.rs` | Rendered scroll, culling, height fixup |
| `src/state.rs` | Tab pending fields, `clear_sync_pending_scroll()` |

## Future enhancements

1. **Soft follow** — optional lerp toward target during scroll (smoother than idle-only snap).
2. **Mapping warmup** — delay split sync until all block heights measured (heavy Mermaid docs).

## Manual test checklist (v0.3.0)

### Mode toggle (`sync_scroll_enabled` on)

1. Long markdown (100+ lines, code blocks): scroll to ~50%, Ctrl+E Raw↔Rendered — same content region visible.
2. Top / bottom: within a few px of edge, toggle stays at edge.
3. Sync off: toggle resets scroll to top (legacy behavior).

### Split live sync

1. Enable **Sync** in minimap footer; scroll raw with wheel and scrollbar — preview follows after brief idle.
2. **2-way** on: scroll preview — raw follows (middle of document).
3. **2-way** on: scroll preview to **bottom** — preview stays at bottom; raw scrolls to its bottom (no upward jump).
4. **2-way** on: scroll preview to **top** — both panes at top.
5. **2-way** off: preview scroll does not move raw.
6. Top/bottom of either pane — both panes align to edges (raw → preview and preview → raw).
7. Turn **Sync** off while scrolling preview — no post-scroll snap; no ghost jumps from earlier sync state.
8. Code-block-heavy doc, sync off — scroll rendered only; verify no repeated small jumps (height fixup should be minimal).
9. Long task list, scroll to middle — toggle several checkboxes; scroll position must stay fixed (no up/down nudge).

## Related docs

- [View mode persistence](./view-mode-persistence.md) — per-file Raw/Split/Rendered restore
- [Rendered viewport culling](./markdown/rendered-view-viewport-culling.md) — block height cache and performance
