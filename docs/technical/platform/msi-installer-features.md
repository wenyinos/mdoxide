# Windows MSI Installer Features

## Overview

The Windows MSI installer (`wix/main.wxs`) uses WiX Toolset v3 via `cargo-wix`. It presents a feature tree during installation, allowing users to opt in/out of each feature group independently.

## Feature Tree

| Feature | Default | Description |
|---------|---------|-------------|
| **Ferrite Editor** | On (required) | Core `ferrite.exe` binary |
| Start Menu Shortcut | On | Shortcut in Start Menu > Ferrite |
| Desktop Shortcut | Off | Shortcut on the Desktop |
| **File Associations** | On | Registers Ferrite in "Open With" and Windows Default Apps |
| Markdown (.md, .markdown) | On | ProgId `Ferrite.md` |
| Plain Text (.txt) | On | ProgId `Ferrite.txt` |
| JSON (.json) | On | ProgId `Ferrite.json` |
| YAML (.yaml, .yml) | On | ProgId `Ferrite.yaml` |
| TOML (.toml) | On | ProgId `Ferrite.toml` |
| CSV (.csv, .tsv) | On | ProgId `Ferrite.csv` |
| **Explorer Context Menu** | Off | Right-click "Open with Ferrite" |
| Open with Ferrite (files) | Off | `HKLM\Software\Classes\*\shell\OpenWithFerrite` |
| Open Folder with Ferrite | Off | `HKLM\Software\Classes\Directory\shell\OpenWithFerrite` + Background |
| **Add to System PATH** | Off | Appends install dir to system PATH |

## Key Files

| File | Purpose |
|------|---------|
| `wix/main.wxs` | WiX installer definition (feature tree, components, UI) |
| `wix/license.rtf` | Minimal MIT license RTF (required by WixUI_FeatureTree, dialog is skipped) |

## Implementation Details

### File Associations

Uses the modern `OpenWithProgids` approach rather than forcefully setting extension defaults:

1. **ProgId registration** at `HKLM\Software\Classes\Ferrite.<ext>` with description, icon, and open command
2. **OpenWithProgids** entry under each extension (e.g., `HKLM\Software\Classes\.md\OpenWithProgids\Ferrite.md`)
3. **ApplicationCapabilities** registration at `HKLM\Software\OlaProeis\Ferrite\Capabilities` for Windows Settings > Default Apps
4. **RegisteredApplications** entry at `HKLM\Software\RegisteredApplications\Ferrite`

This is non-invasive -- it adds Ferrite to the "Open With" menu and Default Apps settings without overriding existing defaults.

### Explorer Context Menu

Registry entries under `HKLM\Software\Classes`:
- `*\shell\OpenWithFerrite` -- right-click any file
- `Directory\shell\OpenWithFerrite` -- right-click a folder
- `Directory\Background\shell\OpenWithFerrite` -- right-click folder background (uses `%V` for current path)

### PATH Entry

Uses WiX `Environment` element with `Permanent='no'` (removed on uninstall), `Part='last'` (appends), `System='yes'` (system-level).

### Launch After Install

`CustomAction` with `FileKey='ferrite.exe'` and `Return='asyncNoWait'`, triggered from ExitDialog's "Launch Ferrite" checkbox.

### UI Flow

Uses `WixUI_FeatureTree` which provides: Welcome -> CustomizeDlg (feature tree + install dir) -> VerifyReady -> Install -> Exit. License dialog is skipped via `Publish` overrides with higher `Order` values.

### Cross-Feature References

Registry values use `[APPLICATIONFOLDER]ferrite.exe` instead of `[#ferrite.exe]` to avoid ICE69 cross-feature reference warnings. Both resolve to the same path, but `[APPLICATIONFOLDER]` doesn't create a component cross-reference.

## Building

```bash
cargo install cargo-wix
cargo wix                     # full release build + MSI
cargo wix --no-build          # MSI only (requires existing target/release/ferrite.exe)
```

WiX Toolset v3 must be installed. Either:
- Install via `winget install WiXToolset.WiXToolset` (requires admin)
- Download binaries from [wix3 releases](https://github.com/wixtoolset/wix3/releases) and use `--bin-path`

## Testing

1. Install with all features on -- verify associations, context menu, PATH, shortcuts
2. Install with all features off -- verify clean install, no registry pollution
3. Upgrade over previous version -- verify clean upgrade
4. Uninstall -- verify all registry entries and shortcuts removed
