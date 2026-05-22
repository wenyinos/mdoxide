# Code block Run (preview)

The **Run** control on fenced code blocks appears in **Rendered** and **Split** preview when code execution is allowed by settings. It sits in the code-block header alongside Copy and Edit.

## Behaviour

- **Languages:**
  - **Shell:** `bash`, `sh`, `shell`, `zsh`, `pwsh`, `powershell`, `ps1`, `cmd`, `bat`, `batch` — fence language is compared case-insensitively after trim.
  - **Python:** `python`, `python3`, `py`. Detection prefers `python3` on Unix and `python` then `py` on Windows.
  - **Not runnable:** other fences (e.g. `csharp`, `rust`, `js`) do not show **Run** — only the shell and Python families above are implemented.
- **Gating:** Uses `enable_code_execution`, `allow_shell`, `allow_python`, `code_execution_consent_acknowledged`, `code_execution_timeout_secs`, and `code_execution_show_inline_output` — see [Code execution settings](../config/code-execution-settings.md). **Run** can appear while the master toggle is still off until consent is recorded (first modal or enabling from Settings); see [Code execution consent dialog](./code-execution-consent-dialog.md). If only Python blocks show **Run**, enable **Allow shell blocks** under Settings → Editor → Code execution (or use a supported shell fence — `csharp` / `rust` never show **Run** until those runners exist).
- **Working directory:** The open file’s parent folder when the tab has a path; otherwise the process default.

## Copy-paste examples (Windows-friendly)

Use these fenced languages so **Run** appears (with shell/Python allowed in Settings):

**PowerShell** (`powershell`, `pwsh`, or `ps1`):

````markdown
```powershell
Write-Output "Hello from PowerShell"
Get-Location
```
````

**Command Prompt** (`cmd` or `bat`):

````markdown
```cmd
echo Hello from CMD
cd
```
````

**Python** (stdout only if you print or call code that prints):

````markdown
```python
print("Hello from Python")
```
````

PowerShell runs with `-ExecutionPolicy Bypass` for the generated temp `.ps1` so default machine policies are less likely to block one-off snippets.

## Inline output panel

Each Run launches a background worker via [`spawn_run`](../../../src/markdown/code_execution.rs); the UI thread polls a shared [`RunHandle`] (`Arc<Mutex<RunState>>`) once per frame. The widget renders a transient output panel directly below the code block with:

- **Status header:** rotating Braille spinner + "Running" while live, then ✓ "Exit 0", ✗ "Exit N", "Timed out after Ns", "Stopped by user", or "Failed".
- **Elapsed time** (`123ms` / `4.2s` / `1m 12s`).
- **Stdout** and (if non-empty) a separated **stderr** section.
- **ANSI colors** parsed via [`ansi_render`](../../../src/markdown/ansi_render.rs), which wraps `vte::Parser` with a small `Perform` adapter and reuses [`crate::terminal::AnsiColor`] / `TerminalTheme::ferrite_dark|light` so colors match the integrated terminal.
- **Windows line endings:** Many CLIs (including Python’s `print` on Windows) emit **CRLF** (`\r\n`). The parser treats a lone **carriage return** as “clear this line” (for progress-style `\r` overwrites). Before parsing, [`ansi_render::parse`](../../../src/markdown/ansi_render.rs) therefore **normalizes `\r\n` → `\n`**, so the panel shows the same text as **Copy** / raw bytes. Standalone `\r` (not followed by `\n`) is unchanged.
- **Live action:** **Stop** (kills the running child via the cancellation token; the slot becomes Dismiss once the run is no longer `Running`).
- **Post-run actions:** **Copy** (clipboard, ANSI-stripped), **Insert as block** (appends a fenced ` ```output ` block right after the source), **Dismiss** (clears the panel state).

When `code_execution_show_inline_output` is **off**, the panel is hidden and completion (including timeouts and user cancellations) falls back to the legacy toast notification (one-shot per run, routed through `format_completion_toast`).

## Threading & cancellation model

`spawn_run` spawns one worker thread per Run. The worker:

1. Writes a temp script (`ferrite_code_<nanos>.sh|.ps1|.bat|.py`) into `std::env::temp_dir`, with cross-platform interpreter selection.
2. Spawns the child process with `Stdio::piped()` for stdout/stderr; on Windows we set `CREATE_NO_WINDOW` to avoid a console flash.
3. Spawns dedicated **reader threads** for stdout and stderr so blocking `read()` never starves the timeout/`try_wait` loop. Each reader pushes raw bytes into the shared `RunState`. Killing the child closes the pipes, which lets `read()` return `Ok(0)` and the readers join cleanly — no zombie threads.
4. **Polls the cancellation token** (`RunState.cancel: Arc<AtomicBool>`) every ~20 ms inside `wait_child`. The worker holds its own clone of the `Arc<AtomicBool>` so it never has to lock the outer `Mutex<RunState>` to check the flag. When the flag flips it kills the child, joins reader threads, and returns `RunError::Cancelled`.
5. On exit (or timeout, or cancellation), records `RunStatus::Completed { exit_code }` / `TimedOut` / `Cancelled` / `Failed { message }` and calls `egui_ctx.request_repaint()` so the UI reflects the final state immediately.

The cancellation token is exposed to the UI through the small `code_execution::cancel(&RunHandle)` helper. The UI thread calls it from the **Stop** button handler in `EditableCodeBlock::show`. The button is disabled (`add_enabled(false, …)`) once `cancel_requested` is true, so a double-click cannot enqueue a second cancellation. The repaint cadence stays at `request_repaint_after(80 ms)` while a run is in progress, which keeps the spinner rotating and the elapsed-time label fresh.

`RunState.timeout_secs` is captured at spawn time and surfaced in `RunSnapshot` so the panel can render `Timed out after Ns` without re-reading settings. Toast fallback uses the same value via `format_completion_toast`.

ANSI parsing happens in the UI layer (`ansi_render::parse`, including CRLF normalization — see `normalize_crlf` in [`ansi_render.rs`](../../../src/markdown/ansi_render.rs)) so the worker stays focused on transport. This keeps the rendered output consistent with whatever theme is active and avoids duplicating SGR handling already covered by `terminal/handler.rs`.

## Code map

| Area | Location |
|------|----------|
| Runner, gating helpers, `spawn_run`, `cancel`, `RunHandle`, `RunStatus`, cancel token | `src/markdown/code_execution.rs` |
| ANSI parser + renderer (`AnsiLine`, `AnsiSegment`, `render_lines`; `parse` normalizes CRLF) | `src/markdown/ansi_render.rs` |
| Run button, Stop button + inline output panel | `src/markdown/widgets.rs` — `EditableCodeBlock::show`, `render_run_output_panel`, `run_status_label`, `running_spinner_frame` |
| Insert-as-fenced-block handler | `src/markdown/editor.rs` — `render_code_block`, `insert_output_block_after` |
| Settings snapshot into preview | `src/markdown/editor.rs` — `MarkdownEditor::show_rendered_editor` (`code_execution_ctx_id`) |
| Build `CodeExecutionUi` + cwd | `src/app/central_panel.rs` — `CodeExecutionUi::from_settings_with_workdir` |
| Toast drain (fallback path) | `src/app/mod.rs` (after `render_ui`), `drain_code_execution_toasts` |
| Strings | `locales/en.yaml` — `widgets.code_block.run_*` (incl. `run_stop`, `run_status_cancelled`, parameterised `run_status_timed_out`), `settings.editor.code_execution_*` |

## Known limitations (v0.3.0)

Manual regression on Windows passed using [`test_md/test_code_execution.md`](../../../test_md/test_code_execution.md). The following edge cases remain; hardening is scheduled for **v0.3.1** — see [ROADMAP.md](../../../ROADMAP.md) (*Executable code blocks — hardening*).

| Limitation | Impact | Workaround |
|------------|--------|------------|
| **`bash` / `shell` fences on Windows without Git Bash or WSL** | Fallback tries `pwsh` / `powershell` / `cmd` but writes temp files with those extensions while keeping **bash source** — likely confusing spawn/parse errors. | Use `powershell`, `pwsh`, or `cmd` fences on plain Windows; install Git Bash or WSL for `bash`. |
| **`sh` / `zsh` fences** | Only one interpreter is tried (`sh` or `zsh`); no platform fallback chain. | Use `bash` (Unix / Git Bash) or `powershell` / `cmd` (Windows). |
| **Run output keyed by code-block start line** | Inserting or deleting lines **above** a block changes its egui id; in-flight or completed inline output can disappear or attach to the wrong block. | **Dismiss** and **Run** again after structural edits above the fence. |
| **Empty scroll while running** | A slow script with no early stdout shows a blank output area until bytes arrive (status header still shows “Running”). | Cosmetic only; wait for output or use **Stop**. |
| **Copy / Insert as block and stderr** | Clipboard and inserted ` ```output ` blocks concatenate stderr without the on-screen `stderr` heading. | Paste from panel manually if you need labelled sections. |

## Validation

- `cargo test --bin ferrite markdown::ansi_render` covers the SGR parser (plain text, basic colors, 256-color, truecolor, bold/reset, carriage return rewrite, **CRLF vs bare `\r`**, empty input, trailing newline).
- `cargo test --bin ferrite markdown::code_execution` covers language classification (incl. `pwsh`), Run-button visibility flags, status glyph mapping (incl. `Cancelled`), `RunState.timeout_secs` capture, and the idempotent `cancel(&RunHandle)` helper.
- Manual: full checklist in [`test_md/test_code_execution.md`](../../../test_md/test_code_execution.md). Spot checks:
  - Enable Settings → Editor → Code execution; open a markdown file containing shell or python fences; click Run and verify the inline panel reports stdout/stderr with colors and an accurate exit indicator.
  - Long-running snippet (`sleep 60`, `while True: pass`): click **Stop** and confirm the panel transitions to `Stopped by user` within ~100 ms, the spinner stops rotating, and the UI scrolls/interacts normally throughout.
  - Lower the timeout (e.g. 5s), run an infinite loop, and confirm the panel reads `Timed out after 5s` once the worker reaps the child.
  - Toggle `code_execution_show_inline_output` off and repeat both flows; the toast must read `Run failed: Stopped by user` / `Run failed: Timed out after Ns`.
