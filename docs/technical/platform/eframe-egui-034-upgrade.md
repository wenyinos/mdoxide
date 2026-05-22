# eframe / egui 0.31 → 0.34 Upgrade (complete)

This document captures the **v0.3.0** GUI stack upgrade from `eframe` / `egui` 0.31.1
to 0.34.2, breaking API changes, and migration patterns applied in Ferrite.

See also [eframe-egui-031-upgrade.md](./eframe-egui-031-upgrade.md) for the prior
0.28 → 0.31 migration.

## Why 0.34

- Task **89**: egui 0.34 + HarfRust validation for **v0.3.0**.
- Rust **1.92** (see `rust-toolchain.toml`).
- Windows uses **glow** backend (wgpu-hal conflict with current stack).

## Subtask 89.3 — Panels, ScrollArea, viewport rects

### `screen_rect` → `viewport_rect` / `content_rect`

egui 0.34 deprecates `Context::screen_rect()`. The shim maps it to `content_rect()`,
not the full window. Ferrite uses explicit APIs:

| Use case | API |
|----------|-----|
| Borderless resize, full-window modal overlay | `ctx.viewport_rect()` via [`viewport_window_rect`](../../../src/ui/window.rs) |
| Floating panels constrained to visible window | Same (root viewport) |
| Layout inside panels / after panels allocated | `ui.max_rect()`, `ui.clip_rect()`, or `ctx.content_rect()` |

**Call sites updated (89.3):** `ui/window.rs`, `ui/settings.rs`, `ui/about.rs`,
`ui/search.rs`, `ui/productivity_panel.rs`, `ui/terminal_panel.rs`, `app/mod.rs`
(diagnostic log).

### ScrollArea (0.34 defaults)

- **Edge fade:** New gradient fade at scroll edges (`ScrollStyle.fade`). Disabled
  globally in `FerriteApp::new` (`fade.strength = 0.0`) to preserve pre-upgrade look.
- **content_margin:** Comes from `ScrollStyle`; no per-area overrides added.
- Existing `.auto_shrink([false, false])` on file tree, outline, search, etc. kept.

### Panels

`SidePanel`, `TopBottomPanel`, and `CentralPanel` remain as type aliases over
`Panel` with `.show(ctx, …)`. Ferrite already sets explicit `Frame` / margins on
title bar, ribbon, central panel, and status bar; no panel API migration in 89.3.

## Subtask 89.4 — Menus, popups, tooltips

### Popup API (`Memory::toggle_popup` → `Popup`)

egui 0.34 deprecates `popup_below_widget`, `Memory::toggle_popup`, and
`Memory::close_popup` in favor of [`Popup`](https://docs.rs/egui/latest/egui/containers/struct.Popup.html).

| Pattern | Migration |
|---------|-----------|
| Toggle button + dropdown | `Popup::from_toggle_button_response(&response).id(id).show(\|ui\| …)` |
| State-driven overlay | `Popup::new(…).open_bool(&mut open).show(…)` |
| Picker closes on item click | `.close_behavior(PopupCloseBehavior::CloseOnClick)` |
| Dismiss on outside click only | `CloseOnClickOutside` (recent-files popup) |

**Updated (89.4):** `app/status_bar.rs` (recent files + CSV/encoding pickers),
`markdown/widgets.rs` (rendered link edit popup — local `popup_open` bool to avoid
borrow conflict with `open_bool`).

Menus (`menu_button`, `context_menu`) and explicit `ui.close_menu()` calls in
`file_tree.rs`, `terminal_panel.rs`, `tree_viewer.rs` were reviewed; no API changes
needed (egui 0.34 menu stack handles close-on-click for leaf items).

### Tooltips

`show_tooltip_at_pointer` → `Tooltip::always_open(…, PopupAnchor::Pointer).show(…)`.

**Updated (89.4):** `markdown/csv_viewer.rs` (truncated cell tooltips),
`editor/ferrite/editor.rs` (LSP diagnostic hover).

Most tooltips still use `Response::on_hover_text` (unchanged).

## Subtask 89.5 — Fonts, skrifa/vello_cpu, Galley audit

### Text backend (egui 0.34 defaults)

- Ferrite depends on `egui = "0.34"` with no extra epaint font features; **skrifa** +
  **vello_cpu** are the default layout/raster path (see `Cargo.lock` → `epaint`).
- No `ab_glyph` in the dependency tree for 0.34. HarfRust (`harfrust`) remains a
  separate shaping path for complex-script cursor/selection and cluster layout.

### Font registration (`src/fonts.rs`)

- Unchanged pattern: `FontData::from_static` / `from_owned`, named families
  (Inter, JetBrains Mono, CJK, complex scripts, Phosphor), `ctx.set_fonts` +
  `configure_text_styles`, lazy CJK/complex-script loading, atlas pre-warm via
  `glyph_width`.
- Added [`row_height_for_font`](../../../src/fonts.rs) — use `Fonts::row_height`
  instead of measuring an empty galley (skrifa can return zero height for `""`).

### Galley / cursor conventions (audited)

| API | Index type | Notes |
|-----|------------|--------|
| `Galley::cursor_from_pos` | **Character** (`CCursor::index`) | Used in editor mouse, markdown click-to-cursor |
| `Galley::pos_from_cursor` | **Character** | Wrapped + unwrapped cursor rendering |
| `Galley::layout_from_cursor` | Row within galley | Selection highlights |
| HarfRust `ShapedGlyph::cluster` | **UTF-8 byte** | `shaping.rs`, `column_to_x_offset` / `x_to_column` |

**Complex-script lines:** `LineCache::get_shaped_line` draws one mini-galley per
HarfRust cluster at shaped x-offsets; line height uses `row_height_for_font`.
Cursor/selection for non-wrapped lines use HarfRust advances when
`needs_complex_script_fonts` is true. Full visual parity → **89.6**.

**Editor:** `FIXED_LINE_HEIGHT` (20px) still drives viewport/scroll math; galley
row heights can differ when wrap is enabled (unchanged from 0.31).

### Files touched (89.5)

- `src/fonts.rs` — `row_height_for_font`
- `src/editor/ferrite/shaping.rs` — module docs (skrifa vs HarfRust)
- `src/editor/ferrite/line_cache.rs` — shaped-line height + galley invariant docs
- `src/markdown/cache.rs` — test isolation (`clear_block_height_cache` in unit tests)

## Subtask 89.6 — HarfRust validation (egui 0.34)

### Integration review

- Traced **HarfRust → `ShapedCluster` → mini-galley → paint** under skrifa; removed stale
  `ab_glyph` references from shaping docs.
- **Authoritative metrics:** cursor, selection, mouse, IME (non-wrapped) use HarfRust
  advances via `shape_line_clusters` / `column_to_x_offset` / `x_to_column`.
- **Drawing:** egui lays out each cluster substring; clusters are placed at cumulative
  HarfRust `advance`. `debug_assert!(validate_cluster_byte_ranges)` on cache build.
- **Trace:** if skrifa galley width exceeds advance by >15%, `ferrite::shaping` trace log
  (helps spot ligature/layout drift during manual QA).

### New API / tests

| Item | Purpose |
|------|---------|
| `shape_line_clusters` | Single entry for cache + tests |
| `validate_cluster_byte_ranges` | Invariant checker (full UTF-8 coverage, monotonic ranges) |
| 7 integration tests | Arabic roundtrip, Bengali/Devanagari validate, ligature cluster count, etc. |

```bash
cargo test shaping::    # 32 tests (was 24)
```

### Documented limitations (unchanged scope)

- Word wrap + complex script still uses egui galley only.
- No HarfRust glyph-ID rasterization (substring mini-galleys only).

See [harfrust-text-shaping.md](../editor/harfrust-text-shaping.md) for full pipeline.

## Subtask 89.7 — Mutex / deadlock audit (egui 0.34)

### Scope

Audit cross-thread locking after the 0.34 upgrade: `egui::Mutex` usage in Ferrite,
`std::sync::Mutex` hot spots, and whether any lock is held across `fonts_mut`,
`Context::run`, or `request_repaint` while waiting on another thread.

### `egui::Mutex` in Ferrite

**Zero usages.** Ferrite uses `std::sync::Mutex` (and `Arc`) everywhere. egui 0.34
uses `epaint::mutex::Mutex` internally (loaders, plugins, text-edit undoer); Ferrite
does not need to adopt it unless storing large blobs in egui temp memory (see egui
docs: wrap in `Arc<Mutex<…>>` for cheap clone).

### Inventory (`std::sync::Mutex`)

| Location | Shared state | Threads | Lock pattern |
|----------|--------------|---------|--------------|
| [`single_instance.rs`](../../../src/single_instance.rs) | `Arc<Mutex<Option<egui::Context>>>` | UI + accept thread | Brief lock; bg thread calls `request_repaint()` only |
| [`terminal/mod.rs`](../../../src/terminal/mod.rs) + [`widget.rs`](../../../src/terminal/widget.rs) | `Arc<Mutex<TerminalScreen>>` | **UI only** | PTY reader uses `mpsc` channel; UI locks in `poll()` then releases before widget paint |
| [`markdown/code_execution.rs`](../../../src/markdown/code_execution.rs) | `RunHandle = Arc<Mutex<RunState>>` | UI + worker + pipe readers | Short locks; `AtomicBool` cancel avoids lock in poll loop |
| [`markdown/cache.rs`](../../../src/markdown/cache.rs) | `AST_CACHE`, `BLOCK_HEIGHT_CACHE` statics | UI only | `with_cache()` — lock released between get/insert |
| [`markdown/mermaid/mod.rs`](../../../src/markdown/mermaid/mod.rs) | `DIAGRAM_CACHE` static | UI only | Same `with_cache` pattern |
| [`fonts.rs`](../../../src/fonts.rs) | `CUSTOM_FONT_BYTES`, `LAST_CUSTOM_FONT_ERROR` | UI only | Brief lock in `ttf_bytes_for_font_id_shaping`; no overlap with `fonts_mut` |
| [`diag.rs`](../../../src/diag.rs) | `OnceLock<Mutex<HashSet>>` | any | Dedup logging only |

### Cross-thread paths (channel-first, no egui lock from workers)

| Path | Mechanism | egui touch |
|------|-----------|------------|
| Single-instance IPC | `mpsc` + `repaint_ctx` mutex | `request_repaint()` from accept thread |
| Background file load | `mpsc` (`FileLoadMsg`) | UI `poll_file_load_messages` → `request_repaint()` |
| Code execution | `RunHandle` + pipe drain threads | `request_repaint()` when worker finishes |
| LSP | `mpsc` command/event channels | UI polls events; no `Context` on worker |
| Update check | `mpsc` one-shot | No egui from worker |
| Productivity pipeline | `mpsc` + cancel channel | No egui from worker |
| Terminal PTY read | `mpsc` bytes channel | UI `Terminal::poll()` applies VTE to screen |
| Terminal git/status | `mpsc` status channel | UI reads in `poll()` |

### Lock-order / `fonts_mut` rules verified

1. **No lock held during `fonts_mut` callbacks** — terminal widget calls
   `fonts_mut` for char width *before* acquiring `screen` lock; no other site
   nests `Mutex::lock` inside a `fonts_mut` closure.
2. **`ttf_bytes_for_font_id_shaping`** — `CUSTOM_FONT_BYTES` lock is released
   before `row_height_for_font` / `painter.layout_no_wrap` (which use egui fonts).
3. **No worker thread** calls `fonts_mut`, `set_fonts`, or `Context::run`.
4. **Terminal screen** — `poll_all()` (locks screen) completes before
   `TerminalWidget::show()` (locks again) in the same frame; locks are sequential,
   not nested. `terminal_panel` scrollback probe uses a scoped block and drops
   the guard before `widget.show()`.
5. **`RunHandle`** — pipe threads hold the mutex only for `extend_from_slice`;
   UI snapshots once per frame (`RunSnapshot` clone). Cancel uses `AtomicBool`.

### Findings

- **No deadlock cycles** identified for egui 0.34.
- **No code changes required** for 89.7; existing patterns match egui guidance
  (channels for cross-thread work, brief mutex + `request_repaint` wakeup).
- **Contention note (acceptable):** terminal widget holds `screen` lock for the
  full paint pass; PTY data is buffered on a channel so the reader thread never
  blocks on the screen mutex. Long lock duration may affect frame time under heavy
  output but is not a deadlock risk.
- **Future:** if storing large per-tab state in egui `Memory`, prefer
  `Arc<Mutex<T>>` (egui-recommended) over growing temp maps.

### Verification (89.7)

```bash
cargo check          # OK
cargo test           # 1484 passed; 3 known failures in state::tests (pre-existing, triage 89.8)
cargo test shaping:: # 32/32
```

Manual smoke (already exercised in 89.3–89.6): open file via second instance,
run fenced code block, terminal output while scrolling editor — no hangs observed.

## Subtask 89.8 — MSRV, CI, version bump, final docs

### MSRV & toolchain

| Item | Value |
|------|--------|
| Rust MSRV | **1.92** (`rust-version` in `Cargo.toml`, `rust-toolchain.toml`) |
| egui / eframe | **0.34.2** |
| egui-phosphor | **0.12.0** |
| Ferrite version | **0.3.0** |

### CI / packaging

- **`.github/workflows/ci.yml`** — PR/push: `cargo check` + `cargo test` on Linux and
  Windows using `dtolnay/rust-toolchain@master` (reads `rust-toolchain.toml`).
- **`.github/workflows/release.yml`** — release builds use the same pinned toolchain
  instead of `@stable`.
- **Nix** (`flake.nix`, `nix.yml`) — unchanged; uses nixpkgs Rust for package builds.

### Documentation & release metadata

- [`CHANGELOG.md`](../../../CHANGELOG.md) — `[0.3.0]` release notes.
- [`ROADMAP.md`](../../../ROADMAP.md) — v0.3.0 scope; v0.3.1 items as follow-ups.
- [`docs/ai-context.md`](../../ai-context.md) — egui **0.34.2** line.
- [`v0.3.0-regression-matrix.md`](./v0.3.0-regression-matrix.md) — §8: 0.34 delta + release gates.
- [`docs/building.md`](../../building.md) — prerequisites **Rust 1.92+**.

### Test fixes (89.8)

Three `state::tests` failures triaged: default **Quick note workflow** (`Settings::default`)
suppresses save prompts for modified untitled tabs. Tests that assert classic save-on-quit
behaviour now set `quick_note_workflow = false` explicitly.

### `App::update` deferral

Ferrite still implements deprecated `eframe::App::update`; `App::ui` remains an empty stub
until a dedicated lifecycle migration (out of Task 89 scope).

### Verification (89.8)

```bash
cargo check
cargo test           # 1487 passed; 0 failed
cargo test shaping:: # 32/32
```

## Deferred (follow-up releases)

| Topic | Target |
|-------|--------|
| `App::update` → `App::ui` / `App::logic` lifecycle | Separate task |
| Wayland / macOS Sonoma keyboard verification [#106](https://github.com/OlaProeis/Ferrite/issues/106) / [#111](https://github.com/OlaProeis/Ferrite/issues/111) | Community / CI hardware |
| macOS Developer ID signing [#130](https://github.com/OlaProeis/Ferrite/issues/130) | Blocked on Apple Developer Program enrollment |

## Verification

```bash
cargo check
cargo test
cargo run   # smoke: Latin + CJK/complex-script editor, rendered markdown click-to-cursor
```

**89.3 smoke:** resize, sidebars, settings/about overlays, scroll file tree.

**89.5 smoke:** monospace/proportional editor, Arabic/Bengali sample lines (cursor +
selection), wrapped markdown block click positions.
