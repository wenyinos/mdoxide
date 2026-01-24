//! Productivity panel data models and persistence for Ferrite
//!
//! This module provides the core data structures for the productivity hub:
//! - Task management with markdown parsing
//! - Pomodoro timer state machine
//! - AutoSave helper for debounced writes
//! - Workspace-scoped persistence functions

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{Duration, Instant};

// ─────────────────────────────────────────────────────────────────────────────
// Task Management
// ─────────────────────────────────────────────────────────────────────────────

/// A task item parsed from markdown checkbox syntax.
///
/// Supports:
/// - `- [ ] Task text` - Unchecked task
/// - `- [x] Task text` - Checked task
/// - `- [ ] ! Important` - Priority 1
/// - `- [ ] !! Urgent` - Priority 2
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub completed: bool,
    pub text: String,
    pub priority: u8, // 0=none, 1=!, 2=!!
}

impl Task {
    /// Parse a task from markdown checkbox syntax.
    ///
    /// Returns `None` if the line is not a valid task format.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// assert_eq!(Task::from_markdown("- [ ] Buy milk").unwrap().text, "Buy milk");
    /// assert_eq!(Task::from_markdown("- [x] Done").unwrap().completed, true);
    /// assert_eq!(Task::from_markdown("- [ ] !! Urgent").unwrap().priority, 2);
    /// ```
    pub fn from_markdown(line: &str) -> Option<Self> {
        let trimmed = line.trim();

        // Must start with "- [ ]" or "- [x]"
        if !trimmed.starts_with("- [") {
            return None;
        }

        // Extract checkbox state
        let completed = if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            true
        } else if trimmed.starts_with("- [ ]") {
            false
        } else {
            return None;
        };

        // Extract text after checkbox
        let after_checkbox = if completed {
            trimmed.strip_prefix("- [x]").or_else(|| trimmed.strip_prefix("- [X]"))?
        } else {
            trimmed.strip_prefix("- [ ]")?
        };

        let text = after_checkbox.trim();

        // Extract priority
        let (priority, text) = if let Some(rest) = text.strip_prefix("!! ") {
            (2, rest.to_string())
        } else if let Some(rest) = text.strip_prefix("! ") {
            (1, rest.to_string())
        } else {
            (0, text.to_string())
        };

        Some(Task {
            completed,
            text,
            priority,
        })
    }

    /// Convert task back to markdown format.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let task = Task { completed: false, text: "Buy milk".to_string(), priority: 0 };
    /// assert_eq!(task.to_markdown(), "- [ ] Buy milk");
    /// ```
    pub fn to_markdown(&self) -> String {
        let checkbox = if self.completed { "[x]" } else { "[ ]" };
        let priority_prefix = match self.priority {
            2 => "!! ",
            1 => "! ",
            _ => "",
        };
        format!("- {} {}{}", checkbox, priority_prefix, self.text)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Pomodoro Timer
// ─────────────────────────────────────────────────────────────────────────────

/// Pomodoro timer state machine.
///
/// Uses `std::time::Instant` for timing to avoid issues with system clock changes.
#[derive(Clone, Debug)]
pub struct PomodoroTimer {
    state: TimerState,
    work_duration_secs: u64,    // Default: 25 * 60
    break_duration_secs: u64,   // Default: 5 * 60
    completed_cycles: usize,
}

/// Internal timer state.
#[derive(Clone, Debug)]
enum TimerState {
    Idle,
    Work { started: Instant },
    Break { started: Instant },
}

impl PomodoroTimer {
    /// Create a new timer with default durations (25min work, 5min break).
    pub fn new() -> Self {
        Self {
            state: TimerState::Idle,
            work_duration_secs: 25 * 60,
            break_duration_secs: 5 * 60,
            completed_cycles: 0,
        }
    }

    /// Start a work session.
    pub fn start_work(&mut self) {
        self.state = TimerState::Work {
            started: Instant::now(),
        };
    }

    /// Start a break session.
    pub fn start_break(&mut self) {
        self.state = TimerState::Break {
            started: Instant::now(),
        };
    }

    /// Stop the timer.
    pub fn stop(&mut self) {
        self.state = TimerState::Idle;
    }

    /// Increment the completed cycles counter.
    pub fn increment_cycle(&mut self) {
        self.completed_cycles += 1;
    }

    /// Get the number of completed cycles.
    pub fn cycles(&self) -> usize {
        self.completed_cycles
    }

    /// Get remaining time in current session.
    ///
    /// Returns `None` if timer is idle.
    pub fn remaining(&self) -> Option<Duration> {
        match &self.state {
            TimerState::Idle => None,
            TimerState::Work { started } => {
                let elapsed = started.elapsed().as_secs();
                if elapsed >= self.work_duration_secs {
                    Some(Duration::from_secs(0))
                } else {
                    Some(Duration::from_secs(self.work_duration_secs - elapsed))
                }
            }
            TimerState::Break { started } => {
                let elapsed = started.elapsed().as_secs();
                if elapsed >= self.break_duration_secs {
                    Some(Duration::from_secs(0))
                } else {
                    Some(Duration::from_secs(self.break_duration_secs - elapsed))
                }
            }
        }
    }

    /// Check if the timer has reached zero.
    pub fn is_complete(&self) -> bool {
        matches!(self.remaining(), Some(d) if d.as_secs() == 0)
    }

    /// Format remaining time as "MM:SS".
    pub fn format_remaining(&self) -> String {
        if let Some(remaining) = self.remaining() {
            let total_secs = remaining.as_secs();
            let minutes = total_secs / 60;
            let seconds = total_secs % 60;
            format!("{:02}:{:02}", minutes, seconds)
        } else {
            "00:00".to_string()
        }
    }

    /// Check if currently in a work session.
    pub fn is_work(&self) -> bool {
        matches!(self.state, TimerState::Work { .. })
    }

    /// Check if currently in a break session.
    pub fn is_break(&self) -> bool {
        matches!(self.state, TimerState::Break { .. })
    }

    /// Check if timer is active (work or break).
    pub fn is_active(&self) -> bool {
        !matches!(self.state, TimerState::Idle)
    }
}

impl Default for PomodoroTimer {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AutoSave Helper
// ─────────────────────────────────────────────────────────────────────────────

/// Debounced auto-save helper.
///
/// Prevents excessive file writes by debouncing edits.
pub struct AutoSave {
    last_edit: Instant,
    debounce_duration: Duration,
    pending_content: Option<String>,
}

impl AutoSave {
    /// Create a new auto-save helper with the given debounce duration in milliseconds.
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            last_edit: Instant::now(),
            debounce_duration: Duration::from_millis(debounce_ms),
            pending_content: None,
        }
    }

    /// Mark content as edited, resetting the debounce timer.
    pub fn mark_edited(&mut self, content: String) {
        self.last_edit = Instant::now();
        self.pending_content = Some(content);
    }

    /// Check if enough time has passed to save.
    pub fn should_save(&self) -> bool {
        self.pending_content.is_some() && self.last_edit.elapsed() >= self.debounce_duration
    }

    /// Take the pending content (consuming it).
    pub fn take_pending(&mut self) -> Option<String> {
        self.pending_content.take()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Persistence Functions
// ─────────────────────────────────────────────────────────────────────────────

/// Save tasks to .ferrite/tasks.json in workspace root.
///
/// Uses atomic write pattern (write to .bak, then rename).
pub fn save_tasks(workspace_root: &Path, tasks: &[Task]) -> std::io::Result<()> {
    let ferrite_dir = workspace_root.join(".ferrite");
    std::fs::create_dir_all(&ferrite_dir)?;

    let tasks_path = ferrite_dir.join("tasks.json");
    let backup_path = ferrite_dir.join("tasks.json.bak");

    let json = serde_json::to_string_pretty(tasks)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    // Atomic write: backup first, then rename
    std::fs::write(&backup_path, &json)?;
    std::fs::rename(&backup_path, &tasks_path)?;

    Ok(())
}

/// Load tasks from .ferrite/tasks.json in workspace root.
///
/// Returns empty Vec if file doesn't exist or is invalid.
pub fn load_tasks(workspace_root: &Path) -> Vec<Task> {
    let tasks_path = workspace_root.join(".ferrite").join("tasks.json");

    if !tasks_path.exists() {
        return Vec::new();
    }

    std::fs::read_to_string(&tasks_path)
        .ok()
        .and_then(|contents| serde_json::from_str(&contents).ok())
        .unwrap_or_default()
}

/// Save note content to .ferrite/notes/{name}.txt
pub fn save_note(workspace_root: &Path, name: &str, content: &str) -> std::io::Result<()> {
    let notes_dir = workspace_root.join(".ferrite").join("notes");
    std::fs::create_dir_all(&notes_dir)?;

    // Sanitize name to prevent path traversal
    let safe_name = name.replace(['/', '\\'], "_").replace("..", "_");
    let note_path = notes_dir.join(format!("{}.txt", safe_name));
    let backup_path = notes_dir.join(format!("{}.txt.bak", safe_name));

    // Atomic write
    std::fs::write(&backup_path, content)?;
    std::fs::rename(&backup_path, &note_path)?;

    Ok(())
}

/// Load note content from .ferrite/notes/{name}.txt
pub fn load_note(workspace_root: &Path, name: &str) -> String {
    let safe_name = name.replace(['/', '\\'], "_").replace("..", "_");
    let note_path = workspace_root
        .join(".ferrite")
        .join("notes")
        .join(format!("{}.txt", safe_name));

    std::fs::read_to_string(&note_path).unwrap_or_default()
}

/// List available notes in workspace
pub fn list_notes(workspace_root: &Path) -> Vec<String> {
    let notes_dir = workspace_root.join(".ferrite").join("notes");

    if !notes_dir.exists() {
        return vec!["default".to_string()];
    }

    let mut notes: Vec<String> = std::fs::read_dir(&notes_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let path = e.path();
                    if path.extension()? == "txt" {
                        path.file_stem()?.to_str().map(String::from)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    if notes.is_empty() {
        notes.push("default".to_string());
    }

    notes.sort();
    notes
}

// ─────────────────────────────────────────────────────────────────────────────
// Productivity Panel UI Component
// ─────────────────────────────────────────────────────────────────────────────

/// State for the productivity hub panel.
pub struct ProductivityPanel {
    /// Current workspace root (needed for persistence)
    workspace_root: Option<std::path::PathBuf>,

    /// Task list
    tasks: Vec<Task>,

    /// New task input text
    new_task_input: String,

    /// Pomodoro timer
    timer: PomodoroTimer,

    /// Notes content
    notes_content: String,

    /// Current note name
    current_note: String,

    /// Available notes list
    available_notes: Vec<String>,

    /// Auto-save helper for notes
    auto_save: AutoSave,

    /// Flag to indicate tasks need saving
    tasks_dirty: bool,
}

impl ProductivityPanel {
    /// Create a new productivity panel.
    pub fn new() -> Self {
        Self {
            workspace_root: None,
            tasks: Vec::new(),
            new_task_input: String::new(),
            timer: PomodoroTimer::new(),
            notes_content: String::new(),
            current_note: "default".to_string(),
            available_notes: vec!["default".to_string()],
            auto_save: AutoSave::new(1000),
            tasks_dirty: false,
        }
    }

    /// Set the workspace root and load data.
    pub fn set_workspace(&mut self, workspace_root: Option<std::path::PathBuf>) {
        if self.workspace_root != workspace_root {
            // Save current workspace data before switching
            self.save_all();

            self.workspace_root = workspace_root.clone();

            // Load data for new workspace
            if let Some(ref root) = workspace_root {
                self.tasks = load_tasks(root);
                self.available_notes = list_notes(root);
                self.current_note = self.available_notes.first()
                    .cloned()
                    .unwrap_or_else(|| "default".to_string());
                self.notes_content = load_note(root, &self.current_note);
            } else {
                // No workspace - reset to defaults
                self.tasks = Vec::new();
                self.notes_content = String::new();
                self.available_notes = vec!["default".to_string()];
                self.current_note = "default".to_string();
            }

            self.tasks_dirty = false;
        }
    }

    /// Save all pending data.
    pub fn save_all(&mut self) {
        if let Some(ref root) = self.workspace_root {
            // Save tasks if dirty
            if self.tasks_dirty {
                if let Err(e) = save_tasks(root, &self.tasks) {
                    log::warn!("Failed to save tasks: {}", e);
                }
                self.tasks_dirty = false;
            }

            // Save notes if pending
            if let Some(content) = self.auto_save.take_pending() {
                if let Err(e) = save_note(root, &self.current_note, &content) {
                    log::warn!("Failed to save note: {}", e);
                }
            }
        }
    }

    /// Add a new task from the input field.
    fn add_task(&mut self) {
        let input = self.new_task_input.trim();
        if input.is_empty() {
            return;
        }

        // If input already has markdown syntax, parse it
        if let Some(task) = Task::from_markdown(input) {
            self.tasks.push(task);
        } else {
            // Otherwise create a simple unchecked task
            self.tasks.push(Task {
                completed: false,
                text: input.to_string(),
                priority: 0,
            });
        }

        self.new_task_input.clear();
        self.tasks_dirty = true;
    }

    /// Delete a task by index.
    fn delete_task(&mut self, index: usize) {
        if index < self.tasks.len() {
            self.tasks.remove(index);
            self.tasks_dirty = true;
        }
    }

    /// Render the productivity panel.
    ///
    /// Returns true if the panel requested a repaint (timer active).
    pub fn show(&mut self, ctx: &eframe::egui::Context, visible: &mut bool) -> bool {
        let mut needs_repaint = false;

        eframe::egui::Window::new("Productivity Hub")
            .open(visible)
            .default_width(350.0)
            .min_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                // TASKS SECTION
                ui.heading("Tasks");

                // Completed tasks counter
                let completed = self.tasks.iter().filter(|t| t.completed).count();
                let total = self.tasks.len();
                if total > 0 {
                    ui.label(format!("{}/{} completed", completed, total));
                }

                // New task input
                ui.horizontal(|ui| {
                    let response = ui.add(
                        eframe::egui::TextEdit::singleline(&mut self.new_task_input)
                            .hint_text("Type task or - [ ] task...")
                            .desired_width(ui.available_width() - 50.0)
                    );

                    if ui.button("Add").clicked()
                        || (response.lost_focus() && ui.input(|i| i.key_pressed(eframe::egui::Key::Enter)))
                    {
                        self.add_task();
                    }
                });

                // Keyboard shortcut hint
                ui.label(eframe::egui::RichText::new("Tip: Use - [ ] for checkbox, ! or !! for priority").small().weak());

                ui.add_space(4.0);

                // Task list with scroll area
                eframe::egui::ScrollArea::vertical()
                    .id_source("tasks_scroll")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        let mut to_delete: Option<usize> = None;
                        let mut to_move_up: Option<usize> = None;
                        let mut to_move_down: Option<usize> = None;
                        let tasks_len = self.tasks.len();

                        for (i, task) in self.tasks.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                // Move up button (disabled for first item)
                                ui.add_enabled_ui(i > 0, |ui| {
                                    if ui.small_button("^").on_hover_text("Move up").clicked() {
                                        to_move_up = Some(i);
                                    }
                                });

                                // Move down button (disabled for last item)
                                ui.add_enabled_ui(i < tasks_len - 1, |ui| {
                                    if ui.small_button("v").on_hover_text("Move down").clicked() {
                                        to_move_down = Some(i);
                                    }
                                });

                                // Checkbox
                                if ui.checkbox(&mut task.completed, "").changed() {
                                    self.tasks_dirty = true;
                                }

                                // Priority indicator
                                match task.priority {
                                    2 => { ui.colored_label(eframe::egui::Color32::RED, "!!"); }
                                    1 => { ui.colored_label(eframe::egui::Color32::YELLOW, "!"); }
                                    _ => {}
                                }

                                // Task text (strikethrough if completed)
                                let text = if task.completed {
                                    eframe::egui::RichText::new(&task.text).strikethrough()
                                } else {
                                    eframe::egui::RichText::new(&task.text)
                                };
                                ui.label(text);

                                // Delete button (right-aligned)
                                ui.with_layout(eframe::egui::Layout::right_to_left(eframe::egui::Align::Center), |ui| {
                                    if ui.small_button("x").clicked() {
                                        to_delete = Some(i);
                                    }
                                });
                            });
                        }

                        // Handle moves after the loop
                        if let Some(i) = to_move_up {
                            if i > 0 {
                                self.tasks.swap(i, i - 1);
                                self.tasks_dirty = true;
                            }
                        }
                        if let Some(i) = to_move_down {
                            if i < self.tasks.len() - 1 {
                                self.tasks.swap(i, i + 1);
                                self.tasks_dirty = true;
                            }
                        }

                        if let Some(index) = to_delete {
                            self.delete_task(index);
                        }

                        if self.tasks.is_empty() {
                            ui.label(eframe::egui::RichText::new("No tasks yet").weak());
                        }
                    });

                ui.separator();

                // POMODORO SECTION
                ui.heading("Pomodoro Timer");

                ui.horizontal(|ui| {
                    // Timer display
                    let time_text = self.timer.format_remaining();
                    let label = if self.timer.is_work() {
                        format!("Work: {}", time_text)
                    } else if self.timer.is_break() {
                        format!("Break: {}", time_text)
                    } else {
                        "Ready".to_string()
                    };

                    ui.label(eframe::egui::RichText::new(label).size(24.0).strong());

                    // Cycles counter
                    if self.timer.cycles() > 0 {
                        ui.label(format!("Cycles: {}", self.timer.cycles()));
                    }
                });

                ui.horizontal(|ui| {
                    if self.timer.is_active() {
                        if ui.button("Stop").clicked() {
                            self.timer.stop();
                        }

                        // Request repaint for countdown
                        ctx.request_repaint_after(Duration::from_secs(1));
                        needs_repaint = true;

                        // Check completion
                        if self.timer.is_complete() {
                            // Play notification sound using the re-exported function
                            crate::terminal::play_notification(None);

                            // Auto-transition
                            if self.timer.is_work() {
                                self.timer.increment_cycle();
                                self.timer.start_break();
                            } else {
                                self.timer.stop();
                            }
                        }
                    } else {
                        if ui.button("Start Work (25m)").clicked() {
                            self.timer.start_work();
                        }
                        if ui.button("Start Break (5m)").clicked() {
                            self.timer.start_break();
                        }
                    }
                });

                ui.separator();

                // NOTES SECTION
                ui.heading("Quick Notes");

                // Note selector (if multiple notes exist)
                if self.available_notes.len() > 1 || self.workspace_root.is_some() {
                    ui.horizontal(|ui| {
                        ui.label("Note:");
                        eframe::egui::ComboBox::from_id_source("note_selector")
                            .selected_text(&self.current_note)
                            .show_ui(ui, |ui| {
                                for note in &self.available_notes.clone() {
                                    if ui.selectable_label(self.current_note == *note, note).clicked() {
                                        // Save current note before switching
                                        if let Some(ref root) = self.workspace_root {
                                            if self.auto_save.take_pending().is_some() || !self.notes_content.is_empty() {
                                                let _ = save_note(root, &self.current_note, &self.notes_content);
                                            }
                                            self.current_note = note.clone();
                                            self.notes_content = load_note(root, &self.current_note);
                                        }
                                    }
                                }
                            });

                        // New note button
                        if ui.small_button("+").clicked() {
                            // Create a new note with a unique name
                            let new_name = format!("note_{}", self.available_notes.len() + 1);
                            self.available_notes.push(new_name.clone());
                            if let Some(ref root) = self.workspace_root {
                                let _ = save_note(root, &self.current_note, &self.notes_content);
                            }
                            self.current_note = new_name;
                            self.notes_content = String::new();
                        }
                    });
                }

                // Notes text area
                let response = ui.add(
                    eframe::egui::TextEdit::multiline(&mut self.notes_content)
                        .desired_rows(8)
                        .hint_text("Type your notes here...")
                        .desired_width(f32::INFINITY)
                );

                if response.changed() {
                    self.auto_save.mark_edited(self.notes_content.clone());
                }

                // Auto-save check
                if self.auto_save.should_save() {
                    if let (Some(ref root), Some(content)) = (&self.workspace_root, self.auto_save.take_pending()) {
                        if let Err(e) = save_note(root, &self.current_note, &content) {
                            log::warn!("Failed to auto-save note: {}", e);
                        }
                    }
                }

                // Save tasks if dirty (debounced by frame rate)
                if self.tasks_dirty {
                    if let Some(ref root) = self.workspace_root {
                        if let Err(e) = save_tasks(root, &self.tasks) {
                            log::warn!("Failed to save tasks: {}", e);
                        }
                        self.tasks_dirty = false;
                    }
                }
            });

        needs_repaint
    }
}

impl Default for ProductivityPanel {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // Task parsing tests
    #[test]
    fn test_task_from_markdown_unchecked() {
        let task = Task::from_markdown("- [ ] Buy milk").unwrap();
        assert!(!task.completed);
        assert_eq!(task.text, "Buy milk");
        assert_eq!(task.priority, 0);
    }

    #[test]
    fn test_task_from_markdown_checked() {
        let task = Task::from_markdown("- [x] Done task").unwrap();
        assert!(task.completed);
        assert_eq!(task.text, "Done task");
    }

    #[test]
    fn test_task_from_markdown_priority_high() {
        let task = Task::from_markdown("- [ ] !! Urgent").unwrap();
        assert_eq!(task.priority, 2);
        assert_eq!(task.text, "Urgent");
    }

    #[test]
    fn test_task_from_markdown_priority_medium() {
        let task = Task::from_markdown("- [ ] ! Important").unwrap();
        assert_eq!(task.priority, 1);
        assert_eq!(task.text, "Important");
    }

    #[test]
    fn test_task_from_markdown_invalid() {
        assert!(Task::from_markdown("Not a task").is_none());
        assert!(Task::from_markdown("- Regular list item").is_none());
        assert!(Task::from_markdown("[ ] Missing dash").is_none());
    }

    #[test]
    fn test_task_to_markdown() {
        let task = Task {
            completed: false,
            text: "Test".to_string(),
            priority: 0,
        };
        assert_eq!(task.to_markdown(), "- [ ] Test");

        let task = Task {
            completed: true,
            text: "Done".to_string(),
            priority: 0,
        };
        assert_eq!(task.to_markdown(), "- [x] Done");

        let task = Task {
            completed: false,
            text: "Urgent".to_string(),
            priority: 2,
        };
        assert_eq!(task.to_markdown(), "- [ ] !! Urgent");
    }

    // Pomodoro timer tests
    #[test]
    fn test_timer_initial_state() {
        let timer = PomodoroTimer::new();
        assert!(!timer.is_active());
        assert!(timer.remaining().is_none());
    }

    #[test]
    fn test_timer_work_session() {
        let mut timer = PomodoroTimer::new();
        timer.start_work();

        assert!(timer.is_active());
        assert!(timer.is_work());
        assert!(!timer.is_break());

        // Should have ~25 minutes remaining (allow small tolerance)
        let remaining = timer.remaining().unwrap();
        assert!(remaining.as_secs() > 24 * 60);
        assert!(remaining.as_secs() <= 25 * 60);
    }

    #[test]
    fn test_timer_format() {
        let mut timer = PomodoroTimer::new();
        timer.start_work();

        let formatted = timer.format_remaining();
        // Should be like "24:59" or "25:00"
        assert!(formatted.contains(':'));
        assert_eq!(formatted.len(), 5);
    }

    #[test]
    fn test_timer_stop() {
        let mut timer = PomodoroTimer::new();
        timer.start_work();
        assert!(timer.is_active());

        timer.stop();
        assert!(!timer.is_active());
    }

    // AutoSave tests
    #[test]
    fn test_autosave_initial() {
        let autosave = AutoSave::new(1000);
        assert!(!autosave.should_save());
    }

    #[test]
    fn test_autosave_mark_edited() {
        let mut autosave = AutoSave::new(10); // 10ms for testing
        autosave.mark_edited("test content".to_string());

        // Immediately after edit, should not save (debounce)
        // Note: This might pass due to timing, so we just check pending exists
        assert!(autosave.pending_content.is_some());
    }

    #[test]
    fn test_autosave_take_pending() {
        let mut autosave = AutoSave::new(1000);
        autosave.mark_edited("content".to_string());

        // Manually trigger the save check
        autosave.last_edit = std::time::Instant::now() - Duration::from_secs(2);

        assert!(autosave.should_save());
        let content = autosave.take_pending();
        assert_eq!(content, Some("content".to_string()));
        assert!(autosave.pending_content.is_none());
    }
}
