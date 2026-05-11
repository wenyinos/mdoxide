# Code execution settings (markdown)

Markdown code-block execution will be gated by persisted user preferences until a runner lands in-preview. Settings are stored in `config.json` alongside the rest of [`Settings`](./settings-config.md).

## Fields

| Field | Type | Default | Notes |
|-------|------|---------|--------|
| `enable_code_execution` | `bool` | `false` | Master switch; subprocesses stay off unless the user opts in |
| `allow_shell` | `bool` | `true` | Shell-tagged fenced blocks (e.g. bash, pwsh); ignored when master is off |
| `allow_python` | `bool` | `true` | `python`-tagged fenced blocks; ignored when master is off |
| `code_execution_timeout_secs` | `u32` | `30` | Clamped **5–300** in [`Settings::sanitize`](./settings-config.md) on load. Surfaced verbatim by the inline output panel as `Timed out after Ns` and used by the worker's hard-kill loop in `wait_child` |
| `code_execution_show_inline_output` | `bool` | `true` | Render captured stdout/stderr (with ANSI colors) in a transient panel below the block; when `false`, fall back to the legacy toast-only completion notification (timeout/cancellation toasts route through the same `format_completion_toast`) |
| `code_execution_consent_acknowledged` | `bool` | `false` | Persists that the user accepted code-execution risk (via the first-run consent modal when clicking **Run** in preview, or by enabling execution from Settings). While `false`, preview still shows **Run** for allowed fenced languages even if the master toggle is off; after acknowledgement without opting in, **Run** stays hidden until the master toggle is on |

The Editor settings panel turns `allow_shell` and `allow_python` back **on** the first time the user enables `enable_code_execution` (helps match “defaults on when opting in”). Users can disable either language afterward. That same transition (**master toggle** Off→On) also sets `code_execution_consent_acknowledged = true`, so users who opt in only from Settings never see the modal.

Modal UX when triggering runs from preview is documented in [Code execution consent dialog](../markdown/code-execution-consent-dialog.md).

## UI & i18n

- **Place:** Settings → Editor → Code execution section (below the toggle grid; before language-server overrides).
- **Warning:** One-line advisory using the theme warning text color (`warn_fg`): running code is risky and unsandboxed.
- **Strings:** `settings.editor.code_execution_*` in `locales/en.yaml`.

## Implementation references

- `src/config/settings.rs` — struct fields, `Default`, `sanitize` bounds (`MIN_CODE_EXECUTION_TIMEOUT_SECS` / `MAX_CODE_EXECUTION_TIMEOUT_SECS`)
- `src/ui/settings.rs` — `show_editor_section` subgroup
- `src/markdown/code_execution.rs` — runner, preview snapshot (`CodeExecutionUi`), toast queue, cancellation token (`RunState.cancel`), `cancel(&RunHandle)` helper
- `src/markdown/widgets.rs` — `EditableCodeBlock` Run button, **Stop** button in `render_run_output_panel`
- `src/app/dialogs.rs` — consent modal (`dialog.code_execution_consent.*`), drains pending Run payloads queued from preview
