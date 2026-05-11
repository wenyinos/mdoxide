# Code-block Run cancellation & timeout

User cancellation and hard-timeout enforcement for the [Code block Run](./code-block-run.md) feature. Layered on top of the background-runner introduced in v0.3 (Task #66) and gated by [Code execution settings](../config/code-execution-settings.md).

## Behaviour

- **Stop button.** While a run is `RunStatus::Running`, the inline output panel shows a `⏹ Stop` button in the same header slot that becomes `Dismiss` once the run terminates. One slot, two states — never both at once.
- **Cancellation latency.** The worker polls the cancel token every ~20 ms inside `wait_child`. From a user click to `RunStatus::Cancelled` is `≤ 100 ms` in practice (kill + reaper + reader-thread join).
- **Timeout strings.** `RunStatus::TimedOut` renders as `Timed out after Ns`, where `N` is the configured `code_execution_timeout_secs` captured at spawn time. Cancellation renders as `Stopped by user` (neutral grey).
- **Toast fallback.** When `code_execution_show_inline_output` is **off**, completion (including timeouts and cancellations) is routed through `format_completion_toast`: `Run failed: Stopped by user` / `Run failed: Timed out after Ns`.
- **Disabled-state guard.** Once cancellation is requested but the worker has not yet observed the flag, the Stop button is disabled (`add_enabled(false, …)`) so a double-click cannot enqueue a second cancellation.
- **Spinner liveness.** A 10-frame Braille spinner (`⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`) is selected from `snap.elapsed.as_millis() / 80`, so the visible glyph keeps rotating each frame without storing animation state in egui memory.

## Cancellation token

`RunState` carries an `Arc<AtomicBool>` cancel flag and the worker holds **its own clone of the `Arc`**, separate from the outer `Arc<Mutex<RunState>>`. That way the polling loop never has to lock the mutex to check the flag — only the UI thread takes the lock once when calling `cancel(&handle)` to flip the bit.

```rust
pub fn cancel(handle: &RunHandle) {
    if let Ok(state) = handle.lock() {
        state.cancel.store(true, Ordering::Relaxed);
    }
}
```

`Ordering::Relaxed` is sufficient: the worker only needs eventual visibility, the kill/wait sequence dominates latency, and there is no other shared state being published alongside the flag.

## Reader-thread shutdown

The `wait_child` polling loop drives one of three exits — child completed, timeout, or cancellation — and each path follows the same shape:

1. `child.kill()` (no-op if the child already exited).
2. `child.wait()` to reap the zombie.
3. Join `stdout_thread` and `stderr_thread`.

Killing the child closes its stdio pipes, which lets the reader threads' blocking `read()` return `Ok(0)` and exit their `drain_pipe` loop cleanly. The joins are therefore non-blocking in steady state and there are **no zombie reader threads** after a cancellation.

## UI flow

`EditableCodeBlock::show` snapshots `RunState` once per frame into `RunSnapshot { status, stdout, stderr, elapsed, timeout_secs, cancel_requested }`. The `OutputPanelResponse` returned by `render_run_output_panel` carries a new `stop` flag; when set, the outer `show()` calls `code_execution::cancel(&handle)` and requests an immediate repaint. The existing `request_repaint_after(80 ms)` cadence keeps the spinner rotating between events.

`run_status_label` reads `snap.timeout_secs` so `Timed out after Ns` is built without re-reading settings, which keeps the renderer self-contained when settings change mid-run.

## Code map

| Concern | Location |
|---------|----------|
| `RunStatus::Cancelled`, `RunState.cancel`, `RunState.timeout_secs`, `cancel(&RunHandle)`, `wait_child` polling | `src/markdown/code_execution.rs` |
| `RunSnapshot.{cancel_requested, timeout_secs}`, `OutputPanelResponse.stop`, Stop-button render, Braille spinner, `run_status_label`, `format_completion_toast` | `src/markdown/widgets.rs` — `render_run_output_panel`, `running_spinner_frame` |
| Toast strings + Stop button labels | `locales/en.yaml` — `widgets.code_block.run_stop`, `run_stop_tooltip`, `run_status_cancelled`, parameterised `run_status_timed_out` |

## Validation

- `cargo test --bin ferrite markdown::code_execution` — covers `Cancelled` glyph, idempotent `cancel(&RunHandle)`, `RunState.timeout_secs` capture, terminal-state assertions.
- Manual smoke tests:
  - Long-running snippet (`sleep 60`, `while True: pass`) — Stop transitions to `Stopped by user` within ~100 ms; UI scrolls and interacts normally throughout the run.
  - Lower the timeout (e.g. 5s), run an infinite loop — panel reads `Timed out after 5s` once the worker reaps the child.
  - Toggle `code_execution_show_inline_output` off and repeat both flows — toast wording mirrors the panel labels.
