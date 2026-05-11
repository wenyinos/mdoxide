# Code execution consent dialog

Modal shown the first time the user invokes **Run** on a runnable fenced block in rendered/split markdown preview while consent has not yet been recorded.

## Behaviour

- **Persistence:** `Settings.code_execution_consent_acknowledged` (default `false`) in `config.json`.
- **Preview gate:** `CodeExecutionUi` carries `consent_acknowledged` from settings. `run_button_visible` shows **Run** for shell/Python fences when `allow_*` passes and either the master toggle is on **or** consent is still unset — so users can discover execution without opening Settings first.
- **Defer spawn:** On Run click, if `enable_code_execution && code_execution_consent_acknowledged` both hold in the snapshot, `spawn_run` runs immediately as before. Otherwise the snippet is queued as [`PendingCodeRun`](../../../src/state.rs) via egui temp storage and drained into [`UiState`](../../../src/state.rs) at the start of `render_dialogs`.
- **Settings shortcut:** Turning **Allow code execution** from Off→On in Settings sets consent immediately (`src/ui/settings.rs`) without showing the modal.

## Buttons

| Action | Effect |
|--------|--------|
| **Enable & run** | Master on, consent true, save settings, `spawn_run` for queued payload, attach `RunHandle` under the block id |
| **Just enable** | Master on, consent true, save settings, discard queued run |
| **Cancel** / **Esc** | No settings change, discard queued run |

Initial keyboard focus is on **Cancel**. Strings live under `dialog.code_execution_consent.*` in `locales/en.yaml`.

## References

- `src/app/dialogs.rs` — window rendering and actions
- `src/markdown/widgets.rs` — Run click branch
- `src/markdown/code_execution.rs` — `push_pending_code_execution_consent`, `take_pending_code_execution_consent`, `run_button_visible`
- `src/config/settings.rs` — `code_execution_consent_acknowledged`
