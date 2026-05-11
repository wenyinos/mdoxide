# Quick note workflow (ephemeral untitled tabs)

Optional editor mode (`Settings.quick_note_workflow`, **Settings → Files**): reduces friction for pathless “scratch” tabs.

## Behavior

- **Close tab / Quit:** Modified untitled tabs (no file path) no longer trigger the unsaved-changes confirmation. Saved files on disk still use the normal prompt when modified.
- **Persistence:** Unsaved text is **not** written to a normal file path; it is still captured by the existing session pipeline (`session.json`, `session.recovery.json`, and per-tab files under the config dir `recovery/`). Requires **Restore previous session on startup** so the next launch reloads those buffers.
- **Display names:** `Tab.untitled_display_name` holds an optional label. The tab strip shows it instead of “Untitled”. **Double-click** an untitled document tab to open the rename dialog. The name is stored in session metadata (`SessionTabState.display_title` without a `*` suffix).
- **Save As:** Assigning a path clears `untitled_display_name` on save (real file name comes from disk).

## Code touchpoints

- `Tab::should_prompt_to_save(&Settings)` — returns `false` for pathless tabs when `quick_note_workflow` is true.
- `AppState::has_unsaved_changes()` — uses the same predicate so `request_exit()` does not block on scratch tabs.
- `AppState::resolve_tab_content` — pathless tabs with `has_unsaved_content == false` restore as empty buffers (supports multiple empty untitled tabs in session).
- `AppState::capture_session_state` — uses `Tab::persisted_session_display_title()` for stable session titles (no trailing `*`).

## Caveats

- This is the same in **dev** (`cargo run`) and **installed** builds: persistence uses your Ferrite [config directory](./config-persistence.md), not the project folder.
- On **clean exit**, the app saves `session.json` **and** per-tab bodies under `recovery/`. Older versions deleted `recovery/` on exit, which broke restoring pathless tabs; that is fixed by keeping recovery content and only removing the crash snapshot file (`session.recovery.json`).
- Closing a modified untitled tab **discards** that buffer from the open session immediately (no dialog). Only **app-wide** session save (debounced / exit) preserves content for the next run.
- Turning **Quick note workflow** off restores the original save prompts for new tabs; existing session data is unchanged.
