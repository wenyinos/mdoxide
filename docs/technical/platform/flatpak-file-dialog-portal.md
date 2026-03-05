# Flatpak File Dialog Portal Support

## Overview

Ferrite's file dialogs (Open Folder, Open File, Save File) work correctly inside the Flatpak sandbox without requiring broad filesystem permissions. The fix addresses the issue where "Open Folder" failed on fresh launch unless a file from that folder was already open.

## Key Files

| File | Purpose |
|------|---------|
| `src/files/dialogs.rs` | `open_folder_dialog`, `resolve_initial_dir`, `is_flatpak` |
| `src/app/file_ops.rs` | `handle_open_workspace` with Flatpak-aware error handling |

## Implementation Details

### Root Cause

On a fresh Flatpak install (or when no recent files/workspaces exist), the folder dialog had no starting directory. The xdg-desktop-portal file chooser needs a navigable starting path; without one, it could fail silently or open in an inaccessible sandbox-internal path. Opening a file first gave the app a valid path to use as the initial directory, which is why the bug only appeared on fresh launch.

### Solution

1. **`resolve_initial_dir()`** — When no initial directory is provided (or the provided path doesn't exist), fall back to `$HOME` via `dirs::home_dir()`. This ensures the portal dialog always has a navigable starting point. Applied to all three dialog functions: `open_folder_dialog`, `open_multiple_files_dialog`, `save_file_dialog`.

2. **`is_flatpak()`** — Detects Flatpak environment by checking `std::env::var("FLATPAK_ID").is_ok()`.

3. **Flatpak-aware error handling** — When folder selection fails or the selected path is inaccessible, show user-friendly messages explaining sandbox limitations instead of generic errors.

### Portal Usage

The `rfd` crate (used for file dialogs) already uses xdg-desktop-portal by default on Linux via its built-in ashpd backend. No additional dependencies or Flatpak manifest changes were required. The fix is purely application logic.

## Dependencies Used

- `rfd` — Native file dialogs; uses xdg-desktop-portal on Linux automatically
- `dirs` — `home_dir()` for fallback initial directory

## Flatpak Manifest Impact

**None.** No changes to `finish-args` or permissions. The existing manifest (IPC, X11/Wayland, DRI only) remains correct. Portal access is automatic in the Freedesktop runtime.

## Usage / Testing

1. Build and install Ferrite via Flatpak
2. Fresh launch (no recent files) → File > Open Folder → portal dialog should appear starting at home directory
3. Select a folder → workspace opens with file tree
4. Non-Flatpak builds → unchanged behavior, uses native dialogs
5. If access fails → user sees Flatpak-specific error message

## Related

- [File Dialogs](../files/file-dialogs.md) — General file dialog implementation
- [Flathub Maintenance](../../flathub-maintenance.md) — Release checklist, cargo-sources, moderation
