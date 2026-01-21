# Ferrite Terminal Roadmap

## Vision: Multi-Monitor Terminal Workspace

Transform Ferrite into a powerful terminal workspace with **grid/Kanban-style terminal management**, **Claude Code integration**, and **multi-monitor support**. Think tmux/i3wm tiling, but with visual drag-and-drop and smart AI detection.

---

## Phase 1: Terminal Essentials ⚡ (v0.3.0)
**Status:** In Progress (4/5 completed)
**Goal:** Make the terminal feature production-ready with essential UX

### Navigation & Shortcuts
- [x] **Tab switching** - Ctrl+Tab / Ctrl+Shift+Tab to cycle through terminals
- [x] **Numeric shortcuts** - Ctrl+1-9 to jump to specific terminal
- [x] **Tab rename** - Double-click tab or right-click → "Rename" to set custom names
- [x] **Clear terminal** - Ctrl+L to clear screen (sends Ctrl+L character)
- [ ] **Duplicate tab** - Right-click → "New Terminal Here" (same directory)

### Copy/Paste
- [ ] **Selection copy** - Auto-copy on mouse release (optional setting)
- [ ] **Paste improvements** - Ctrl+V or Shift+Insert to paste
- [ ] **Right-click paste** - Quick paste from context menu

### Settings
- [ ] **Font size control** - Adjust terminal font size independently
- [ ] **Scrollback buffer** - Configurable history size (default 10k lines)
- [ ] **Close confirmation** - Warn before closing terminal with running process

---

## Phase 2: Grid & Tiling System 🎯 (v0.3.1)
**Status:** Planned
**Goal:** Kanban-style terminal layout with drag-and-drop

### Split Panes
- [ ] **Horizontal split** - Right-click → "Split Horizontally" or Ctrl+Shift+H
- [ ] **Vertical split** - Right-click → "Split Vertically" or Ctrl+Shift+V
- [ ] **Resizable dividers** - Drag borders to resize panes
- [ ] **Close pane** - Ctrl+W closes active pane (not entire panel)
- [ ] **Focus navigation** - Ctrl+Arrow keys to move between panes

### Drag-and-Drop Kanban
- [ ] **Drag to reorder** - Drag terminal tabs to rearrange
- [ ] **Drag to split** - Drag tab to edge → create split
- [ ] **Drag to merge** - Drag tab to existing pane → add to that group
- [ ] **Visual drop zones** - Highlight where terminal will land
- [ ] **Swap panes** - Drag entire pane to swap with another

### Layout Management
- [ ] **Save layout** - Right-click → "Save Layout As..." (JSON file)
- [ ] **Load layout** - Quick-load saved terminal arrangements
- [ ] **Layout presets** - Built-in layouts (2-column, 3-row, quad, etc.)
- [ ] **Workspace layouts** - Auto-load layout per project folder

---

## Phase 3: Smart Features 🧠 (v0.3.2)
**Status:** Planned
**Goal:** Claude Code integration and intelligent terminal behavior

### Claude Code Detection ⭐
- [ ] **Prompt detection** - Detect when terminal shows `>` prompt (Claude waiting)
- [ ] **Pattern matching** - Configurable regex patterns for other prompts
- [ ] **Idle detection** - No output for X seconds = waiting for input
- [ ] **Process detection** - Check if foreground process is `claude`, `node`, etc.

### Visual Indicators
- [ ] **Breathing animation** - Slow color pulse when waiting for input
- [ ] **Color customization** - Choose breathing color (default: soft blue)
- [ ] **Tab badge** - Small dot/icon on tab when terminal needs attention
- [ ] **Sound notification** - Optional chime when prompt detected (disabled by default)
- [ ] **Focus on detect** - Auto-switch to terminal when Claude starts waiting

### Smart Shortcuts
- [ ] **Bring to front** - Ctrl+Shift+1 brings Terminal 1 to main view (focused pane)
- [ ] **Maximize pane** - Ctrl+Shift+M temporarily maximizes active pane (like Zoom in tmux)
- [ ] **Restore layout** - Esc exits maximized mode
- [ ] **Cycle layouts** - Ctrl+Shift+L cycles through saved layouts

### Shell & Themes
- [ ] **Shell selector** - Choose PowerShell/cmd/bash/WSL per terminal
- [ ] **Terminal themes** - Color scheme selector (Solarized, Dracula, etc.)
- [ ] **Transparency** - Optional terminal background transparency
- [ ] **Custom color schemes** - JSON-based theme files

---

## Phase 4: Multi-Monitor & Advanced 🖥️ (v0.4.0)
**Status:** Future
**Goal:** Multi-monitor support and workspace distribution

### Floating Windows
- [ ] **Pop out terminal** - Right-click → "Float Window" creates OS window
- [ ] **Drag to float** - Drag tab outside Ferrite → creates floating window
- [ ] **Snap to monitor** - Float window auto-snaps to monitor edges
- [ ] **Multi-monitor awareness** - Remember window positions per monitor
- [ ] **Workspace sync** - Floating terminals sync with main Ferrite workspace

### Monitor Layouts
- [ ] **Monitor detection** - Detect connected monitors (1-4+)
- [ ] **Layout per monitor** - Save different grid layouts for each screen
- [ ] **Quick distribute** - Right-click → "Distribute to Monitors" spreads terminals
- [ ] **Monitor shortcuts** - Ctrl+Shift+F1-F4 moves terminal to specific monitor
- [ ] **Primary screen focus** - Ctrl+Home always focuses main monitor

### Workspace Presets
- [ ] **Named workspaces** - "Development", "Monitoring", "Claude Workflow"
- [ ] **Multi-monitor presets** - Save entire 4-monitor setup as one preset
- [ ] **Auto-detect workspace** - Load workspace based on folder name/git repo
- [ ] **Workspace switcher** - Quick menu to switch entire terminal layout

> **Note on Full 4-Monitor Spanning:**
> True full-screen spanning across 4 monitors is **very complex** (requires OS-level window management, driver coordination). Instead, we use **floating windows** which you can manually arrange across monitors. This is more flexible and respects OS window management.

---

## Phase 5: Pro Features 🚀 (v0.5.0+)
**Status:** Ideas
**Goal:** Advanced productivity and automation

### Advanced Detection
- [ ] **Git status detection** - Show branch/status in terminal tab when in git repo
- [ ] **Build/test detection** - Detect `cargo build`, `npm test` → show progress indicator
- [ ] **Error detection** - Highlight tab in red when command fails
- [ ] **Long-running command** - Badge when command runs > 30 seconds

### Automation
- [ ] **Terminal macros** - Record/replay command sequences
- [ ] **Auto-commands** - Run commands on terminal create (e.g., `cd ~/project && npm start`)
- [ ] **Startup scripts** - Run shell script when opening workspace
- [ ] **Watch mode** - Auto-rerun command on file change

### Collaboration
- [ ] **Session export** - Export terminal layout + history as shareable file
- [ ] **Session import** - Load someone else's terminal setup
- [ ] **Terminal screenshots** - Export terminal output as image/HTML

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
