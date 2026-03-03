Refactoring Test Checklist for Ferrites

All of these features touch code that was moved during the refactoring. Test each one: Title Bar (title_bar.rs) [x ] App icon displays correctly (not garbled) [x ] Window title shows filename correctly

[ ] Auto-save indicator (toggle on/off) works ?? not working? the toggle is working

[x] View mode segmented control (Raw/Split/Rendered) switches correctly
[x] Settings gear button opens settings panel
[x] Zen mode button works
[x] Minimize button works
[ ] Maximize/restore button works (icon changes between states); no its just a square on fullscreen also hoovering hides the icon, different from minimize button 

[x] Close button works (shows save prompt if unsaved)
[x] Window drag to move works from title bar area

Status Bar (status_bar.rs)
[x] File path displays correctly
[x] Recent files popup appears when clicking path
[x] Line/column indicator updates with cursor movement
[x] Word count displays
[x] Encoding selector shows and changes encoding
[x] CSV rainbow columns toggle works
[x] CSV delimiter selector works
[x] Toast messages appear and auto-dismiss
Central Panel / Editor (central_panel.rs)
[x] Tab bar shows all open tabs
[x] Tab close button (x) works
[x] Tab switching works (click + Ctrl+Tab)
[ ] Tab drag reorder works, not shure if implemented

[x] Raw mode editing works (typing, selection, scrolling)
[ ] Rendered mode editing works (WYSIWYG) | it works but the raw editor has some stuttering when loading the new, does not look very nice. like glitching reloading

[x] Split mode shows both panes
[x] CSV viewer displays tables correctly
[x] JSON/YAML/TOML tree viewer works
[x] Minimap displays and navigates
[x] Navigation buttons (top/middle/bottom) work
Keyboard Shortcuts (keyboard.rs)
[X] Ctrl+N (new file)
[x] Ctrl+O (open file)
[x] Ctrl+S (save)
[x] Ctrl+W (close tab)
[x] Ctrl+Tab / Ctrl+Shift+Tab (next/prev tab)
[x] Ctrl+F (find)
[x] Ctrl+H (find & replace)
[x] Ctrl+G (go to line)
[ ] Ctrl+Shift+D (duplicate line), duplicates the line over where the cursor is, and sometimes lines way over something wrong here, the other line commands like move line is okay.

[ ] Ctrl+B (toggle file tree) | this has changed, was conflicting with Bolding text

[x] Ctrl+P (quick switcher)
[x] Ctrl+Shift+F (search in files)
File Operations (file_ops.rs)
[x] Open file dialog works
[x] Save / Save As works
[x] Open workspace/folder works
[x] Close workspace works
[x] Drag-drop file opens it
[x] Drag-drop folder opens as workspace
[ ] Drag-drop image saves to assets/ and inserts link | Link inserts wrong place, several lines above where the cursor is or where we dropped it, also not saving to assets folder, and not then displaying naturally


[x] File tree context menu (new file, rename, delete)
[ ] File watcher detects external changes | not working, not shure if implemented?

[x] Git status refreshes after save
Input Handling (input_handling.rs)
[x] Ctrl+Z (undo) works
[x] Ctrl+Y (redo) works
[ ] Smart paste: select text, paste URL -> creates [text](url) link | not working not shure if implemented

[x] Auto-close brackets: type ( -> gets () with cursor inside
[x] Auto-close quotes: type " -> gets "" with cursor inside
[x] Skip-over: type ) when cursor is before ) -> moves past it
[]x] Alt+Up/Down ()move line) works
Navigation (navigation.rs)
[x] Toggle outline panel shows/hides
[x] Toggle terminal panel shows/hides
[x] Toggle zen mode works
[x] Theme cycling works (Ctrl+Shift+T)
[x] Heading navigation from outline panel
[x] Heading navigation from minimap
Formatting (formatting.rs)
[x] Ctrl+B (bold) **wraps** selection
[x] Ctrl+I (italic) *wraps* selection
[x] Other format commands from toolbar
[x] TOC generation (Ctrl+Shift+U)
[x] JSON format/pretty-print
[x] YAML format/pretty-print
Line Operations (line_ops.rs)
[x] Go to line (Ctrl+G) dialog and navigation

[x] Move line up ()Alt+Up)
[]x] Move line down ()Alt+Down)
[x] Delete line (Ctrl+D)
Find/Replace (find_replace.rs)

[x] Find next/prev (F3/Shift+F3)
[x] Replace works
[x] Replace all works

Export (export.rs)
[ ] Export as HTML (Ctrl+Shift+E) creates file | Hotkey not working, conflict ctrl+shift+E closes the file tree workspace

[x] Copy as HTML puts content on clipboard
Dialogs (dialogs.rs)
[x] Settings panel opens and saves
[x] Close confirmation dialog appears for unsaved files
[x] Crash recovery dialog appears after crash
[x] Auto-save recovery dialog works
[x] Go-to-line dialog works