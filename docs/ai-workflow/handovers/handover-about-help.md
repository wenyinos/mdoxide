# Feature Handover - About/Help Dialog

## Overview

Add a proper About/Help dialog to the application with project information, GitHub link, and keyboard shortcuts reference.

---

## Environment

- **Project:** Sleek Markdown Editor (Rust + egui)
- **Path:** G:\DEV\markDownNotepad
- **Run:** `cargo run` to test changes

---

## Feature Requirements

### 1. About/Help Button in UI

**Location Options (choose one):**
- Status bar (bottom right) - Add a `?` or `ℹ️` icon button next to Settings
- Ribbon - Add to a Help/About section
- Settings panel - Add an "About" tab

**Recommended:** Status bar with `?` icon, keeping it simple and accessible.

### 2. About Dialog Content

The dialog should be a modal window (similar to Settings panel) containing:

#### Header Section
```
Sleek Markdown Editor
Version 0.1.0
A lightweight, fast markdown editor built in Rust
```

#### Links Section
```
🔗 GitHub: https://github.com/[username]/sleek-markdown-editor
📖 Documentation: [link to docs if hosted]
🐛 Report Issue: [GitHub issues link]
```

Use `open::that(url)` crate (already in dependencies) to open links in browser.

#### Keyboard Shortcuts Section

Display a formatted table/list of all shortcuts:

| Category | Shortcut | Action |
|----------|----------|--------|
| **File** | Ctrl+N | New File |
| | Ctrl+O | Open File |
| | Ctrl+S | Save |
| | Ctrl+Shift+S | Save As |
| | Ctrl+W | Close Tab |
| **Edit** | Ctrl+Z | Undo |
| | Ctrl+Y | Redo |
| | Ctrl+F | Find/Replace |
| | Ctrl+A | Select All |
| **View** | Ctrl+E | Toggle Raw/Rendered (or whatever new shortcut) |
| | Ctrl+Shift+O | Toggle Outline |
| | Ctrl++ / Ctrl+- | Zoom In/Out |
| **Formatting** | Ctrl+B | Bold |
| | Ctrl+I | Italic |
| | Ctrl+K | Insert Link |
| **Workspace** | Ctrl+P | Quick File Switcher |
| | Ctrl+Shift+F | Search in Files |
| **Navigation** | Ctrl+Tab | Next Tab |
| | Ctrl+Shift+Tab | Previous Tab |

#### Credits/License Section (Optional)
```
MIT License
© 2024 [Author Name]
Built with egui, comrak, syntect
```

### 3. Implementation Approach

**Option A: Simple Modal (Recommended)**
- Create new file `src/ui/about.rs`
- Add `AboutPanel` struct similar to `SettingsPanel`
- Toggle visibility with `show_about: bool` in AppState or App
- Render as modal overlay using `egui::Window` or `egui::Area`

**Option B: Tabbed in Settings**
- Add "About" tab to existing SettingsPanel
- Less code, reuses existing modal infrastructure

---

## Key Files

- `src/app.rs` - Add about button to status bar, handle toggle
- `src/ui/mod.rs` - Export new AboutPanel
- `src/ui/about.rs` - New file for About dialog (if Option A)
- `src/ui/settings.rs` - Add About tab (if Option B)
- `Cargo.toml` - Version info can be read from here

---

## Implementation Steps

1. **Create AboutPanel struct:**
```rust
pub struct AboutPanel {
    visible: bool,
}

impl AboutPanel {
    pub fn new() -> Self {
        Self { visible: false }
    }
    
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }
    
    pub fn show(&mut self, ctx: &egui::Context, theme_colors: &ThemeColors) {
        if !self.visible { return; }
        // Render modal window
    }
}
```

2. **Add to App struct:**
```rust
about_panel: AboutPanel,
```

3. **Add button to status bar** (near settings icon):
```rust
if ui.button("?").on_hover_text("About / Help (F1)").clicked() {
    self.about_panel.toggle();
}
```

4. **Add F1 keyboard shortcut** for quick access:
```rust
if input.key_pressed(Key::F1) {
    self.about_panel.toggle();
}
```

5. **Render the panel** at end of update loop:
```rust
self.about_panel.show(ctx, &theme_colors);
```

---

## Design Notes

- Match the visual style of Settings panel (same background, borders, fonts)
- Use `egui::ScrollArea` if content is long (especially shortcuts list)
- Make links clickable using `ui.hyperlink("https://...")` or custom button + `open::that()`
- Consider collapsible sections for shortcuts (by category)
- Version should be read from `env!("CARGO_PKG_VERSION")` macro

---

## Testing Checklist

- [ ] About button visible in status bar (or chosen location)
- [ ] F1 opens the About dialog
- [ ] Clicking outside or pressing Escape closes it
- [ ] All information displays correctly
- [ ] GitHub link opens in browser when clicked
- [ ] Shortcuts list is complete and accurate
- [ ] Dialog looks good in both Light and Dark themes
- [ ] `cargo build` passes

---

## Reference: Existing Patterns

Look at these for implementation patterns:
- `src/ui/settings.rs` - Modal panel pattern
- `src/ui/quick_switcher.rs` - Overlay/popup pattern
- `src/app.rs` - Status bar button rendering

@src/app.rs @src/ui/settings.rs @src/ui/mod.rs
