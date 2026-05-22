# Session Handover

## Environment

- **Project:** Ferrite (markdown editor, Rust + egui)
- **Tech Stack:** Rust 2021 (toolchain **1.92** via `rust-toolchain.toml`), **egui/eframe 0.34.2** (glow backend on Windows)
- **Context file:** Always read [`docs/ai-context.md`](./ai-context.md) first — project rules, architecture, conventions.
- **Branch:** working tree on `master` tag tasks (session recovery hardening — task 106 in progress)

## Handover Rules

- **SCOPE:** Finish **task 106** — work subtasks **106.4 → 106.5 → 106.6 → 106.7** in order, all in one session.
- 106.1, 106.2, 106.3 are **already done**. Read the "What's already in place" section before writing any code.
- Run `cargo check` after every change. Do **not** rely on full `cargo test` — pre-existing test compile errors on this branch (unrelated modules) block test execution.
- Document by feature in `docs/technical/files/` when behaviour changes (update [`session-persistence.md`](./technical/files/session-persistence.md) **after 106 is fully done**, not per-subtask).
- **Related docs:** [`session-persistence.md`](./technical/files/session-persistence.md), [`auto-save.md`](./technical/files/auto-save.md)

---

## Current Task: 106 — Harden session recovery with document identity and UI safeguards

**Priority:** high | **Complexity:** ~8 | **Dependencies:** none

**Status:** in-progress (3/7 subtasks done).

**Goal:** Recovery and autosave only apply when the recovered file's `(path, hash-of-disk-content-at-load-time)` matches the tab's current on-disk identity. When the disk content has diverged from the recovered buffer, surface a non-blocking banner so the user picks `Keep Recovered` vs `Reload from Disk` instead of silently overwriting either side.

### Subtasks

| ID | Title | Status |
|----|-------|--------|
| 106.1 | Extend `RecoveryContent` with `path`, `original_content_hash`, `schema_version` + serde defaults | **done** |
| 106.2 | Populate identity fields in `save_recovery_content` (`Tab::disk_content_hash`) | **done** |
| 106.3 | Backward-compatible `load_recovery_content` / `load_all_recovery_content` + migration hook | **done** |
| **106.4** | Strengthen `resolve_tab_content` — path + hash checks; `session_recovery_identity_mismatch` diag | **next** |
| **106.5** | UI conflict banner: `Keep Recovered` / `Reload from Disk` (non-blocking, `AppState`) | pending |
| **106.6** | Same identity semantics for autosave + pruning | pending |
| **106.7** | Regression tests: tab_id collision, hash mismatch, legacy files, original bleeding repro | pending |

---

## What's already in place (do not redo)

### `RecoveryContent` (`src/config/session.rs`)

```rust
pub const RECOVERY_CONTENT_SCHEMA_VERSION: u32 = 1;

pub struct RecoveryContent {
    pub tab_id: usize,
    pub content: String,
    pub saved_at: u64,
    #[serde(default)] pub path: Option<PathBuf>,
    #[serde(default)] pub original_content_hash: Option<u64>,
    #[serde(default = "RecoveryContent::default_schema_version")]
    pub schema_version: u32,
}

impl RecoveryContent {
    pub fn default_schema_version() -> u32 { RECOVERY_CONTENT_SCHEMA_VERSION }
    pub fn new(tab_id, content) -> Self                 // back-compat (no identity)
    pub fn new_with_identity(tab_id, content, path: Option<PathBuf>,
                             original_content_hash: Option<u64>) -> Self
}
```

### `Tab::disk_content_hash()` (`src/state.rs`)

```rust
/// Hash of the disk content this tab was last loaded from / saved to.
/// Independent of the in-memory buffer — designed for identity checks.
/// Same DefaultHasher algorithm as `crate::config::session::hash_content`,
/// so values from the two are directly comparable.
pub fn disk_content_hash(&self) -> Option<u64>
```

### `save_recovery_content` (helper + state caller)

`crate::config::save_recovery_content(tab_id, content, path: Option<&Path>, hash: Option<u64>)` — writes the full identity-bearing struct.

`AppState::save_recovery_content` already threads `tab.path.as_deref()` and `tab.disk_content_hash()` through. Nothing to change there.

### Loaders + migration

- `pub fn load_recovery_content(tab_id) -> Option<RecoveryContent>` (returns the full struct, **not just `String`**).
- `fn load_all_recovery_content() -> HashMap<usize, RecoveryContent>`.
- `fn migrate_recovery_content(rc) -> Option<RecoveryContent>` — passthrough for current schema, stamp legacy `< current` to current, **reject `> current`**.
- `fn parse_recovery_content_json(json) -> Option<RecoveryContent>` — central JSON entry point used by both loaders.
- `SessionRestoreResult.recovered_content: HashMap<usize, RecoveryContent>` (was `String`).

### `resolve_tab_content` consumer (`src/state.rs`, ~line 5043)

The function already extracts `recovered.content.clone()` — only the `has_unsaved_content` guard exists today. **This is where 106.4 plugs in.**

### Tests already added

- `src/config/session.rs` — round-trip with identity, legacy JSON without new fields → defaults, migration matrix (current / older / future / malformed).
- `src/state.rs` — `Tab::disk_content_hash` for untitled, path-backed, in-memory-edits, after `mark_saved`.

---

## 106.4 — `resolve_tab_content` identity gating (start here)

**Files:** `src/state.rs` (`resolve_tab_content`, ~line 5034) and `src/diag.rs` (event keys are `&'static str`, no enum needed).

**Diag API** is `crate::diag::event(key: &'static str, message: impl AsRef<str>)` — already used in this same function for `"session_recovery_stale_ignored"`. New event key:

```rust
"session_recovery_identity_mismatch"
```

**Logic to add (inside the existing `if let Some(recovered) = … { if session_tab.has_unsaved_content { … } }` branch):**

1. **Path mismatch** — `recovered.path != session_tab.path` → log + emit `session_recovery_identity_mismatch` + fall through to disk load (do **not** return `Recovered`).
2. **Hash mismatch (path-backed tabs only)** — when `session_tab.path` exists on disk and `recovered.original_content_hash` is `Some(want)`:
   - Compute `hash_content(disk)` (function already in `session.rs`).
   - If `hash != want` → log + emit `session_recovery_identity_mismatch` + fall through.
3. **Legacy fallback** — if `recovered.path.is_none() && recovered.original_content_hash.is_none()` → behave as today (apply when `has_unsaved_content`). Document this clearly with a comment because it preserves compatibility with pre-task-106 recovery files. The stale-detection still relies on `session_tab.has_unsaved_content` + `prune_recovery_dir`.
4. **Untitled tabs** — `session_tab.path.is_none()` → require `recovered.path.is_none()` for a match (path equality already covers this).
5. **All checks pass** → existing return `ResolvedContent::Recovered(recovered.content.clone())`.

**Important detail:** if 106.5 needs to surface a banner when content differs from disk **but identity matches** (path-backed tab where the user's recovery has new edits the disk does not), 106.4 should expose that information back to the caller. Two reasonable options:

- (a) Add a `ResolvedContent::RecoveredWithDiskDivergence { content, on_disk_content }` variant, OR
- (b) Stash conflict state on `AppState.recovery_conflicts: HashMap<usize, RecoveryConflict>` from inside `resolve_tab_content`. **Problem:** `resolve_tab_content` takes `&self`, not `&mut self`. So **prefer (a)** — extend `ResolvedContent` and let the caller (`restore_from_session_result`) populate the conflict map.

Look at `ResolvedContent` enum in `state.rs` — it currently has `Recovered(String)` and `FromDisk { … }`. Adding a new variant is the cleanest path.

**Diag event format** (mirror existing):
```rust
crate::diag::event(
    "session_recovery_identity_mismatch",
    format!(
        "tab_id={} title={} session_path={:?} recovered_path={:?} \
         expected_hash={:?} disk_hash={:?}",
        session_tab.tab_id, session_tab.display_title,
        session_tab.path, recovered.path,
        recovered.original_content_hash, current_disk_hash,
    ),
);
```

**Tests to add (state.rs):**
- Path mismatch → `Recovered` not produced.
- Path match + hash match → `Recovered`.
- Path match + hash mismatch → identity rejection.
- Legacy file (path = None, hash = None) on path-backed session_tab → still applied (back-compat) when `has_unsaved_content`.
- Legacy file on untitled session_tab → applied (paths both None).

These tests can construct `RecoveryContent` directly and inject into `SessionRestoreResult.recovered_content` — no file IO needed.

---

## 106.5 — Non-blocking conflict banner

**Files:** `src/state.rs` (new `RecoveryConflict` struct + `AppState.recovery_conflicts`), `src/app/central_panel.rs` (banner UI).

```rust
pub struct RecoveryConflict {
    pub recovered_content: String,
    pub on_disk_content: String,
}
```

Stored as `AppState.recovery_conflicts: HashMap<usize /* tab_id */, RecoveryConflict>`.

**When populated:** in `restore_from_session_result`, when `resolve_tab_content` returns the new `RecoveredWithDiskDivergence` variant. Apply the recovered content to `tab.content`, mark the tab as modified (it already will be because `content != original_content`), and store the conflict.

**Banner rendering** (in `central_panel.rs`, before the editor widget for the active tab):

- Read `state.recovery_conflicts.get(&active_tab.id)`.
- Show a non-modal `egui::Frame` strip with text: `"Recovered content differs from this file on disk."` and two buttons.
- **Keep Recovered:** `state.recovery_conflicts.remove(&tab.id)`. Buffer stays. Modified flag stays (user can save manually). Done.
- **Reload from Disk:** read disk with same encoding detection used elsewhere; replace `tab.content` and call `tab.mark_saved()` (or equivalent — see existing reload path in `app/file_ops.rs`); then `state.recovery_conflicts.remove(&tab.id)`.
- Banner must be **non-blocking** — clicking elsewhere does not dismiss it; only the two buttons clear it. Editing is allowed while it's visible.
- Closing a tab must `recovery_conflicts.remove(&tab.id)` (look for the existing tab-close path; there's typically a tab cleanup in `app/file_ops.rs` or similar).

**Tests:** state-level — populate a conflict, call the action handlers (extracted as small `pub fn` if they aren't already), assert post-conditions on the buffer + map. Pure UI tests are skipped.

**i18n:** add keys to `locales/en.yaml` (project convention — see `ai-context.md`). Search existing yaml for "Recovered" / "Reload" so we match the tone.

---

## 106.6 — Autosave identity hardening

**Files:** `src/config/session.rs` (`AutoSaveMetadata`, `save_auto_save_content`, `check_auto_save_recovery`), `src/app/mod.rs` (`fn check_auto_save_recovery` at ~line 1229, `save_auto_save_content` call at ~line 1192).

`AutoSaveMetadata` already has `tab_id`, `original_path`, `content_hash`. Note: `content_hash` is the hash of the **autosaved buffer**, not of the disk file. To preserve disk identity we need a sibling field.

**Add field** (with serde default for back-compat):

```rust
pub struct AutoSaveMetadata {
    pub tab_id: usize,
    pub original_path: Option<PathBuf>,
    pub saved_at: u64,
    pub content_hash: u64,
    /// Hash of the on-disk content the tab was loaded from at the
    /// moment this autosave was written (added in task 106). `None`
    /// for legacy autosave files; `None` for untitled tabs.
    #[serde(default)]
    pub disk_content_hash: Option<u64>,
}
```

**Save path** (`src/app/mod.rs::save_auto_save_content` call ~line 1192): thread `tab.disk_content_hash()` through. The helper signature in `session.rs` needs to accept the new value too:

```rust
pub fn save_auto_save_content(
    tab_id: usize,
    file_path: Option<&PathBuf>,
    content: &str,
    disk_content_hash: Option<u64>,    // new
) -> bool
```

**Load / apply path** (`src/app/mod.rs::check_auto_save_recovery` ~line 1229): the metadata-vs-tab check is currently mostly id/path-based. Extend to:

- If `metadata.original_path != tab.path` → reject (already implicitly checked, but be explicit).
- If `tab.path` exists on disk AND `metadata.disk_content_hash` is `Some(want)` AND `hash_content(disk_now) != want` → reject + emit `session_recovery_identity_mismatch` (re-use the same event key — the message can include `source=autosave`).
- Otherwise apply, optionally surfacing the same `RecoveryConflict` if the autosave content differs from disk (mirrors 106.5).

**Pruning:** `clear_all_auto_saves` / `delete_auto_save` already exist. Add a function that lists autosave files and deletes the ones whose identity check fails OR whose `tab_id` is not in the current valid set (the equivalent of `prune_recovery_dir` for autosave). Call it once after restore is complete, like `prune_stale_recovery_files` in `state.rs`.

**Original bleeding repro to defeat (from acceptance criteria):** untitled buffer named `asdasd` ends up at autosave file for `tab_id=10` from a previous session that used the same id for `task_50_table_inline_formatting.md`. With the new rules, the metadata's `original_path != tab.path` (`None` vs `Some(.../task_50_…)`) → autosave rejected even if cleanup is bypassed.

---

## 106.7 — Regression tests

`src/config/session.rs` and/or `src/state.rs`. All structural — no real file IO required (build `SessionRestoreResult` and `AutoSaveMetadata` directly):

1. **tab_id collision, different paths** — `RecoveryContent { tab_id: 10, path: Some("/a.md"), … }` + `SessionTabState { tab_id: 10, path: Some("/b.md"), … }` → reject + diag event.
2. **path match, hash mismatch** — disk hash differs from `recovered.original_content_hash` → reject.
3. **Legacy file (no identity) + path-backed session_tab + has_unsaved_content** → applied (back-compat path documented in 106.4).
4. **Original bleeding repro** — untitled `asdasd` recovery for `tab_id=10` against a session_tab that's now path-backed `task_50_table_inline_formatting.md` → rejected even if `prune_recovery_dir` is skipped.
5. **Autosave** counterpart of #1 and #2 against `check_auto_save_recovery`.
6. **Round-trip** of new `disk_content_hash` field in `AutoSaveMetadata` (with-and-without).

Acceptance: the cross-tab bleed scenarios cannot succeed even when pruning is bypassed.

---

## Key files (task 106)

| File | Purpose |
|------|---------|
| `src/config/session.rs` | `RecoveryContent`, `AutoSaveMetadata`, save/load + migration, `hash_content`, recovery dir pruning |
| `src/state.rs` | `Tab::disk_content_hash`, `resolve_tab_content`, `restore_from_session_result`, `AppState.recovery_conflicts` (new) |
| `src/app/mod.rs` | `save_auto_save_content` call, `check_auto_save_recovery` (apply path) |
| `src/app/file_ops.rs` | Existing reload-from-disk pattern (reference for 106.5 banner action) |
| `src/app/central_panel.rs` | Conflict banner UI |
| `src/diag.rs` | `crate::diag::event("session_recovery_identity_mismatch", …)` |
| `locales/en.yaml` | Banner i18n strings |
| `docs/technical/files/session-persistence.md` | Update once 106 is fully done |

---

## Verification baseline

```powershell
cargo check
# Skip full `cargo test` — pre-existing unrelated test compile errors on this
# branch (history.rs::TextBuffer, code_execution.rs::CHECK/X, mermaid imports,
# productivity_panel::Task::to_markdown). Fix only if your changes touch them.
# After each subtask:
#   - cargo check → clean
#   - ReadLints on touched files → no errors
```

- **Manual smoke test (post-106.5/106.6):** kill app with unsaved edits → restart → recovery applied to correct tab only; if disk diverged, banner shown with two actions and editing not blocked.

---

## Model selection

| Complexity | Recommendation |
|------------|----------------|
| **~8** (4 subtasks remaining) | **Strong reasoning model.** 106.4 and 106.5 require careful state-flow reasoning across `resolve_tab_content` (immutable-self), `restore_from_session_result` (mutable), and the central_panel render loop. 106.6 mirrors the same logic on autosave with subtler back-compat. 106.7 must include the **original bleed repro** so it can never regress. |

---

## Task Master

```bash
task-master show 106 --json
task-master set-status --id=106.4 --status=in-progress
# After each subtask passes cargo check + lints:
task-master set-status --id=106.4 --status=done
task-master set-status --id=106.5 --status=in-progress
# … and so on through 106.5, 106.6, 106.7.
# Finally:
task-master set-status --id=106 --status=done
```

> Use the **CLI** (`task-master`) — MCP tooling is currently unreachable on this host. Quote comma-separated IDs (`--id="106,106.4"`) when needed: PowerShell otherwise splits the argument.
