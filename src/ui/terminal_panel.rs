//! Terminal panel UI component.
//!
//! This module provides a bottom panel with an integrated terminal emulator,
//! supporting multiple terminal tabs.

use crate::terminal::{TerminalManager, TerminalWidget};
use eframe::egui::{self, Color32, Id, Ui};

/// Output from the terminal panel.
#[derive(Debug, Default)]
pub struct TerminalPanelOutput {
    /// Whether the panel was closed by the user
    pub closed: bool,
    /// Whether the panel visibility was toggled
    pub toggled: bool,
}

/// Terminal panel state that persists across frames.
pub struct TerminalPanelState {
    /// Terminal manager handling all terminal instances
    pub manager: TerminalManager,
    /// Whether the terminal panel is visible
    pub visible: bool,
    /// Panel height in pixels
    pub height: f32,
    /// Whether a terminal has been initialized
    pub initialized: bool,
    /// Scroll offset for scrollback viewing
    pub scroll_offset: usize,
    /// Working directory for new terminals (workspace root or current file's directory)
    pub working_dir: Option<std::path::PathBuf>,
}

impl Default for TerminalPanelState {
    fn default() -> Self {
        Self {
            manager: TerminalManager::new(),
            visible: false,
            height: 300.0,
            initialized: false,
            scroll_offset: 0,
            working_dir: None,
        }
    }
}

impl TerminalPanelState {
    /// Create a new terminal panel state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the panel is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Toggle panel visibility.
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible && !self.initialized {
            self.initialize();
        }
    }

    /// Show the panel.
    pub fn show(&mut self) {
        self.visible = true;
        if !self.initialized {
            self.initialize();
        }
    }

    /// Hide the panel.
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Initialize the first terminal if needed.
    fn initialize(&mut self) {
        if !self.initialized {
            match self.manager.create_terminal(self.working_dir.clone()) {
                Ok(_) => {
                    self.initialized = true;
                    log::info!("Terminal initialized in {:?}", self.working_dir);
                }
                Err(e) => {
                    log::error!("Failed to initialize terminal: {}", e);
                }
            }
        }
    }

    /// Set the panel height.
    pub fn set_height(&mut self, height: f32) {
        self.height = height.clamp(100.0, 600.0);
    }

    /// Get the panel height.
    pub fn height(&self) -> f32 {
        self.height
    }
}

/// Terminal panel UI component.
pub struct TerminalPanel {
    /// Unique ID for the panel
    id: Id,
}

impl Default for TerminalPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalPanel {
    /// Create a new terminal panel.
    pub fn new() -> Self {
        Self {
            id: Id::new("terminal_panel"),
        }
    }

    /// Show the terminal panel.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        state: &mut TerminalPanelState,
        is_dark: bool,
    ) -> TerminalPanelOutput {
        let mut output = TerminalPanelOutput::default();

        // Poll all terminals for new data
        state.manager.poll_all();

        // Get theme colors
        let bg_color = if is_dark {
            Color32::from_rgb(30, 30, 30)
        } else {
            Color32::from_rgb(245, 245, 245)
        };
        let border_color = if is_dark {
            Color32::from_rgb(60, 60, 60)
        } else {
            Color32::from_rgb(200, 200, 200)
        };
        let tab_bg = if is_dark {
            Color32::from_rgb(40, 40, 40)
        } else {
            Color32::from_rgb(235, 235, 235)
        };
        let tab_active_bg = if is_dark {
            Color32::from_rgb(50, 50, 55)
        } else {
            Color32::from_rgb(255, 255, 255)
        };
        let text_color = if is_dark {
            Color32::from_rgb(220, 220, 220)
        } else {
            Color32::from_rgb(30, 30, 30)
        };

        // Draw panel background
        let panel_rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(panel_rect, 0.0, bg_color);

        // Draw top border
        ui.painter().line_segment(
            [
                egui::pos2(panel_rect.left(), panel_rect.top()),
                egui::pos2(panel_rect.right(), panel_rect.top()),
            ],
            egui::Stroke::new(1.0, border_color),
        );

        ui.vertical(|ui| {
            // Tab bar and controls
            ui.horizontal(|ui| {
                ui.add_space(8.0);

                // Terminal tabs
                let titles = state.manager.terminal_titles();
                let mut close_tab: Option<usize> = None;

                for (idx, title, is_active) in &titles {
                    let tab_response = ui.add(
                        egui::Button::new(
                            egui::RichText::new(title)
                                .size(12.0)
                                .color(text_color),
                        )
                        .fill(if *is_active { tab_active_bg } else { tab_bg })
                        .stroke(egui::Stroke::new(1.0, border_color))
                        .rounding(egui::Rounding::same(4.0)),
                    );

                    if tab_response.clicked() {
                        state.manager.set_active(*idx);
                    }

                    // Show close button on hover
                    if tab_response.hovered() {
                        let close_response = ui.add(
                            egui::Button::new(
                                egui::RichText::new("×")
                                    .size(12.0)
                                    .color(text_color),
                            )
                            .frame(false)
                            .min_size(egui::vec2(16.0, 16.0)),
                        );
                        if close_response.clicked() {
                            close_tab = Some(*idx);
                        }
                    }

                    ui.add_space(4.0);
                }

                // New terminal button
                let new_btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new("+")
                            .size(14.0)
                            .color(text_color),
                    )
                    .fill(tab_bg)
                    .stroke(egui::Stroke::new(1.0, border_color))
                    .rounding(egui::Rounding::same(4.0))
                    .min_size(egui::vec2(24.0, 24.0)),
                );

                if new_btn.clone().on_hover_text("New terminal").clicked() {
                    if let Err(e) = state.manager.create_terminal(state.working_dir.clone()) {
                        log::error!("Failed to create terminal: {}", e);
                    }
                }

                // Handle tab close
                if let Some(idx) = close_tab {
                    state.manager.close_terminal(idx);
                }

                // Spacer
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Close panel button
                    let close_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new("×")
                                .size(16.0)
                                .color(text_color),
                        )
                        .frame(false)
                        .min_size(egui::vec2(24.0, 24.0)),
                    );

                    if close_btn.clone().on_hover_text("Close terminal panel").clicked() {
                        output.closed = true;
                        state.hide();
                    }

                    ui.add_space(8.0);
                });
            });

            ui.add_space(4.0);

            // Render active terminal
            if let Some(terminal) = state.manager.active_terminal_mut() {
                let screen = terminal.screen();

                // Create terminal widget
                let widget = TerminalWidget::new(screen)
                    .font_size(14.0)
                    .focused(true)
                    .is_dark(is_dark);

                let widget_output = widget.show(ui);

                // Send keyboard input to terminal
                if !widget_output.input.is_empty() {
                    terminal.write_input(&widget_output.input);
                }

                // Handle resize
                if let Some((cols, rows)) = widget_output.new_size {
                    terminal.resize(cols, rows);
                }
            } else {
                // No terminal - show placeholder
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new("No terminal. Click + to create one.")
                            .color(text_color)
                            .size(14.0),
                    );
                });
            }
        });

        output
    }
}
