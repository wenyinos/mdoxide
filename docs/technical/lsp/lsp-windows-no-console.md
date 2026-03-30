# LSP: Windows — No Console Window (Task 34)

## Problem

On Windows, spawning console-based language servers (for example rust-analyzer, pyright) with `std::process::Command` can briefly show or leave visible `cmd.exe` console windows. Ferrite is a GUI app; LSP children should run without flashing consoles.

## Solution

In `src/lsp/manager.rs`, `spawn_server()` sets the Win32 process creation flag **`CREATE_NO_WINDOW` (`0x08000000`)** on the command before `spawn()`, using `std::os::windows::process::CommandExt::creation_flags`.

The block is wrapped in `#[cfg(windows)]` so Linux and macOS builds are unchanged.

## Behavior

- **Console LSP servers:** The flag suppresses the associated console window.
- **GUI servers:** The flag is ignored by the OS; behavior is unchanged.
- **Restarts / missing binary:** Same code path as Task 24 lifecycle; no extra windows on auto-restart or failed spawn.

## Reference

- [Microsoft: CreateProcessA — `CREATE_NO_WINDOW`](https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags)
