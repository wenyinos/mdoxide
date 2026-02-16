# Special Tabs System

Special tabs render application UI panels (Settings, About/Help) inline within the main tab bar, similar to how VS Code and Cursor handle their settings. This replaces the previous modal window approach, giving panels more screen real estate and a more integrated feel.

## Architecture

### Types

```
TabKind::Document                    # Normal file editing tab (default)
TabKind::Special(SpecialTabKind)     # Non-editable UI panel tab
```

**`SpecialTabKind`** enum (in `state.rs`):
- `Settings` - Application settings panel
- `About` - About/Help information panel

Designed to be extensible - add new variants for future panels (e.g., Extensions, Themes, Keybindings standalone).

### Key Properties

| Property | Document Tab | Special Tab |
|----------|-------------|-------------|
| Editable | Yes | No |
| View mode selector | Shown | Hidden |
| Auto-save indicator | Shown | Hidden |
| `is_modified()` | Checks content | Always `false` |
| `should_prompt_to_save()` | Checks content | Always `false` |
| Session persistence | Saved/restored | Excluded |
| Tab title | Filename (+ `*` if modified) | Icon + panel name |
| `Ctrl+S` | Saves file | No-op |

### Single Instance

Only one tab per `SpecialTabKind` can exist at a time. `AppState::open_special_tab()` checks for an existing tab of that kind and focuses it instead of creating a duplicate.

## Code Locations

| What | Where |
|------|-------|
| `TabKind`, `SpecialTabKind` enums | `src/state.rs` |
| `Tab.kind` field | `src/state.rs` (Tab struct) |
| `Tab::is_special()` helper | `src/state.rs` |
| `AppState::open_special_tab()` | `src/state.rs` |
| `AppState::open_settings_tab()` | `src/state.rs` |
| `AppState::toggle_about()` | `src/state.rs` |
| Special tab content rendering | `src/app/central_panel.rs` → `render_special_tab_content()` |
| Settings inline rendering | `src/ui/settings.rs` → `SettingsPanel::show_inline()` |
| About inline rendering | `src/ui/about.rs` → `AboutPanel::show_inline()` |
| Title bar special tab checks | `src/app/title_bar.rs` |
| Save guard for special tabs | `src/app/file_ops.rs` |

## Rendering Flow

```
render_central_panel()
  ├── Render tab bar (all tabs, including special)
  ├── Check active_tab.kind
  │   ├── TabKind::Special(kind) → render_special_tab_content(ui, kind)
  │   │   ├── Settings → settings_panel.show_inline(ui, settings, is_dark)
  │   │   └── About → about_panel.show_inline(ui, is_dark)
  │   └── TabKind::Document → (normal editor rendering)
```

## Inline Panel Layout

Both `show_inline()` methods use a consistent layout:

```
┌──────────────────────────────────────────────┐
│  Sidebar (140-160px)  │  Content (fills rest) │
│  ┌──────────────┐     │  ┌──────────────────┐ │
│  │ Panel Title  │     │  │                  │ │
│  │              │     │  │  ScrollArea with  │ │
│  │ ○ Section 1  │     │  │  section content  │ │
│  │ ● Section 2  │  │  │  │                  │ │
│  │ ○ Section 3  │     │  │                  │ │
│  │              │     │  │                  │ │
│  │ ─────────── │     │  │                  │ │
│  │ [Reset All] │     │  │                  │ │
│  └──────────────┘     │  └──────────────────┘ │
└──────────────────────────────────────────────┘
```

## Adding a New Special Tab

1. Add variant to `SpecialTabKind` in `state.rs`:
   ```rust
   pub enum SpecialTabKind {
       Settings,
       About,
       MyNewPanel,  // Add here
   }
   ```

2. Update `title()` and `icon()` on `SpecialTabKind`.

3. Add a `show_inline()` method to your panel struct.

4. Add the rendering case in `render_special_tab_content()` in `central_panel.rs`.

5. Add a trigger (keyboard shortcut, button, etc.) that calls `open_special_tab(SpecialTabKind::MyNewPanel)`.

## Previous Approach (Modal)

Before v0.2.7, settings and about panels were modal windows:
- Rendered via `egui::Window` with semi-transparent overlay
- Controlled by `UiState.show_settings` / `UiState.show_about` flags
- Fixed size, centered on screen
- The old `SettingsPanel::show()` and `AboutPanel::show()` methods still exist but are unused

The modal approach limited screen space and felt disconnected from the tab-based workflow.
