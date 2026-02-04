# Ferrite Project

## Vision

Transform Ferrite into a powerful, fast, lightweight, maximally customizable developer productivity hub. A "Swiss Army Knife" for developers - not a VSCode replacement, but a focused tool that does key things exceptionally well.

## Core Value

**The ONE thing that must always work:** Fast markdown editing with integrated terminal workspace.

## Tech Stack

- **Language:** Rust (performance, safety, single binary)
- **GUI Framework:** egui/eframe (immediate mode, cross-platform)
- **Terminal:** portable-pty + vte (PTY spawning, ANSI parsing)
- **Serialization:** serde + serde_json (settings, layouts)
- **Git:** git2 (repository detection, status)
- **Audio:** rodio (notification sounds)
- **Regex:** regex crate (pattern matching)

## Validated Requirements (v0.1.0 - v0.4.0)

### Phase 1: Terminal Essentials (Complete)
- [x] Tab switching (Ctrl+Tab, Ctrl+Shift+Tab)
- [x] Numeric shortcuts (Ctrl+1-9)
- [x] Tab rename (double-click, right-click menu)
- [x] Clear terminal (Ctrl+L)
- [x] Duplicate tab (New Terminal Here)
- [x] Copy/paste (Ctrl+Shift+C/V, right-click, Shift+Insert)
- [x] Text selection and auto-copy
- [x] Font size control
- [x] Scrollback buffer configuration
- [x] Close confirmation for running processes

### Phase 2: Grid & Tiling (Complete)
- [x] Horizontal/vertical splits
- [x] Resizable dividers
- [x] Close pane (Ctrl+W)
- [x] Focus navigation (Ctrl+Arrow keys)
- [x] Drag to reorder tabs
- [x] Drag to split/merge
- [x] Visual drop zones
- [x] Swap panes
- [x] Layout presets (2-column, 2-row, 2x2 grid)
- [x] Save/load layouts (JSON)
- [x] Workspace layouts per project

### Phase 3: Smart Features (Complete)
- [x] Claude Code prompt detection
- [x] Pattern matching (configurable regex)
- [x] Idle detection
- [x] Process detection
- [x] Breathing animation (waiting indicator)
- [x] Color customization
- [x] Tab badge (attention indicator)
- [x] Sound notification
- [x] Focus on detect
- [x] Maximize pane (Ctrl+Shift+M)
- [x] Cycle layouts (Ctrl+Shift+L)
- [x] Shell selector (PowerShell/cmd/bash/WSL)
- [x] Terminal themes (Solarized, Dracula, etc.)
- [x] Transparency
- [x] Custom color schemes

### Phase 4: Multi-Monitor & Advanced (Complete)
- [x] Pop out terminal (floating windows)
- [x] Drag to float
- [x] Snap to monitor (OS native)
- [x] Multi-monitor awareness
- [x] Workspace sync (re-dock on close)
- [x] Layout per monitor (via floating)
- [x] Scatter all tabs
- [x] Primary screen focus (Ctrl+Home)
- [x] Named workspaces
- [x] Workspace switcher

### Phase 5: Pro Features (Complete)
- [x] Git status detection in tab
- [x] Build/test detection (cargo, npm)
- [x] Error detection (red tab highlight)
- [x] Long-running command badge
- [x] Terminal macros (record/playback)
- [x] Auto-commands (startup command)
- [x] Watch mode (auto-rerun on file change)
- [x] Session export/import
- [x] Terminal screenshots

## Current Milestone: v0.5.0 "Swiss Army Knife"

**Goal:** Expand Ferrite into a modular developer productivity hub with AI integration, power terminal features, database tools, and productivity panels.

**Architecture:** Modular Panels - features as toggleable panels, single binary, show/hide what you need.

**Target features:**
- Developer Productivity Hub (task tracking, pomodoro, notes, GitHub/GitLab)
- AI-First Editor (Claude/LLM integration, inline completions, chat panel, Ollama)
- Power Terminal (SSH sessions, tmux-like features, command history, remote dev)
- Data & Database Tools (SQLite/PostgreSQL browser, CSV/JSON viewer, API testing)

## Decisions

- **2024-01-24:** Chose Modular Panels architecture over Plugin System. Single binary, all features built-in but toggleable. Can evolve to hybrid plugins later if needed.
- **2024-01-24:** Target "powerful fast lightweight customizable to the max" - not a VSCode replacement.

---

*Last updated: 2026-01-24*
