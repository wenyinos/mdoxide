# Windows MSI Installer Features

## Overview

The Windows MSI installer (`wix/main.wxs`) uses WiX Toolset v3 via `cargo-wix`. It presents a feature tree during installation, allowing users to opt in/out of each feature group independently.

## Feature Tree

| Feature | Default | Description |
|---------|---------|-------------|
| **MDOxide Editor** | On (required) | Core `mdoxide.exe` binary |
| Start Menu Shortcut | On | Shortcut in Start Menu > MDOxide |
| Desktop Shortcut | Off | Shortcut on the Desktop |
| **File Associations** | On | Registers MDOxide in "Open With" and Windows Default Apps |
| Markdown (.md, .markdown) | On | ProgId `MDOxide.md` |
| Plain Text (.txt) | On | ProgId `MDOxide.txt` |
| JSON (.json) | On | ProgId `MDOxide.json` |
| YAML (.yaml, .yml) | On | ProgId `MDOxide.yaml` |
| TOML (.toml) | On | ProgId `MDOxide.toml` |
| CSV (.csv, .tsv) | On | ProgId `MDOxide.csv` |
| **Explorer Context Menu** | Off | Right-click "Open with MDOxide" |
| Open with MDOxide (files) | Off | `HKLM\Software\Classes\*\shell\OpenWithMDOxide` |
| Open Folder with MDOxide | Off | `HKLM\Software\Classes\Directory\shell\OpenWithMDOxide` + Background |
| **Add to System PATH** | Off | Appends install dir to system PATH |

## Key Files

| File | Purpose |
|------|---------|
| `wix/main.wxs` | WiX installer definition (feature tree, components, UI) |
| `wix/license.rtf` | Minimal MIT license RTF (required by WixUI_FeatureTree, dialog is skipped) |

## Implementation Details

### File Associations

Uses the modern `OpenWithProgids` approach rather than forcefully setting extension defaults:

1. **ProgId registration** at `HKLM\Software\Classes\MDOxide.<ext>` with description, icon, and open command
2. **OpenWithProgids** entry under each extension (e.g., `HKLM\Software\Classes\.md\OpenWithProgids\MDOxide.md`)
3. **ApplicationCapabilities** registration at `HKLM\Software\OlaProeis\MDOxide\Capabilities` for Windows Settings > Default Apps
4. **RegisteredApplications** entry at `HKLM\Software\RegisteredApplications\MDOxide`

This is non-invasive -- it adds MDOxide to the "Open With" menu and Default Apps settings without overriding existing defaults.

### Explorer Context Menu

Registry entries under `HKLM\Software\Classes`:
- `*\shell\OpenWithMDOxide` -- right-click any file
- `Directory\shell\OpenWithMDOxide` -- right-click a folder
- `Directory\Background\shell\OpenWithMDOxide` -- right-click folder background (uses `%V` for current path)

### PATH Entry

Uses WiX `Environment` element with `Permanent='no'` (removed on uninstall), `Part='last'` (appends), `System='yes'` (system-level).

### Launch After Install

`CustomAction` with `FileKey='mdoxide.exe'` and `Return='asyncNoWait'`, triggered from ExitDialog's "Launch MDOxide" checkbox.

### UI Flow

Uses `WixUI_FeatureTree` which provides: Welcome -> CustomizeDlg (feature tree + install dir) -> VerifyReady -> Install -> Exit. License dialog is skipped via `Publish` overrides with higher `Order` values.

### Cross-Feature References

Registry values use `[APPLICATIONFOLDER]mdoxide.exe` instead of `[#mdoxide.exe]` to avoid ICE69 cross-feature reference warnings. Both resolve to the same path, but `[APPLICATIONFOLDER]` doesn't create a component cross-reference.

## Building

```bash
cargo install cargo-wix
cargo wix                     # full release build + MSI
cargo wix --no-build          # MSI only (requires existing target/release/mdoxide.exe)
```

WiX Toolset v3 must be installed. Either:
- Install via `winget install WiXToolset.WiXToolset` (requires admin)
- Download binaries from [wix3 releases](https://github.com/wixtoolset/wix3/releases) and use `--bin-path`

## Testing

1. Install with all features on -- verify associations, context menu, PATH, shortcuts
2. Install with all features off -- verify clean install, no registry pollution
3. Upgrade over previous version -- verify clean upgrade
4. Uninstall -- verify all registry entries and shortcuts removed
