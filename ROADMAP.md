# Ferrite Terminal Roadmap

## Vision: Multi-Monitor Terminal Workspace

Transform Ferrite into a powerful terminal workspace with **grid/Kanban-style terminal management**, **Claude Code integration**, and **multi-monitor support**. Think tmux/i3wm tiling, but with visual drag-and-drop and smart AI detection.

---

## Phase 1: Terminal Essentials ⚡ (v0.3.0)
**Status:** Completed ✅
**Goal:** Make the terminal feature production-ready with essential UX

### Navigation & Shortcuts
> **Note:** Terminal shortcuts use **context-aware** behavior. Ctrl+Tab/Ctrl+1-9 control terminals when terminal is focused, and control files when editor is focused.

- [x] **Tab switching** - Ctrl+Tab / Ctrl+Shift+Tab to cycle through terminals (when terminal focused)
- [x] **Numeric shortcuts** - Ctrl+1-9 to jump to specific terminal (when terminal focused)
- [x] **Tab rename** - Double-click tab or right-click → "Rename" to set custom names
- [x] **Clear terminal** - Ctrl+L to clear screen when terminal focused (sends Ctrl+L to shell)
- [x] **Duplicate tab** - Right-click → "New Terminal Here" (same directory)

**Context-Aware Keyboard Behavior:**
| Shortcut | When Editor Focused | When Terminal Focused |
|----------|---------------------|----------------------|
| Ctrl+Tab | Switch file tabs | Switch terminal tabs |
| Ctrl+Shift+Tab | Previous file tab | Previous terminal tab |
| Ctrl+1-6 | Format markdown headings | Jump to terminal 1-6 |
| Ctrl+7-9 | (unused) | Jump to terminal 7-9 |
| Ctrl+W | Close file tab | (file behavior) |
| Ctrl+F4 | (unused) | Close terminal tab |

### Copy/Paste
- [x] **Copy terminal content** - Ctrl+Shift+C to copy visible screen to clipboard
- [x] **Paste from clipboard** - Ctrl+Shift+V, right-click paste, or Shift+Insert
- [x] **Selection copy** - Auto-copy selected text on mouse release (optional setting)
- [x] **Text selection** - Mouse drag to select specific text (currently copies entire screen)
- [x] **Right-click paste** - Quick paste from context menu

### Settings
- [x] **Font size control** - Adjust terminal font size independently
- [x] **Scrollback buffer** - Configurable history size (default 10k lines)
- [x] **Close confirmation** - Warn before closing terminal with running process

---

## Phase 2: Grid & Tiling System ✅ (v0.3.1)
**Status:** Complete
**Goal:** Kanban-style terminal layout with drag-and-drop

### Split Panes
- [x] **Horizontal split** - Right-click → "Split Horizontally" or Ctrl+Shift+H
- [x] **Vertical split** - Right-click → "Split Vertically" or Ctrl+Shift+V
- [x] **Resizable dividers** - Drag borders to resize panes
- [x] **Close pane** - Ctrl+W closes active pane (not entire panel)
- [x] **Focus navigation** - Ctrl+Arrow keys to move between panes

### Drag-and-Drop Kanban
- [x] **Drag to reorder** - Drag terminal tabs to rearrange
- [x] **Drag to split** - Drag tab to edge → create split
- [x] **Drag to merge** - Drag tab to center → bring to active
- [x] **Visual drop zones** - Highlight where terminal will land
- [x] **Swap panes** - Drag entire pane to swap with another

### Layout Management
- [x] **Layout presets** - Built-in layouts (2-column, 2-row, 2x2 grid)
- [x] **Save layout** - Right-click → "Save Layout As..." (JSON file)
- [x] **Load layout** - Quick-load saved terminal arrangements
- [x] **Workspace layouts** - Auto-load layout per project folder

---

## Phase 3: Smart Features ✅ (v0.3.2)
**Status:** Complete
**Goal:** Claude Code integration and intelligent terminal behavior

### Claude Code Detection ⭐
- [x] **Prompt detection** - Detect when terminal shows `>` prompt (Claude waiting)
- [x] **Pattern matching** - Configurable regex patterns for other prompts
- [x] **Idle detection** - No output for X seconds = waiting for input
- [x] **Process detection** - Check if foreground process is `claude`, `node`, etc.

### Visual Indicators
- [x] **Breathing animation** - Slow color pulse when waiting for input
- [x] **Color customization** - Choose breathing color (default: soft blue)
- [x] **Tab badge** - Small dot/icon on tab when terminal needs attention
- [x] **Sound notification** - Optional chime when prompt detected (disabled by default)
- [x] **Focus on detect** - Auto-switch to terminal when Claude starts waiting

### Smart Shortcuts
- [x] **Bring to front** - Ctrl+1-9 focuses Terminal N (works with floating)
- [x] **Maximize pane** - Ctrl+Shift+M temporarily maximizes active pane (like Zoom in tmux)
- [x] **Restore layout** - Esc exits maximized mode
- [x] **Cycle layouts** - Ctrl+Shift+L cycles through saved layouts

### Shell & Themes
- [x] **Shell selector** - Choose PowerShell/cmd/bash/WSL per terminal
- [x] **Terminal themes** - Color scheme selector (Solarized, Dracula, etc.)
- [x] **Transparency** - Optional terminal background transparency
- [x] **Custom color schemes** - JSON-based theme files

---

## Phase 4: Multi-Monitor & Advanced 🖥️ (v0.4.0)
**Status:** Completed (Core Features) ✅
**Goal:** Multi-monitor support and workspace distribution

### Floating Windows
- [x] **Pop out terminal** - Right-click → "Float Window" creates OS window
- [x] **Drag to float** - Drag tab outside Ferrite → creates floating window
- [x] **Snap to monitor** - Float window auto-snaps to monitor edges (OS Native)
- [x] **Multi-monitor awareness** - Remember window positions per monitor (via OS/egui)
- [x] **Workspace sync** - Floating terminals sync with main Ferrite workspace (re-dock on close)

### Monitor Layouts
- [ ] **Monitor detection** - Detect connected monitors (1-4+) (Limited by OS API access)
- [x] **Layout per monitor** - Save different grid layouts for each screen (via Floating Windows)
- [x] **Quick distribute** - Right-click → "Distribute to Monitors" spreads terminals (Implemented as "Scatter All Tabs")
- [x] **Monitor shortcuts** - Ctrl+Shift+F1-F4 moves terminal to specific monitor (Implemented as Focus Main + Pop Out)
- [x] **Primary screen focus** - Ctrl+Home always focuses main monitor

### Workspace Presets
- [x] **Named workspaces** - "Development", "Monitoring", "Claude Workflow" (Save/Load)
- [x] **Multi-monitor presets** - Save entire 4-monitor setup as one preset (Workspace Save)
- [ ] **Auto-detect workspace** - Load workspace based on folder name/git repo
- [x] **Workspace switcher** - Quick menu to switch entire terminal layout

> **Note on Full 4-Monitor Spanning:**
> True full-screen spanning across 4 monitors is **very complex** (requires OS-level window management, driver coordination). Instead, we use **floating windows** which you can manually arrange across monitors. This is more flexible and respects OS window management.

---

## Phase 5: Pro Features 🚀 (v0.5.0+)
**Status:** Completed ✅
**Goal:** Advanced productivity and automation

### Advanced Detection
- [x] **Git status detection** - Show branch/status in terminal tab when in git repo
- [x] **Build/test detection** - Detect `cargo build`, `npm test` → show progress indicator
- [x] **Error detection** - Highlight tab in red when command fails
- [x] **Long-running command** - Badge when command runs > 30 seconds

### Automation
- [x] **Terminal macros** - Record/replay command sequences (Playback from settings)
- [x] **Auto-commands** - Run commands on terminal create (Startup command setting)
- [x] **Startup scripts** - Run shell script when opening workspace (via Startup command)
- [x] **Watch mode** - Auto-rerun command on file change

### Collaboration
- [x] **Session export** - Export terminal layout + history as shareable file
- [x] **Session import** - Load someone else's terminal setup
- [x] **Terminal screenshots** - Export terminal output as image/HTML

---

## Manual Test Plan 📝

Please verify the following scenarios to ensure stability:

### 1. Terminal Essentials
- [ ] **Run a Command:** Type `dir` (Windows) or `ls` (Linux) and press Enter. Output should appear.
- [ ] **Interrupt Process:** Run `ping -t google.com`. Press `Ctrl+C`. The process should stop immediately.
- [ ] **Selection Copy:** Enable "Copy Selection Automatically" in Settings > Terminal. Select text with mouse. Paste it into Notepad to verify.
- [ ] **Manual Copy/Paste:** Select text -> `Ctrl+Shift+C`. Paste with `Ctrl+Shift+V`.
- [ ] **Word Deletion:** Type a sentence. Press `Ctrl+W`. The last word should be deleted.
- [ ] **Settings:** Go to Settings > Terminal. Change Font Size to 20. The terminal font should grow. Change Scrollback to 50000. New terminals should use this limit.

### 2. Grid & Splitting
- [ ] **Split Horizontal:** Right-click a tab -> "Split Horizontal". You should see two terminals side-by-side.
- [ ] **Split Vertical:** Right-click a tab -> "Split Vertical". You should see two terminals stacked.
- [ ] **Nested Split:** In one of the split panes, right-click and split again. It should create a subdivision.
- [ ] **Resizing:** Drag the divider between two terminals. They should resize smoothly without freezing.
- [ ] **Focus:** Click on different panes. The active cursor should move to the clicked pane.
- [ ] **Focus Navigation:** Use `Ctrl+Left/Right/Up/Down` to jump focus between panes without clicking.
- [ ] **Close Pane:** Click a pane to focus it. Press `Ctrl+W`. Only that pane should close. If it was the last pane, the tab should close.
- [ ] **Layout Presets:** Click the "+" button context menu -> "Layouts" -> "2x2 Grid". A new tab with 4 terminals should open.

### 3. Smart Features
- [ ] **Prompt Detection:** Open a terminal. It should have a subtle breathing blue border when sitting at the prompt.
- [ ] **Activity Indicator:** Run `timeout 5` (Windows) or `sleep 5` (Linux). The breathing border should stop while the command runs. When it finishes and the prompt returns, the breathing should resume.

---

## Feasibility Analysis

| Feature | Achievable? | Effort | Technical Notes |
|---------|-------------|--------|-----------------|
| **Grid/tiling layout** | ✅ YES | High | Similar to egui's `Grid`, custom split logic |
| **Drag-and-drop** | ✅ YES | Medium | egui has drag-drop primitives |
| **Claude detection** | ✅ YES | Low-Medium | Parse last line, check for `>` or custom regex |
| **Breathing animation** | ✅ YES | Low | egui animation with `animate_bool` |
| **Floating windows** | ✅ YES | High | Use `egui::ViewportBuilder`, multi-window support |
| **Multi-monitor awareness** | ✅ YES | Medium | `winit` provides monitor info |
| **4-monitor full-screen** | ⚠️ VERY HARD | Very High | OS-specific, not recommended |

---

## Timeline Estimate

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| Phase 1 | 2-3 weeks | Tab switching, rename, clear, duplicate |
| Phase 2 | 4-6 weeks | Split panes, drag-drop, layout save/load |
| Phase 3 | 3-4 weeks | Claude detection, breathing colors, smart shortcuts |
| Phase 4 | 4-6 weeks | Floating windows, multi-monitor presets |
| Phase 5 | TBD | Advanced features based on user feedback |

**Total to Multi-Monitor Support:** ~3-4 months of focused development

---

## Why This Is Special

### What Makes This Different from VSCode/Tmux?

1. **Visual Kanban** - Drag-and-drop terminal arrangement (not keyboard-only like tmux)
2. **Claude Integration** - Built-in AI prompt detection with visual breathing
3. **Markdown + Terminals** - Edit docs while running commands in same window
4. **Multi-monitor native** - Designed for 4-monitor dev setups from day one
5. **Workspace-aware** - Terminals remember positions per project

### Use Cases

**Scenario 1: Claude Code Workflow**
- Terminal 1 (left): Claude Code main session (breathing blue when waiting)
- Terminal 2 (top-right): `npm run dev` with live reload
- Terminal 3 (bottom-right): `git status` monitoring
- Ctrl+Shift+1 instantly focuses Claude terminal when it needs input

**Scenario 2: Multi-Monitor Development**
- Monitor 1: Ferrite editor with markdown docs
- Monitor 2: 4 terminals in quad layout (build, test, logs, shell)
- Monitor 3: Floating terminal running database
- Monitor 4: Another floating terminal for SSH session

**Scenario 3: Kanban Task Board**
- "TODO" pane: Terminal with task list
- "In Progress" pane: Active build/test terminal (breathing green)
- "Done" pane: Completed command history
- Drag terminals between panes as tasks progress

---

## Contributing

This is an ambitious roadmap! If you want to help build any of these features, check out:
- [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines
- [Issues](https://github.com/OlaProeis/Ferrite/issues) for specific tasks
- [Discussions](https://github.com/OlaProeis/Ferrite/discussions) for feature ideas

Let's build the ultimate terminal workspace! 🚀
