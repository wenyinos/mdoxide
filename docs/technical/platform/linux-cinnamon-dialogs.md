# Linux Cinnamon Desktop Detection for File Dialogs

## Problem (GitHub #116)

On Linux Mint Cinnamon, `XDG_CURRENT_DESKTOP` is set to `X-Cinnamon`, which lowercases to `x-cinnamon`. The desktop detection code only matched `"cinnamon"`, causing Cinnamon to be misclassified as a non-native desktop that requires xdg-desktop-portal. This led to:

1. Dialog cancellation (user pressing Cancel) being misreported as a portal failure.
2. Portal installation instructions recommending `xdg-desktop-portal-wlr` (wrong for Cinnamon).

## Solution

Three fixes in `src/files/dialogs.rs` (and related call sites):

### 1. Native Desktop Detection

Added `"x-cinnamon"` to the `has_native` match list in `detect_linux_desktop()` so that `XDG_CURRENT_DESKTOP=X-Cinnamon` is correctly recognized as a desktop with native file dialog support.

### 2. Portal Install Instructions

Updated `portal_install_instructions()` to accept the detected desktop environment and return Cinnamon-appropriate packages (`xdg-desktop-portal-xapp` and `xdg-desktop-portal-gtk`) instead of the wlroots-specific `xdg-desktop-portal-wlr`.

### 3. Cancellation Handling

Changed all dialog functions (`open_folder_dialog`, `open_multiple_files_dialog`, `save_file_dialog`) and the export dialog in `app/export.rs` to always return `DialogResult::Cancelled` when rfd returns `None`. Previously, `None` on portal-requiring desktops was classified as `DialogResult::Failed`, which showed an intrusive error dialog even for normal user cancellation.

Debug-level logs are still emitted on portal-requiring desktops so the issue is visible in logs without false-positive error dialogs.

## Files Changed

| File | Change |
|------|--------|
| `src/files/dialogs.rs` | Added `x-cinnamon` to native list, desktop-aware portal instructions, cancellation fix |
| `src/app/file_ops.rs` | Updated `portal_install_instructions()` call to pass desktop env |
| `src/app/export.rs` | Applied same cancellation fix to export save dialog |
