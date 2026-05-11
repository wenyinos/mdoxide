# Productivity Panel

A workspace-scoped sidebar panel that bundles three productivity tools: a
markdown-style task list, a Pomodoro timer, and quick notes. Lives at
`src/ui/productivity_panel.rs`.

## Modes

The panel renders in two modes that share a single `show_content()` body:

| Mode      | Trigger                                            | Container                                |
|-----------|----------------------------------------------------|------------------------------------------|
| Docked    | Default. Hub tab in the outline sidebar.            | `OutlinePanel` (see `outline_panel.rs`)  |
| Detached  | Click `⤴ Detach` from the Hub tab, or hotkey.       | `egui::Window` floated by `app/mod.rs`   |

The active mode is driven by two settings in `Settings`:
`productivity_panel_docked` and `productivity_panel_visible`.

### Switching between modes

- **Detach**: outline panel's `⤴ Detach` button → sets
  `productivity_panel_docked = false` and `productivity_panel_visible = true`,
  switches outline back to the Outline tab.
- **Dock**: floating window's `⤵ Dock` button → sets
  `productivity_panel_docked = true`, `productivity_panel_visible = false`,
  re-enables the outline panel and selects the Productivity tab.
- **Floating window `×` (close)**: behaves identically to Dock. The window's
  title-bar close button never hides the panel outright; instead it routes
  back into the sidebar so the panel never becomes unreachable mid-session.
  See `ProductivityPanel::show()` for the implementation
  (`was_visible && !*visible` → `dock_requested = true`).

## Visual Design

Each subsection is wrapped in a themed "card":

- 1 px border in `card_border` (light: #DADEE4, dark: #3C3E46)
- 6 px rounded corners
- Subtle translucent fill (`card_bg`) so cards lift off the panel background
- 10×8 px inner margin
- Card header: bold 13 px label with a leading icon (`✓`, `⏱`, `📝`); optional
  right-aligned badge (e.g. task progress `3/5`, Pomodoro cycle count)

Colors are derived from `ui.visuals()` so the panel inherits the active theme
and adapts automatically to light/dark mode.

### Tasks

- Single-line input + `Add` button (Enter also adds)
- Tip line uses muted italic 10 px text
- Each row alternates background tint for readability (`row_alt_bg`)
- Priority chips: small bordered pill with semi-transparent fill
  (`!` warn / `!!` danger), instead of plain text markers
- Reorder (`▲` `▼`) and delete (`✕`) buttons are frame-less and use
  `weak_text_color` so they recede until hovered

### Pomodoro

- Large centered monospace timer (`34 px`), color tracks state
  (accent for Work, success for Break, primary text when Idle)
- Status label below timer ("Work session" / "Break" / "Ready")
- Action buttons share the row width: split 50/50 when idle (▶ Start Work,
  ☕ Start Break), full-width when active (⏹ Stop)
- Cycle count badge in the card header

### Notes

- Combo box selector + inline icon actions (`➕` new, `✏` rename, `🗑` delete)
- Inline rename input replaces the row when editing
- Multiline text area auto-saves on a 1 s debounce via `AutoSave`

## Persistence

- Tasks: `<workspace>/.ferrite/tasks.json` (atomic write via `.bak` rename)
- Notes: `<workspace>/.ferrite/notes/<name>.txt`
- Without an open workspace the panel shows a yellow info banner and runs
  in-memory only (no save). All state is reset on workspace switch via
  `set_workspace()`.

## Repaint Behavior

`show_content()` returns `true` while the Pomodoro timer is active so the
caller (outline or floating window) can call `request_repaint_after(1s)`.
This keeps the panel idle-cheap when no timer is running.

## Sizing & Layout (v0.3.0)

The Hub now keeps a stable size in both modes. Two fixes worth knowing about:

### Docked panel — outer footprint lock

`egui::SidePanel` stores the rendered content's `min_rect` in `PanelState` at
the end of every frame:

```rust
// from egui::containers::panel::SidePanel::show_inside_dyn
let inner_response = frame.show(&mut panel_ui, |ui| { add_contents(ui) });
let rect = inner_response.response.rect;     // = content_ui.min_rect() + margin
PanelState { rect }.store(ui.ctx(), id);
```

That means *any* widget reporting a wider preferred size (an unwrapped label,
a `desired_width(f32::INFINITY)` text edit, a button with long text, etc.)
permanently grew the sidebar — and the user could no longer shrink it,
because the next frame's content overflowed again and re-wrote the larger
width into `PanelState`.

To stop content overflow from leaking into `PanelState`, `OutlinePanel`
allocates a strict rectangle and hosts productivity content inside a clipped
child UI:

```rust
// src/ui/outline_panel.rs (Productivity tab branch)
let avail_w = ui.available_width();
let avail_h = ui.available_height();
let (content_rect, _) = ui.allocate_exact_size(
    Vec2::new(avail_w, avail_h),
    Sense::hover(),
);
let mut child_ui = ui.child_ui(content_rect, Layout::top_down(Align::Min), None);
child_ui.set_clip_rect(content_rect);
child_ui.set_max_width(avail_w);
ScrollArea::vertical()
    .auto_shrink([false, false])
    .show(&mut child_ui, |ui| { panel.show_content(ui, ctx); });
```

`child_ui` allocations do not propagate back to the parent, so the panel's
final `min_rect` is exactly `avail_w × avail_h`. `PanelState` always stores
the user's chosen width and resize is honored.

The minimum sidebar width remains **260 px** (`MIN_PANEL_WIDTH` in
`src/ui/outline_panel.rs`, mirrored by `Settings::MIN_OUTLINE_WIDTH`) — the
floor required for the five tab labels (Outline / Stats / Links / FM / Hub)
to stay readable.

### Floating window — capped auto-resize

`egui::containers::Resize` runs this every frame when the user is not
actively dragging the corner:

```rust
state.desired_size = state.desired_size.max(state.last_content_size);
```

That makes a floating `Window` grow to fit content but never shrink. Combined
with `desired_width(f32::INFINITY)` on the notes textarea, the window
expanded without bound. The fix:

- `default_size([dock_width, 540])` — opens at the current sidebar width on
  first detach, no visual jump.
- `max_size([dock_width.max(560), screen_h - 60])` — hard ceiling for the
  auto-resize loop.
- Notes textarea uses `desired_width(ui.available_width())` instead of
  `f32::INFINITY`.

### Cursor flicker at the panel edge

Scrollbars previously overlapped the `SidePanel` resize grab radius, causing
the OS cursor to flicker between *resize* and *normal* arrows. Mitigated app-
wide with:

```rust
// src/app/mod.rs (CreationContext setup)
style.spacing.scroll.bar_outer_margin = 6.0;
```
