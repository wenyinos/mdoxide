# Single Instance & File Opening Test

Test the single-instance protocol and file association features (v0.2.7).

## Single Instance Protocol

### Basic Test
1. Launch MDOxide normally
2. Double-click this .md file in Windows Explorer
3. **Expected:** File opens as a new tab in the existing MDOxide window (no new process)

### Multiple Files
1. With MDOxide running, select multiple .md files in Explorer
2. Right-click -> Open With -> MDOxide (or drag onto MDOxide)
3. **Expected:** All files open as tabs in existing window

### Speed Test
1. With MDOxide running, double-click a file in Explorer
2. **Expected:** Tab appears in <100ms (near-instant, was 3-10s before)

## Checklist

- [ ] Second instance forwards file path and exits quickly
- [ ] File opens as new tab in existing window
- [ ] No new MDOxide windows spawn
- [ ] Lock file is created properly
- [ ] Lock file is cleaned up on normal exit
- [ ] If MDOxide crashes, stale lock file doesn't prevent restart
- [ ] Multiple rapid file opens all get handled

## File Associations (Windows MSI)

After installing via .msi:

- [ ] .md files show "Open with MDOxide" in right-click context menu
- [ ] .json files show "Open with MDOxide"
- [ ] .yaml/.yml files show "Open with MDOxide"
- [ ] .toml files show "Open with MDOxide"
- [ ] .csv/.tsv files show "Open with MDOxide"
- [ ] .txt files show "Open with MDOxide"
- [ ] MDOxide appears in Windows Settings -> Default Apps
- [ ] Context menu "Open Folder with MDOxide" works on directories
- [ ] Explorer background right-click "Open Folder with MDOxide" works
