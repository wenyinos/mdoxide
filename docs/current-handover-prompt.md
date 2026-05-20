# Session Handover

## Environment

- **Project:** Ferrite (markdown editor, Rust + egui)
- **Tech Stack:** Rust 2021 (toolchain **1.92** via `rust-toolchain.toml`), **egui/eframe 0.34.2** (glow backend on Windows)
- **Context file:** Always read [`docs/ai-context.md`](./ai-context.md) first — project rules, architecture, conventions.
- **Branch:** `egui-0.34-upgrade` (not `master`)
- **Parent task:** Task **89** — egui 0.34 upgrade + HarfRust validation
- **Next subtask:** **89.7** — `egui::Mutex` / lock-order deadlock audit

## Handover Rules

- **SCOPE:** Start **89.7** only. Do **not** re-do 89.1–89.6 unless fixing a regression found in testing.
- **NO HISTORY:** Do not carry forward unrelated work (Mermaid flowchart spacing, etc.).
- Run `cargo check` / `cargo test` after changes.
- Use Context7 MCP for egui 0.34 API docs when unsure.
- Keep deprecated **`App::update`** with empty **`App::ui`** stub until a dedicated follow-up (not in task 89).
- **Docs:** §89.7 notes go in [`eframe-egui-034-upgrade.md`](./technical/platform/eframe-egui-034-upgrade.md). **Final** polish (`ai-context.md`, `ROADMAP`, `CHANGELOG`, version bump, full regression matrix sign-off) → **89.8**.

---

## Task 89 — Subtask map

| ID | Title | Status |
|----|--------|--------|
| **89.1** | Deps: egui/eframe 0.34.2, phosphor 0.12, rust-toolchain, glow (Windows) | done (`ad60ac1`) |
| **89.2** | Compile fixes + `App::ui` stub; keep `App::update` | done (`3a9da15`) |
| **89.3** | Panels, ScrollArea, `screen_rect` → viewport rects | done (uncommitted on branch) |
| **89.4** | Menus, popups (`Popup` API), tooltips | done (uncommitted on branch) |
| **89.5** | Font/text layout, skrifa/vello_cpu, Galley audit | done |
| **89.6** | HarfRust shaping validation + tests | done |
| **89.7** | `egui::Mutex` deadlock audit | **next** |
| **89.8** | MSRV/CI, regression matrix, version bump, **final docs** | pending |

```bash
task-master show 89.7 --json
task-master set-status -i 89.7 -s in-progress
```

---

## Completed work (89.5–89.6) — summary for 89.8 doc pass

### 89.5 — Fonts / Galley / skrifa

- Confirmed egui 0.34 default text backend (skrifa + vello_cpu); no Ferrite feature flags.
- Added `fonts::row_height_for_font` — avoid empty-galley height 0 under skrifa.
- Galley audit: `CCursor` = character index; HarfRust `cluster` = UTF-8 byte index.
- §89.5 in [`eframe-egui-034-upgrade.md`](./technical/platform/eframe-egui-034-upgrade.md).

### 89.6 — HarfRust validation

- **Bug fix:** `group_clusters` now sorts by `byte_start` + normalizes full UTF-8 coverage (Arabic ligature `لا` cursor).
- API: `shape_line_clusters`, `validate_cluster_byte_ranges`.
- **32** shaping unit tests (`cargo test shaping::`).
- Updated [`harfrust-text-shaping.md`](./technical/editor/harfrust-text-shaping.md) (removed ab_glyph wording).
- §89.6 in `eframe-egui-034-upgrade.md`.
- **Known limit:** word wrap + complex script still uses egui galley only (not HarfRust).

### Manual testing

- **User completed** manual checks for 89.3–89.6 (including complex scripts, fonts, popups, shaping samples). No blocking regressions reported.

---

## Current Task: 89.7 — Mutex / deadlock audit

1. Search for `egui::Mutex` (may be **zero** usages — audit `std::sync::Mutex` + `egui::Context` sharing too).
2. Hot spots: `single_instance.rs` (`Arc<Mutex<Option<egui::Context>>>`), terminal `Arc<Mutex<TerminalScreen>>`, code-run `RunHandle`, global caches (`markdown/cache.rs`, mermaid), `fonts.rs` statics, background file load / workers if enabled.
3. Verify no lock held across `fonts_mut` / `Context::run` / repaint while waiting on another thread.
4. Document findings + any fixes in `eframe-egui-034-upgrade.md` §89.7.

**Out of scope here:** version bump, CHANGELOG, full regression matrix (**89.8**).

---

## Verification baseline

```bash
cargo check
cargo test
cargo test shaping::
```

- **Expected:** `cargo check` OK; **~1484** tests pass, **3 failures** in `state::tests` (see below).
- Shaping: **32/32** pass.

### Known issues (address in 89.7 or 89.8)

- **`cargo test`:** 3 failures in `state::tests` (`test_appstate_has_unsaved_changes`, `test_appstate_quit_with_mixed_tabs`, `test_appstate_request_exit_with_changes`). Triage/fix before **89.8** release closure.
- **Uncommitted:** 89.3–89.6 + docs on `egui-0.34-upgrade` (only `ad60ac1`, `3a9da15` committed). Consider commit(s) in **89.8**.

---

## Documentation debt (89.8)

| Item | When |
|------|------|
| [`eframe-egui-034-upgrade.md`](./technical/platform/eframe-egui-034-upgrade.md) | §89.7–89.8; finalize |
| `docs/ai-context.md` | egui version line still says 0.31 |
| `CHANGELOG`, `ROADMAP`, version **0.4.0** | **89.8** |
| [`v0.3.0-regression-matrix.md`](./technical/platform/v0.3.0-regression-matrix.md) | Re-run / adapt for 0.4.0 in **89.8** |
| `App::update` → `App::ui` migration | Separate task (not 89) |

---

## Key references

| Doc | Purpose |
|-----|---------|
| [`eframe-egui-034-upgrade.md`](./technical/platform/eframe-egui-034-upgrade.md) | Living 0.31→0.34 migration log (89.3–89.6 written) |
| [`harfrust-text-shaping.md`](./technical/editor/harfrust-text-shaping.md) | Shaping pipeline + known limits |
| [`eframe-egui-031-upgrade.md`](./technical/platform/eframe-egui-031-upgrade.md) | Prior 0.28→0.31 patterns |
| [`single-instance.md`](./technical/platform/single-instance.md) | `Mutex` + `egui::Context` repaint thread |
| `test_md/test_complex_scripts.md` | Manual script samples (already exercised) |

---

## Task Master

```bash
task-master show 89,89.7 --json
task-master set-status -i 89.7 -s in-progress
# When 89.7 pass criteria met:
task-master set-status -i 89.7 -s done
```
