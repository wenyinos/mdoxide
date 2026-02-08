<overview>
## Problem Statement

Traditional notepad applications lack markdown support, forcing users to either use bloated electron-based editors (VS Code, Obsidian) that consume 500MB+ RAM, or plain text editors without formatting preview. Technical users, developers, and markdown writers need a fast, lightweight editor that provides real-time markdown rendering without sacrificing performance or simplicity.

## Target Users

**Primary Persona: Technical Writer / Developer**
- Writes documentation, notes, READMEs daily
- Values keyboard shortcuts and efficiency
- Needs quick startup for jotting ideas
- Works across Windows, macOS, Linux
- Frustrated by slow, bloated editors

**Secondary Persona: Markdown Enthusiast**
- Uses markdown for personal notes, blogs
- Wants live preview while writing
- Appreciates clean, distraction-free UI
- May not know all markdown syntax by heart

## Success Metrics

- Startup time: < 500ms cold start
- Memory usage: < 50MB RAM idle, < 100MB active
- Binary size: < 50MB compiled
- Markdown rendering: Matches CommonMark spec
- Cross-platform: Runs identically on Windows, macOS, Linux
- Session persistence: 100% restore of tabs/state on restart

</overview>

---

<functional-decomposition>

## Capability Tree

### Capability: Application Foundation
Core infrastructure that all other features depend on.

#### Feature: Application State Management
- **Description**: Central state struct managing all application data and UI state
- **Inputs**: User actions, file events, config changes
- **Outputs**: Updated state propagated to UI
- **Behavior**: Reactive state updates, event handling, state serialization

#### Feature: Configuration System
- **Description**: Persistent storage of user preferences and app settings
- **Inputs**: User preference changes, startup config load
- **Outputs**: JSON config file, loaded settings struct
- **Behavior**: Read/write to platform-specific config dirs, handle missing/corrupted config

#### Feature: Error Handling
- **Description**: Consistent error handling and user feedback
- **Inputs**: Operation results, system errors
- **Outputs**: User-friendly error messages, error recovery
- **Behavior**: Graceful degradation, error logging, user notification

---

### Capability: File Management
All file I/O operations and file state tracking.

#### Feature: File Operations
- **Description**: Open, save, save-as, and create new files
- **Inputs**: File paths, file content, user triggers
- **Outputs**: File content loaded, file saved to disk
- **Behavior**: Native file dialogs, path validation, write confirmation

#### Feature: Unsaved Changes Tracking
- **Description**: Track and indicate when files have unsaved modifications
- **Inputs**: Text changes, save events
- **Outputs**: Dirty flag per tab, visual indicator
- **Behavior**: Compare current vs saved content, prompt on close

#### Feature: Recent Files History
- **Description**: Track and display recently opened files for quick access
- **Inputs**: File open events, config storage
- **Outputs**: Ordered list of recent files with timestamps
- **Behavior**: Store last 10 files, deduplicate, validate existence on load

---

### Capability: Editor Core
The text editing experience and input handling.

#### Feature: Text Editor Widget
- **Description**: Main text input area with full editing capabilities
- **Inputs**: Keyboard input, mouse selection, clipboard
- **Outputs**: Text content, cursor position, selection range
- **Behavior**: Text insertion, deletion, selection, scrolling

#### Feature: Line Numbers
- **Description**: Display line numbers alongside editor content
- **Inputs**: Text content, scroll position
- **Outputs**: Rendered line number column
- **Behavior**: Sync with editor scroll, muted styling, toggle visibility

#### Feature: Word/Character Statistics
- **Description**: Real-time count of words, characters, lines, paragraphs
- **Inputs**: Text content
- **Outputs**: Statistics displayed in status bar
- **Behavior**: Efficient counting, update on text change

---

### Capability: Tab System
Multi-document editing with tabbed interface.

#### Feature: Tab Management
- **Description**: Create, switch, close, and organize document tabs
- **Inputs**: User actions (new tab, close, click)
- **Outputs**: Active tab state, tab list
- **Behavior**: Tab creation, switching, close with unsaved prompt

#### Feature: Tab Bar UI
- **Description**: Horizontal tab strip showing all open documents
- **Inputs**: Tab list, active tab, unsaved states
- **Outputs**: Rendered tab bar with close buttons
- **Behavior**: Click to switch, show unsaved indicator, add new tab button

#### Feature: Tab State Persistence
- **Description**: Save and restore open tabs between sessions
- **Inputs**: Tab list, file paths, active tab
- **Outputs**: Serialized tab state in config
- **Behavior**: Save on exit, restore on startup, handle missing files

---

### Capability: Markdown Rendering
Parse and display formatted markdown content.

#### Feature: Markdown Parser
- **Description**: Convert markdown text to rendered HTML/egui widgets
- **Inputs**: Raw markdown text
- **Outputs**: Parsed markdown AST, rendered preview
- **Behavior**: CommonMark spec compliance, real-time parsing

#### Feature: Preview Pane
- **Description**: Display rendered markdown alongside or instead of editor
- **Inputs**: Parsed markdown, view mode
- **Outputs**: Formatted preview with styling
- **Behavior**: Scroll sync with editor, styled elements

#### Feature: View Mode Toggle
- **Description**: Switch between edit-only, preview-only, and split views
- **Inputs**: User toggle action, keyboard shortcut
- **Outputs**: Updated view layout
- **Behavior**: Smooth transitions, persist preference

#### Feature: Code Block Syntax Highlighting
- **Description**: Apply syntax highlighting to fenced code blocks
- **Inputs**: Code block content, language identifier
- **Outputs**: Highlighted code in preview
- **Behavior**: Language detection, themed highlighting

#### Feature: Hyperlink Handling
- **Description**: Render and handle clickable links in preview
- **Inputs**: Parsed link elements
- **Outputs**: Clickable links that open in browser
- **Behavior**: Visual hover feedback, external browser launch

#### Feature: Table Rendering
- **Description**: Parse and display markdown tables
- **Inputs**: Table markdown syntax
- **Outputs**: Formatted table in preview
- **Behavior**: Column alignment, cell padding, borders

---

### Capability: Theme System
Visual customization and appearance management.

#### Feature: Theme Data Structure
- **Description**: Define colors, fonts, spacing for UI theming
- **Inputs**: Theme definition
- **Outputs**: Applied visual styles
- **Behavior**: Consistent color palette, font stack

#### Feature: Built-in Themes
- **Description**: Default light and dark themes
- **Inputs**: Theme selection
- **Outputs**: Applied theme
- **Behavior**: Full UI theming, preview theming

#### Feature: Theme Switching
- **Description**: UI to select and switch themes
- **Inputs**: User selection
- **Outputs**: Applied theme, persisted preference
- **Behavior**: Instant switch, save to config

---

### Capability: Window & UI
Application window and overall UI layout.

#### Feature: Window Management
- **Description**: Responsive window with proper sizing and title
- **Inputs**: Window events, current file
- **Outputs**: Window title, size, position
- **Behavior**: Title updates, size persistence, responsive layout

#### Feature: Status Bar
- **Description**: Bottom bar showing file path, stats, encoding, position
- **Inputs**: Current file, cursor position, stats
- **Outputs**: Rendered status bar
- **Behavior**: Real-time updates, clickable elements

#### Feature: Keyboard Shortcuts
- **Description**: Global keyboard shortcuts for common actions
- **Inputs**: Key events
- **Outputs**: Triggered actions
- **Behavior**: Ctrl+S save, Ctrl+N new, Ctrl+O open, etc.

</functional-decomposition>

---

<structural-decomposition>

## Repository Structure

```
sleek-markdown-editor/
├── Cargo.toml                 # Dependencies and project config
├── README.md                  # Project documentation
├── src/
│   ├── main.rs               # Entry point, eframe setup
│   ├── app.rs                # Main App struct, update loop
│   ├── state.rs              # AppState, event handling
│   ├── config/
│   │   ├── mod.rs            # Config module exports
│   │   ├── settings.rs       # Settings struct, defaults
│   │   └── persistence.rs    # Read/write config file
│   ├── editor/
│   │   ├── mod.rs            # Editor module exports
│   │   ├── widget.rs         # Text editor widget
│   │   ├── line_numbers.rs   # Line number rendering
│   │   └── stats.rs          # Word/character counting
│   ├── tabs/
│   │   ├── mod.rs            # Tab module exports
│   │   ├── manager.rs        # Tab creation/switching logic
│   │   ├── bar.rs            # Tab bar UI widget
│   │   └── state.rs          # Tab data structures
│   ├── files/
│   │   ├── mod.rs            # File module exports
│   │   ├── operations.rs     # Open/save/new logic
│   │   ├── dialogs.rs        # Native file dialogs
│   │   └── recent.rs         # Recent files tracking
│   ├── markdown/
│   │   ├── mod.rs            # Markdown module exports
│   │   ├── parser.rs         # Comrak integration
│   │   ├── preview.rs        # Preview pane widget
│   │   ├── syntax.rs         # Syntect integration
│   │   └── elements.rs       # Tables, links, code blocks
│   ├── theme/
│   │   ├── mod.rs            # Theme module exports
│   │   ├── colors.rs         # Color palette definitions
│   │   ├── light.rs          # Light theme
│   │   ├── dark.rs           # Dark theme
│   │   └── manager.rs        # Theme switching logic
│   ├── ui/
│   │   ├── mod.rs            # UI module exports
│   │   ├── layout.rs         # Main layout (split view)
│   │   ├── status_bar.rs     # Status bar widget
│   │   └── shortcuts.rs      # Keyboard shortcut handling
│   └── error.rs              # Error types and handling
├── assets/
│   └── icons/                # Application icons
└── tests/
    ├── editor_tests.rs
    ├── markdown_tests.rs
    └── config_tests.rs
```

## Module Definitions

### Module: config
- **Maps to capability**: Application Foundation
- **Responsibility**: Manage all persistent settings and preferences
- **Exports**:
  - `Settings` - User preferences struct
  - `load_config()` - Load config from disk
  - `save_config()` - Write config to disk
  - `config_dir()` - Get platform config directory

### Module: editor
- **Maps to capability**: Editor Core
- **Responsibility**: Text editing widget and related features
- **Exports**:
  - `EditorWidget` - Main text editor component
  - `LineNumbers` - Line number renderer
  - `Statistics` - Word/char/line counter

### Module: tabs
- **Maps to capability**: Tab System
- **Responsibility**: Multi-document tab management
- **Exports**:
  - `TabManager` - Tab creation/switching
  - `TabBar` - Tab bar UI widget
  - `Tab` - Individual tab data

### Module: files
- **Maps to capability**: File Management
- **Responsibility**: All file I/O operations
- **Exports**:
  - `open_file()` - Open file from disk
  - `save_file()` - Save to disk
  - `new_file()` - Create new document
  - `RecentFiles` - Recent file history

### Module: markdown
- **Maps to capability**: Markdown Rendering
- **Responsibility**: Parse and render markdown
- **Exports**:
  - `parse_markdown()` - Convert text to AST
  - `PreviewPane` - Rendered preview widget
  - `highlight_code()` - Syntax highlighting

### Module: theme
- **Maps to capability**: Theme System
- **Responsibility**: Visual theming
- **Exports**:
  - `Theme` - Theme data structure
  - `LIGHT_THEME` - Default light
  - `DARK_THEME` - Default dark
  - `ThemeManager` - Theme switching

### Module: ui
- **Maps to capability**: Window & UI
- **Responsibility**: Layout and window management
- **Exports**:
  - `MainLayout` - Split view layout
  - `StatusBar` - Bottom status bar
  - `handle_shortcuts()` - Keyboard handling

</structural-decomposition>

---

<dependency-graph>

## Dependency Chain

### Foundation Layer (Phase 0)
No dependencies - these are built first.

- **error.rs**: Defines Result types, error enums, error display
- **config/settings.rs**: Settings struct with serde derive
- **config/persistence.rs**: Config file read/write (depends on settings)
- **state.rs**: AppState struct (depends on config)

### Core Layer (Phase 1)
Basic functionality to get a working editor.

- **main.rs**: Entry point (depends on: app)
- **app.rs**: Main App impl (depends on: state, ui/layout)
- **editor/widget.rs**: Text editor (depends on: state)
- **files/operations.rs**: Open/save (depends on: state, error)
- **files/dialogs.rs**: Native dialogs (depends on: files/operations)

### Tab Layer (Phase 2)
Multi-document support.

- **tabs/state.rs**: Tab data structures (depends on: state)
- **tabs/manager.rs**: Tab logic (depends on: tabs/state, files)
- **tabs/bar.rs**: Tab bar UI (depends on: tabs/manager, theme)

### Markdown Layer (Phase 3)
Markdown parsing and preview.

- **markdown/parser.rs**: Comrak integration (depends on: error)
- **markdown/preview.rs**: Preview pane (depends on: parser, theme)
- **markdown/syntax.rs**: Syntect highlighting (depends on: theme)
- **markdown/elements.rs**: Tables, links, code (depends on: parser, syntax)

### Theme Layer (Phase 4)
Visual customization.

- **theme/colors.rs**: Color definitions (no deps)
- **theme/light.rs**: Light theme (depends on: colors)
- **theme/dark.rs**: Dark theme (depends on: colors)
- **theme/manager.rs**: Theme switching (depends on: light, dark, config)

### Polish Layer (Phase 5)
UI refinements.

- **editor/line_numbers.rs**: Line numbers (depends on: editor/widget, theme)
- **editor/stats.rs**: Statistics (depends on: editor/widget)
- **ui/status_bar.rs**: Status bar (depends on: state, theme, stats)
- **ui/shortcuts.rs**: Keyboard shortcuts (depends on: app, files, tabs)
- **ui/layout.rs**: Split view (depends on: editor, markdown/preview, theme)
- **files/recent.rs**: Recent files (depends on: config, files/operations)

</dependency-graph>

---

<implementation-roadmap>

## Development Phases

### Phase 0: Foundation
**Goal**: Establish project structure, error handling, and configuration system

**Entry Criteria**: Clean Rust workspace

**Tasks**:
- [ ] Initialize Cargo project with all dependencies (depends on: none)
  - Acceptance: `cargo build` succeeds with all deps
  - Test: Compile check

- [ ] Create error handling module with custom Result types (depends on: none)
  - Acceptance: Error enum covers file, config, parse errors
  - Test: Unit tests for error display

- [ ] Implement Settings struct with serde serialization (depends on: none)
  - Acceptance: Settings can serialize to JSON
  - Test: Serialize/deserialize roundtrip

- [ ] Implement config persistence to platform directories (depends on: Settings)
  - Acceptance: Config loads from ~/.config (Linux), %APPDATA% (Windows), ~/Library (Mac)
  - Test: Write and read config file

- [ ] Create AppState struct with initial fields (depends on: Settings, error)
  - Acceptance: AppState holds current file, tabs list, settings
  - Test: State initialization

**Exit Criteria**: Project compiles, config can be saved/loaded

**Delivers**: Foundation for all other features

---

### Phase 1: Core Editor
**Goal**: Working text editor with file operations

**Entry Criteria**: Phase 0 complete

**Tasks**:
- [ ] Create basic eframe window with responsive sizing (depends on: AppState)
  - Acceptance: Window opens, resizes smoothly, shows app name in title
  - Test: Manual window interaction

- [ ] Implement text editor widget with input capture (depends on: eframe window)
  - Acceptance: Can type text, see cursor, select text
  - Test: Type and verify text content

- [ ] Implement file open with native dialog (depends on: text editor, error)
  - Acceptance: File picker opens, selected file loads into editor
  - Test: Open known test file

- [ ] Implement file save and save-as (depends on: file open)
  - Acceptance: Content saves to disk, save-as prompts for location
  - Test: Save and verify file content

- [ ] Implement new file creation (depends on: file save)
  - Acceptance: Creates blank document, tracks unsaved state
  - Test: New file, type, verify unsaved indicator

- [ ] Add unsaved changes tracking with visual indicator (depends on: text editor)
  - Acceptance: Tab/title shows asterisk when modified
  - Test: Modify text, check indicator

- [ ] Implement basic keyboard shortcuts (depends on: file operations)
  - Acceptance: Ctrl+S saves, Ctrl+O opens, Ctrl+N new
  - Test: Shortcut triggers

**Exit Criteria**: Can create, edit, save, and open text files

**Delivers**: Functional text editor (single document)

---

### Phase 2: Tab System
**Goal**: Multi-document editing with tabs

**Entry Criteria**: Phase 1 complete

**Tasks**:
- [ ] Design Tab data structure with file path, content, dirty flag (depends on: Phase 1)
  - Acceptance: Tab struct holds all document state
  - Test: Create Tab, verify fields

- [ ] Implement TabManager for tab creation and switching (depends on: Tab struct)
  - Acceptance: Can create tabs, switch between them, close tabs
  - Test: Create 3 tabs, switch, verify content preserved

- [ ] Build TabBar UI widget with close buttons (depends on: TabManager, theme)
  - Acceptance: Tabs render horizontally, click to switch, X to close
  - Test: Visual inspection, click handling

- [ ] Add new tab button to TabBar (depends on: TabBar)
  - Acceptance: Plus button creates new empty tab
  - Test: Click plus, verify new tab

- [ ] Implement tab close with unsaved prompt (depends on: TabManager, unsaved tracking)
  - Acceptance: Closing unsaved tab prompts save/discard/cancel
  - Test: Modify, close, verify prompt

- [ ] Add Ctrl+Tab / Ctrl+W shortcuts (depends on: TabManager, shortcuts)
  - Acceptance: Ctrl+Tab cycles tabs, Ctrl+W closes current
  - Test: Shortcut behavior

- [ ] Persist open tabs between sessions (depends on: TabManager, config)
  - Acceptance: Tabs restore on app restart
  - Test: Open 3 files, close app, reopen, verify tabs

**Exit Criteria**: Full tab system with persistence

**Delivers**: Multi-document editing

---

### Phase 3: Markdown Rendering
**Goal**: Live markdown preview

**Entry Criteria**: Phase 2 complete

**Tasks**:
- [ ] Integrate comrak markdown parser (depends on: error handling)
  - Acceptance: Parse markdown string to AST
  - Test: Parse test markdown, verify structure

- [ ] Create PreviewPane widget for rendered output (depends on: parser, theme)
  - Acceptance: Displays formatted headers, paragraphs, lists
  - Test: Render test document

- [ ] Implement real-time preview updates (depends on: PreviewPane, editor)
  - Acceptance: Preview updates as user types
  - Test: Type markdown, see live update

- [ ] Implement ViewMode enum and toggle (depends on: PreviewPane)
  - Acceptance: Switch between Edit, Preview, Split modes
  - Test: Toggle modes, verify layout

- [ ] Add Alt+V keyboard shortcut for view toggle (depends on: ViewMode)
  - Acceptance: Alt+V cycles view modes
  - Test: Shortcut behavior

- [ ] Integrate syntect for code block highlighting (depends on: PreviewPane, theme)
  - Acceptance: Fenced code blocks show syntax colors
  - Test: Code block with language tag

- [ ] Implement table rendering in preview (depends on: parser)
  - Acceptance: Markdown tables display as formatted tables
  - Test: Table markdown renders correctly

- [ ] Implement clickable hyperlinks (depends on: PreviewPane)
  - Acceptance: Links show hover state, click opens browser
  - Test: Click link, verify browser opens

- [ ] Persist view mode preference (depends on: ViewMode, config)
  - Acceptance: View mode restores on restart
  - Test: Set mode, restart, verify

**Exit Criteria**: Full CommonMark rendering with live preview

**Delivers**: Markdown editor with split view

---

### Phase 4: Theme System
**Goal**: Light/dark themes with switching

**Entry Criteria**: Phase 3 complete

**Tasks**:
- [ ] Define Theme struct with colors, fonts, spacing (depends on: none)
  - Acceptance: Theme struct covers all UI elements
  - Test: Struct instantiation

- [ ] Create light theme with professional colors (depends on: Theme struct)
  - Acceptance: Readable light palette
  - Test: Visual inspection

- [ ] Create dark theme with comfortable contrast (depends on: Theme struct)
  - Acceptance: Eye-friendly dark palette
  - Test: Visual inspection

- [ ] Implement ThemeManager for switching (depends on: light, dark, config)
  - Acceptance: Can switch themes, change applies instantly
  - Test: Switch theme, verify colors change

- [ ] Add theme selector dropdown in UI (depends on: ThemeManager)
  - Acceptance: Dropdown shows available themes
  - Test: Select theme via dropdown

- [ ] Persist theme preference (depends on: ThemeManager, config)
  - Acceptance: Theme restores on restart
  - Test: Set theme, restart, verify

**Exit Criteria**: Working theme system with persistence

**Delivers**: Customizable appearance

---

### Phase 5: Polish & Completion
**Goal**: UI refinements and remaining MVP features

**Entry Criteria**: Phase 4 complete

**Tasks**:
- [ ] Add line number display to editor (depends on: editor, theme)
  - Acceptance: Line numbers show alongside content
  - Test: Type multiline, verify numbers

- [ ] Add line number toggle in settings (depends on: line numbers, config)
  - Acceptance: Can hide/show line numbers
  - Test: Toggle setting, verify

- [ ] Implement word/character/line statistics (depends on: editor)
  - Acceptance: Accurate counts update in real-time
  - Test: Known text, verify counts

- [ ] Create status bar widget (depends on: state, theme, stats)
  - Acceptance: Shows file path, line:col, stats, theme
  - Test: Visual inspection

- [ ] Implement recent files tracking (depends on: config, file operations)
  - Acceptance: Last 10 files stored with timestamps
  - Test: Open files, check recent list

- [ ] Add recent files menu (depends on: recent files tracking)
  - Acceptance: Menu shows recent files, click opens
  - Test: Click recent file, verify opens

- [ ] Update window title with current file (depends on: tabs, file operations)
  - Acceptance: Title shows "App Name - filename.md"
  - Test: Open file, check title

- [ ] Add smooth transitions between view modes (depends on: ViewMode)
  - Acceptance: View changes animate smoothly
  - Test: Visual inspection

- [ ] Cross-platform testing and fixes (depends on: all features)
  - Acceptance: Works on Windows, macOS, Linux
  - Test: Test on each platform

**Exit Criteria**: Polished MVP ready for use

**Delivers**: Complete Phase 1 MVP per original PRD

</implementation-roadmap>

---

<test-strategy>

## Test Pyramid

```
        /\
       /E2E\       ← 10% (Full workflow tests)
      /------\
     /Integration\ ← 30% (Module interactions)
    /------------\
   /  Unit Tests  \ ← 60% (Fast, isolated)
  /----------------\
```

## Coverage Requirements
- Line coverage: 70% minimum
- Function coverage: 80% minimum for public APIs

## Critical Test Scenarios

### File Operations
**Happy path**:
- Open existing .md file, content loads correctly
- Save file, content matches on disk
- New file creates empty document

**Edge cases**:
- Open very large file (10MB+)
- File with unusual encoding
- Empty file

**Error cases**:
- Open non-existent file
- Save to read-only location
- Permission denied

### Markdown Parser
**Happy path**:
- Headers (h1-h6) render correctly
- Bold, italic, strikethrough work
- Lists (ordered, unordered, nested)

**Edge cases**:
- Deeply nested lists
- Mixed markdown in single paragraph
- Unicode content

**Error cases**:
- Malformed table syntax
- Unclosed code block

### Tab System
**Happy path**:
- Create, switch, close tabs
- Tab state persists between switches

**Edge cases**:
- 50+ tabs open
- Rapid tab switching

**Error cases**:
- Close tab with corrupted content

</test-strategy>

---

<architecture>

## System Components

1. **eframe/egui Runtime**: Window management, event loop, rendering
2. **App State Manager**: Central state, event handling
3. **Editor Engine**: Text editing, cursor, selection
4. **Markdown Pipeline**: Parser → AST → Renderer
5. **Theme Engine**: Color management, style application
6. **File System Layer**: I/O operations, dialogs, watching

## Data Models

### Tab
```rust
struct Tab {
    id: usize,
    title: String,
    file_path: Option<PathBuf>,
    content: String,
    is_dirty: bool,
    cursor_position: usize,
    scroll_offset: f32,
}
```

### Settings
```rust
struct Settings {
    theme: ThemeName,
    view_mode: ViewMode,
    show_line_numbers: bool,
    recent_files: Vec<RecentFile>,
    window_size: (u32, u32),
    last_open_tabs: Vec<PathBuf>,
}
```

### Theme
```rust
struct Theme {
    name: String,
    background: Color32,
    foreground: Color32,
    accent: Color32,
    editor_bg: Color32,
    preview_bg: Color32,
    // ... more colors
}
```

## Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Language | Rust 1.70+ | Performance, safety, cross-platform |
| GUI | egui 0.28 + eframe 0.28 | Immediate mode, fast, portable |
| Markdown | comrak 0.22 | CommonMark compliant, fast |
| Syntax | syntect 5.1 | Sublime Text highlighting engine |
| Config | serde + serde_json | Standard Rust serialization |
| Paths | dirs 5 | Platform-specific directories |
| Links | open 5 | Open URLs in default browser |
| File watch | notify 6 | Cross-platform file watching |

**Decision: egui over other GUI frameworks**
- **Rationale**: Immediate mode GUI is simpler, faster to iterate, smaller binary
- **Trade-offs**: Less native look than Qt/GTK, but consistent cross-platform
- **Alternatives considered**: Tauri (Electron-like size), Druid (less mature), GTK-rs (complex)

</architecture>

---

<risks>

## Technical Risks

**Risk**: egui text editing may not match native feel
- **Impact**: Medium - affects daily usability
- **Likelihood**: Medium
- **Mitigation**: Test early with real users, iterate on feedback
- **Fallback**: Custom text widget implementation if needed

**Risk**: Syntax highlighting performance on large files
- **Impact**: High - core feature
- **Likelihood**: Low (syntect is battle-tested)
- **Mitigation**: Lazy highlighting, viewport-only rendering
- **Fallback**: Disable highlighting for files > 1MB

**Risk**: Cross-platform file dialog inconsistencies
- **Impact**: Low - cosmetic issue
- **Likelihood**: Medium
- **Mitigation**: Test on all platforms, use rfd crate for native dialogs
- **Fallback**: Custom file browser widget

## Scope Risks

**Risk**: Feature creep beyond MVP
- **Impact**: High - delays launch
- **Likelihood**: Medium
- **Mitigation**: Strict phase gating, defer nice-to-haves to Phase 2+
- **Fallback**: Ship MVP without polish features if needed

## Dependency Risks

**Risk**: egui breaking changes in future versions
- **Impact**: Medium
- **Likelihood**: Low (0.28 is stable)
- **Mitigation**: Lock dependency versions, test upgrades in branch
- **Fallback**: Stay on current version

</risks>

---

<appendix>

## References
- [egui documentation](https://docs.rs/egui)
- [CommonMark spec](https://commonmark.org/)
- [comrak documentation](https://docs.rs/comrak)
- [syntect documentation](https://docs.rs/syntect)

## Glossary
- **CommonMark**: Standardized markdown specification
- **Immediate Mode GUI**: UI paradigm where widgets are recreated each frame
- **AST**: Abstract Syntax Tree - parsed representation of markdown

## Out of Scope (MVP)
- Cloud sync / collaborative editing
- Plugin system
- Mobile versions
- Vim/Emacs keybindings
- Custom fonts
- Command palette

## Cargo.toml Dependencies
```toml
[dependencies]
egui = "0.28"
eframe = "0.28"
comrak = "0.22"
syntect = "5.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dirs = "5"
notify = "6"
open = "5"
rfd = "0.14"  # Native file dialogs
```

</appendix>

