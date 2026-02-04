# Phase 7: Productivity Hub - Research

**Researched:** 2026-01-24
**Domain:** Task management, Pomodoro timer, note-taking with local persistence
**Confidence:** HIGH

## Summary

Phase 7 adds a productivity hub with task tracking, Pomodoro timer, and quick notes. Research reveals the project already has strong foundations: comrak (the existing markdown parser) supports GFM task lists natively, chrono is already a dependency, and the project has established patterns for JSON persistence and egui UI.

The standard approach is:
- Parse markdown checkboxes using comrak's existing GFM extension (already in Cargo.toml)
- Use `std::time::Instant` for Pomodoro countdown (not chrono - avoids clock adjustment issues)
- Reuse existing sound notification system from `src/terminal/sound.rs`
- Follow established persistence patterns from `src/config/persistence.rs` for `.ferrite/` storage
- Use egui's immediate mode with `request_repaint_after()` for timer updates

**Primary recommendation:** Build on existing infrastructure. No new dependencies needed except potentially `egui_dock` if complex tab layout is desired (currently not needed for simple sections).

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| comrak | 0.22 | Markdown parsing with GFM task lists | Already in project, supports task list extension natively |
| std::time::Instant | stdlib | Countdown timer (not chrono) | Immune to system clock changes, perfect for elapsed time |
| serde/serde_json | 1.x | JSON serialization for persistence | Already in project, standard Rust serialization |
| egui | 0.28 | UI rendering | Project's UI framework, supports immediate mode timers |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| chrono | 0.4 | Date formatting for timestamps | Already in project, use for "last saved" timestamps only |
| egui_dock | 0.x | Advanced tab/docking layout | Optional - only if complex docking needed (not required for this phase) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| std::time::Instant | chrono::DateTime | chrono affected by system clock changes (DST, manual adjustments) - bad for timers |
| comrak GFM extension | pulldown-cmark + custom parser | More work, comrak already handles GFM task lists |
| Simple egui sections | egui_dock tabs | More complexity, not needed for this phase's simple layout |

**Installation:**
No new dependencies required - all needed crates already in Cargo.toml.

## Architecture Patterns

### Recommended Project Structure
```
src/
├── ui/
│   └── productivity_panel.rs   # Main panel (tasks + timer + notes in sections)
└── .ferrite/                   # Workspace-scoped storage
    ├── tasks.json              # Task list per workspace
    └── notes/                  # Notes directory
        ├── default.txt         # Default note
        └── [name].txt          # Named notes
```

### Pattern 1: Task Checkbox Parsing and Rendering
**What:** Parse markdown checkbox syntax in real-time, render as interactive checkboxes
**When to use:** For task input field
**Example:**
```rust
// Enable GFM task list extension in comrak (already available)
let mut options = comrak::Options::default();
options.extension.tasklist = true;
options.render.unsafe_ = true; // Required for HTML input elements

// Parse markdown with task lists
// Input: "- [ ] Task one\n- [x] Task two"
// Output: Renders checkboxes, maintains state

// For egui rendering, parse tasks manually:
struct Task {
    completed: bool,
    text: String,
    priority: u8, // 0=none, 1=!, 2=!!
}

impl Task {
    fn from_markdown(line: &str) -> Option<Self> {
        // Match: "- [ ] task" or "- [x] task"
        let re = regex::Regex::new(r"^- \[([ x])\] (.+)$").unwrap();
        if let Some(caps) = re.captures(line.trim()) {
            let completed = &caps[1] == "x";
            let mut text = caps[2].to_string();

            // Extract priority
            let priority = if text.starts_with("!! ") {
                text = text[3..].to_string();
                2
            } else if text.starts_with("! ") {
                text = text[2..].to_string();
                1
            } else {
                0
            };

            Some(Task { completed, text, priority })
        } else {
            None
        }
    }

    fn to_markdown(&self) -> String {
        let checkbox = if self.completed { "[x]" } else { "[ ]" };
        let priority_str = match self.priority {
            2 => "!! ",
            1 => "! ",
            _ => "",
        };
        format!("- {} {}{}", checkbox, priority_str, self.text)
    }
}
```

### Pattern 2: Pomodoro Timer with std::time::Instant
**What:** Countdown timer immune to system clock changes
**When to use:** For Pomodoro work/break cycles
**Example:**
```rust
use std::time::{Duration, Instant};

#[derive(Clone)]
enum TimerState {
    Idle,
    Work { started: Instant, duration: Duration },
    Break { started: Instant, duration: Duration },
    Paused { state: Box<TimerState>, elapsed: Duration },
}

impl TimerState {
    fn start_work() -> Self {
        TimerState::Work {
            started: Instant::now(),
            duration: Duration::from_secs(25 * 60), // 25 minutes
        }
    }

    fn remaining(&self) -> Option<Duration> {
        match self {
            TimerState::Work { started, duration } |
            TimerState::Break { started, duration } => {
                let elapsed = started.elapsed();
                duration.checked_sub(elapsed)
            }
            TimerState::Paused { elapsed, .. } => Some(*elapsed),
            TimerState::Idle => None,
        }
    }

    fn is_complete(&self) -> bool {
        self.remaining().map_or(false, |r| r.as_secs() == 0)
    }
}

// In egui update():
if matches!(timer_state, TimerState::Work { .. } | TimerState::Break { .. }) {
    // Request repaint every second for countdown display
    ctx.request_repaint_after(Duration::from_secs(1));

    if timer_state.is_complete() {
        // Play sound notification
        sound_notifier.play_notification(None);
        // Transition to break or idle
    }
}
```
Source: [Rust Cookbook Duration Calculation](https://rust-lang-nursery.github.io/rust-cookbook/datetime/duration.html)

### Pattern 3: Auto-Save with Debouncing
**What:** Debounced writes to prevent excessive I/O on every keystroke
**When to use:** For notes panel auto-save
**Example:**
```rust
struct AutoSave {
    last_edit: Instant,
    debounce_duration: Duration,
    pending_content: Option<String>,
}

impl AutoSave {
    fn new(debounce_ms: u64) -> Self {
        Self {
            last_edit: Instant::now(),
            debounce_duration: Duration::from_millis(debounce_ms),
            pending_content: None,
        }
    }

    fn mark_edited(&mut self, content: String) {
        self.last_edit = Instant::now();
        self.pending_content = Some(content);
    }

    fn should_save(&self) -> bool {
        self.pending_content.is_some()
            && self.last_edit.elapsed() >= self.debounce_duration
    }

    fn take_pending(&mut self) -> Option<String> {
        self.pending_content.take()
    }
}

// In egui update():
if notes_text_changed {
    auto_save.mark_edited(notes_text.clone());
}

if auto_save.should_save() {
    if let Some(content) = auto_save.take_pending() {
        // Write to .ferrite/notes/[name].txt
        save_note(&note_name, &content);
    }
}
```
Recommended debounce: 1000ms (1 second) based on [RustRover auto-save patterns](https://www.jetbrains.com/help/rust/saving-and-reverting-changes.html)

### Pattern 4: Workspace-Scoped Persistence
**What:** Store tasks/notes per workspace directory, following existing .ferrite/ pattern
**When to use:** For all productivity data
**Example:**
```rust
// Following pattern from src/config/persistence.rs

fn get_workspace_ferrite_dir(workspace_path: &Path) -> PathBuf {
    workspace_path.join(".ferrite")
}

fn save_tasks(workspace_path: &Path, tasks: &[Task]) -> Result<()> {
    let ferrite_dir = get_workspace_ferrite_dir(workspace_path);
    fs::create_dir_all(&ferrite_dir)?;

    let tasks_path = ferrite_dir.join("tasks.json");
    let json = serde_json::to_string_pretty(tasks)?;

    // Atomic write pattern (from persistence.rs)
    let backup_path = ferrite_dir.join("tasks.json.bak");
    fs::write(&backup_path, &json)?;
    fs::rename(&backup_path, &tasks_path)?;

    Ok(())
}

fn load_tasks(workspace_path: &Path) -> Vec<Task> {
    let tasks_path = get_workspace_ferrite_dir(workspace_path).join("tasks.json");

    if !tasks_path.exists() {
        return Vec::new();
    }

    let contents = fs::read_to_string(&tasks_path).ok()?;
    serde_json::from_str(&contents).unwrap_or_default()
}
```
Source: Established pattern from `src/config/persistence.rs` (atomic writes with .bak files)

### Pattern 5: Simple Section Layout (No Tabs Needed)
**What:** Use vertical sections within a single panel (simpler than tabs)
**When to use:** For this phase's requirements (3 sections: tasks, timer, notes)
**Example:**
```rust
egui::Window::new("Productivity Hub")
    .open(&mut settings.productivity_panel_visible)
    .show(ctx, |ui| {
        // Section 1: Tasks
        ui.heading("Tasks");
        egui::ScrollArea::vertical()
            .id_source("tasks_scroll")
            .max_height(200.0)
            .show(ui, |ui| {
                // Render task checkboxes
            });

        ui.separator();

        // Section 2: Pomodoro Timer
        ui.heading("Pomodoro Timer");
        ui.horizontal(|ui| {
            // Timer display and controls
        });

        ui.separator();

        // Section 3: Quick Notes
        ui.heading("Quick Notes");
        ui.text_edit_multiline(&mut notes_text);
    });
```
Note: egui_dock is available if complex docking is needed later, but simple vertical sections are sufficient for this phase. Source: [egui discussions on tab patterns](https://github.com/emilk/egui/discussions/1912)

### Anti-Patterns to Avoid
- **Using chrono for countdown timers:** System clock changes break timers. Use `std::time::Instant` instead.
- **Saving on every keystroke:** Causes excessive I/O. Always debounce (1s recommended).
- **Global task storage:** Tasks must be workspace-scoped per requirements.
- **Blocking file I/O in update():** Keep saves fast, consider background thread for large files (though not needed here).
- **Complex AST parsing for checkboxes:** Simple regex is sufficient for the limited syntax needed.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Markdown checkbox parsing | Custom regex parser for all GFM | comrak with tasklist extension | Handles edge cases, already in project |
| Timer countdown logic | Manual tick counting | std::time::Instant + elapsed() | Handles system sleep, more reliable |
| JSON persistence | Custom file format | serde_json (already used) | Handles escaping, versioning, schema evolution |
| Atomic file writes | Direct fs::write() | Write to .bak then rename (existing pattern) | Prevents corruption on crash/disk full |
| Debouncing | Custom timer logic | Instant + duration comparison | Simple, proven pattern |

**Key insight:** Rust's stdlib provides excellent time primitives (`std::time::Instant`) that are specifically designed for elapsed time measurement and are immune to system clock changes. This is superior to chrono for countdown timers.

## Common Pitfalls

### Pitfall 1: Using chrono for Countdown Timers
**What goes wrong:** Pomodoro timer jumps or becomes negative when system clock changes (DST, manual adjustment)
**Why it happens:** `chrono::DateTime` is based on wall-clock time, which can go backwards
**How to avoid:** Use `std::time::Instant` which measures elapsed time from a fixed point, immune to clock changes
**Warning signs:** Timer showing negative values, sudden jumps in countdown
**Source:** [Rust Cookbook - Measure Time Elapsed](https://www.simonwenkel.com/notes/programming_languages/rust/measure_time_elapsed_with_rust.html)

### Pitfall 2: No Debouncing on Auto-Save
**What goes wrong:** Excessive file writes on every keystroke cause UI lag and wear on SSDs
**Why it happens:** egui's immediate mode runs update() every frame (60+ FPS)
**How to avoid:** Track last edit time, only save after 1 second of inactivity
**Warning signs:** High disk I/O, frame drops when typing in notes
**Source:** [Auto-save debounce patterns](https://www.synthace.com/blog/autosave-with-react-hooks)

### Pitfall 3: Forgetting request_repaint_after() for Timers
**What goes wrong:** Timer doesn't update on screen, appears frozen
**Why it happens:** egui's reactive mode only repaints on input events by default
**How to avoid:** Call `ctx.request_repaint_after(Duration::from_secs(1))` when timer is active
**Warning signs:** Timer only updates when mouse moves or keys are pressed
**Source:** [egui Issue #295 - Timer Callbacks](https://github.com/emilk/egui/issues/295)

### Pitfall 4: Global Task Storage Instead of Workspace-Scoped
**What goes wrong:** Tasks appear in wrong workspace, or tasks from multiple workspaces mix
**Why it happens:** Storing in global config directory instead of `.ferrite/` per workspace
**How to avoid:** Always use `workspace_path.join(".ferrite/tasks.json")`
**Warning signs:** Tasks persisting across different workspace folders

### Pitfall 5: Parsing Markdown on Every Frame
**What goes wrong:** CPU usage spikes, performance degrades with large task lists
**Why it happens:** Re-parsing entire task list on every egui frame (60+ FPS)
**How to avoid:** Parse once when loading, maintain Vec<Task> in memory, only re-parse on edits
**Warning signs:** High CPU usage when productivity panel is open, frame drops with many tasks

### Pitfall 6: Sound Notification Blocking UI Thread
**What goes wrong:** UI freezes when Pomodoro timer completes
**Why it happens:** Sound playback runs synchronously on main thread
**How to avoid:** Existing `src/terminal/sound.rs` spawns processes correctly - reuse it
**Warning signs:** Brief UI freeze when timer hits zero

## Code Examples

Verified patterns from established sources:

### Pomodoro Timer State Machine
```rust
// Complete timer implementation following std::time best practices
use std::time::{Duration, Instant};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PomodoroTimer {
    state: TimerState,
    work_duration_secs: u64,    // Default: 25 * 60
    break_duration_secs: u64,   // Default: 5 * 60
    completed_cycles: usize,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
enum TimerState {
    Idle,
    #[serde(skip)]  // Don't serialize Instant
    Work { started: Instant, duration_secs: u64 },
    #[serde(skip)]
    Break { started: Instant, duration_secs: u64 },
}

impl Default for TimerState {
    fn default() -> Self {
        TimerState::Idle
    }
}

impl PomodoroTimer {
    pub fn new() -> Self {
        Self {
            state: TimerState::Idle,
            work_duration_secs: 25 * 60,
            break_duration_secs: 5 * 60,
            completed_cycles: 0,
        }
    }

    pub fn start_work(&mut self) {
        self.state = TimerState::Work {
            started: Instant::now(),
            duration_secs: self.work_duration_secs,
        };
    }

    pub fn start_break(&mut self) {
        self.state = TimerState::Break {
            started: Instant::now(),
            duration_secs: self.break_duration_secs,
        };
    }

    pub fn stop(&mut self) {
        self.state = TimerState::Idle;
    }

    pub fn remaining(&self) -> Option<Duration> {
        match &self.state {
            TimerState::Work { started, duration_secs } |
            TimerState::Break { started, duration_secs } => {
                let total = Duration::from_secs(*duration_secs);
                total.checked_sub(started.elapsed())
            }
            TimerState::Idle => None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.remaining().map_or(false, |r| r.as_secs() == 0)
    }

    pub fn format_remaining(&self) -> String {
        if let Some(remaining) = self.remaining() {
            let mins = remaining.as_secs() / 60;
            let secs = remaining.as_secs() % 60;
            format!("{:02}:{:02}", mins, secs)
        } else {
            "00:00".to_string()
        }
    }

    pub fn is_work(&self) -> bool {
        matches!(self.state, TimerState::Work { .. })
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state, TimerState::Idle)
    }
}
```
Source: [std::time::Instant docs](https://doc.rust-lang.org/std/time/struct.Instant.html)

### Task Persistence with Atomic Writes
```rust
// Following the pattern from src/config/persistence.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub completed: bool,
    pub text: String,
    pub priority: u8,  // 0=none, 1=!, 2=!!
}

pub fn save_tasks(workspace_path: &Path, tasks: &[Task]) -> std::io::Result<()> {
    let ferrite_dir = workspace_path.join(".ferrite");
    fs::create_dir_all(&ferrite_dir)?;

    let tasks_path = ferrite_dir.join("tasks.json");
    let backup_path = ferrite_dir.join("tasks.json.bak");

    // Serialize to pretty JSON
    let json = serde_json::to_string_pretty(tasks)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    // Atomic write: write to backup, then rename
    fs::write(&backup_path, &json)?;
    fs::rename(&backup_path, &tasks_path)?;

    Ok(())
}

pub fn load_tasks(workspace_path: &Path) -> Vec<Task> {
    let tasks_path = workspace_path.join(".ferrite").join("tasks.json");

    if !tasks_path.exists() {
        return Vec::new();
    }

    let contents = match fs::read_to_string(&tasks_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    serde_json::from_str(&contents).unwrap_or_default()
}
```
Source: Adapted from `src/config/persistence.rs` atomic write pattern

### egui Panel Layout with Timer Updates
```rust
// In src/ui/productivity_panel.rs
use eframe::egui;
use std::time::Duration;

pub fn show_productivity_panel(
    ctx: &egui::Context,
    visible: &mut bool,
    timer: &mut PomodoroTimer,
    tasks: &mut Vec<Task>,
    notes: &mut String,
    auto_save: &mut AutoSave,
) {
    egui::Window::new("Productivity Hub")
        .open(visible)
        .default_width(400.0)
        .show(ctx, |ui| {
            // Tasks section
            ui.heading("Tasks");
            egui::ScrollArea::vertical()
                .id_source("tasks_scroll")
                .max_height(200.0)
                .show(ui, |ui| {
                    for (i, task) in tasks.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.checkbox(&mut task.completed, "").changed() {
                                // Mark for save
                            }

                            let style = if task.completed {
                                egui::TextStyle::Body  // Will be styled with strikethrough
                            } else {
                                egui::TextStyle::Body
                            };

                            let priority_icon = match task.priority {
                                2 => "‼ ",
                                1 => "! ",
                                _ => "",
                            };

                            ui.label(format!("{}{}", priority_icon, task.text));
                        });
                    }
                });

            ui.separator();

            // Pomodoro section
            ui.heading("Pomodoro Timer");
            ui.horizontal(|ui| {
                ui.label(timer.format_remaining());

                if timer.is_active() {
                    if ui.button("Stop").clicked() {
                        timer.stop();
                    }

                    // Critical: Request repaint for countdown
                    ctx.request_repaint_after(Duration::from_secs(1));

                    // Check if complete
                    if timer.is_complete() {
                        // Play sound (reuse terminal sound system)
                        play_pomodoro_sound();

                        // Transition to break or idle
                        if timer.is_work() {
                            timer.start_break();
                        } else {
                            timer.stop();
                        }
                    }
                } else {
                    if ui.button("Start Work (25m)").clicked() {
                        timer.start_work();
                    }
                    if ui.button("Start Break (5m)").clicked() {
                        timer.start_break();
                    }
                }
            });

            ui.separator();

            // Notes section
            ui.heading("Quick Notes");
            let response = ui.add(
                egui::TextEdit::multiline(notes)
                    .desired_rows(10)
                    .hint_text("Type your notes here...")
            );

            if response.changed() {
                auto_save.mark_edited(notes.clone());
            }

            // Auto-save check (non-blocking)
            if auto_save.should_save() {
                if let Some(content) = auto_save.take_pending() {
                    save_note(&content);
                }
            }
        });
}
```
Source: [egui request_repaint_after pattern](https://github.com/emilk/egui/issues/1691)

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| chrono for all time operations | std::time::Instant for elapsed time, chrono for wall-clock only | Rust 1.8+ (2016) | Timers immune to clock changes |
| Save on every edit | Debounced auto-save (1s) | Common since ~2020 | Reduced I/O, better performance |
| Custom markdown parsers | GFM-compliant parsers (comrak) | comrak 0.7+ (2020) | Standards compliance, less maintenance |
| egui polling for updates | request_repaint_after() | egui 0.18+ (2022) | Efficient reactive updates |
| Direct file writes | Atomic write-and-rename | POSIX standard practice | Prevents corruption |

**Deprecated/outdated:**
- **Chrono for countdown timers:** Still works but wrong tool. Use `std::time::Instant` (recommendation from Rust community since ~2019)
- **Blocking file I/O on main thread for large files:** Not an issue for this phase (small JSON files), but worth noting for future
- **pulldown-cmark without GFM extensions:** comrak is more complete for GFM (GitHub Flavored Markdown)

## Open Questions

Things that couldn't be fully resolved:

1. **Pomodoro sound selection**
   - What we know: `src/terminal/sound.rs` already implements cross-platform sound playback
   - What's unclear: Should productivity hub reuse terminal's sound settings, or have its own?
   - Recommendation: Reuse terminal sound system initially, add separate settings if user feedback requests it

2. **Task reordering mechanism**
   - What we know: Context says "manual ordering (no auto-grouping)"
   - What's unclear: Drag-and-drop in egui requires more complex state management
   - Recommendation: Start with up/down arrow buttons (simpler), add drag-and-drop if requested

3. **Multiple notes implementation details**
   - What we know: Context says "create and switch between named notes"
   - What's unclear: UI pattern - dropdown? Tab bar? List?
   - Recommendation: Simple dropdown (less screen space) or list on left side if space permits

## Sources

### Primary (HIGH confidence)
- comrak 0.22 documentation - Task list extension confirmed
- std::time::Instant Rust stdlib docs - Timer best practices
- Existing codebase patterns:
  - `src/config/persistence.rs` - Atomic write pattern
  - `src/terminal/sound.rs` - Cross-platform sound playback
  - `Cargo.toml` - Confirmed dependencies (comrak, chrono, serde_json, egui 0.28)

### Secondary (MEDIUM confidence)
- [comrak GFM Extension](https://docs.rs/comrak/latest/comrak/options/struct.Extension.html)
- [Rust Cookbook - Duration Calculation](https://rust-lang-nursery.github.io/rust-cookbook/datetime/duration.html)
- [egui Timer Patterns](https://github.com/emilk/egui/issues/295)
- [egui request_repaint_after](https://github.com/emilk/egui/issues/1691)
- [Measure Time Elapsed with Rust](https://www.simonwenkel.com/notes/programming_languages/rust/measure_time_elapsed_with_rust.html)

### Tertiary (LOW confidence - marked for validation)
- [Auto-save with React Hooks](https://www.synthace.com/blog/autosave-with-react-hooks) - Pattern applicable to Rust but from different ecosystem
- [egui Tab Discussions](https://github.com/emilk/egui/discussions/1912) - Community discussions, not official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All dependencies already in project, verified in Cargo.toml
- Architecture: HIGH - Patterns verified from existing codebase (persistence.rs, sound.rs)
- Pitfalls: MEDIUM - Based on community best practices and Rust documentation
- Timer implementation: HIGH - std::time::Instant is stdlib, well-documented

**Research date:** 2026-01-24
**Valid until:** 60 days (stable domain, stdlib APIs don't change frequently)

**Notes:**
- No new dependencies required - all capabilities exist in current project
- Existing patterns from Phase 6 (async workers) not needed for this phase - all operations are local/synchronous
- Sound notification system already battle-tested in terminal module
- Workspace-scoped storage pattern already established
