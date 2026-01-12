//! Settings Panel Component for Ferrite
//!
//! This module implements a modal settings panel that allows users to configure
//! appearance, editor behavior, and file handling options with live preview.

use crate::config::{EditorFont, MaxLineWidth, Settings, Theme, ViewMode};
use eframe::egui::{self, Color32, RichText, Ui};

/// Settings panel sections for navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSection {
    #[default]
    Appearance,
    Editor,
    Files,
}

impl SettingsSection {
    /// Get the display label for the section.
    pub fn label(&self) -> &'static str {
        match self {
            SettingsSection::Appearance => "Appearance",
            SettingsSection::Editor => "Editor",
            SettingsSection::Files => "Files",
        }
    }

    /// Get the icon for the section.
    pub fn icon(&self) -> &'static str {
        match self {
            SettingsSection::Appearance => "🎨",
            SettingsSection::Editor => "📝",
            SettingsSection::Files => "📁",
        }
    }
}

/// Result of showing the settings panel.
#[derive(Debug, Clone, Default)]
pub struct SettingsPanelOutput {
    /// Whether settings were modified.
    pub changed: bool,
    /// Whether the panel should be closed.
    pub close_requested: bool,
    /// Whether a reset to defaults was requested.
    pub reset_requested: bool,
}

/// Settings panel state and rendering.
#[derive(Debug, Clone)]
pub struct SettingsPanel {
    /// Currently active settings section.
    active_section: SettingsSection,
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsPanel {
    /// Create a new settings panel instance.
    pub fn new() -> Self {
        Self {
            active_section: SettingsSection::default(),
        }
    }

    /// Show the settings panel as a modal window.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context
    /// * `settings` - The current settings (mutable for live preview)
    /// * `is_dark` - Whether the current theme is dark mode
    ///
    /// # Returns
    ///
    /// Output indicating what actions to take
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        settings: &mut Settings,
        is_dark: bool,
    ) -> SettingsPanelOutput {
        let mut output = SettingsPanelOutput::default();

        // Semi-transparent overlay
        let screen_rect = ctx.screen_rect();
        let overlay_color = if is_dark {
            Color32::from_rgba_unmultiplied(0, 0, 0, 180)
        } else {
            Color32::from_rgba_unmultiplied(0, 0, 0, 120)
        };

        egui::Area::new(egui::Id::new("settings_overlay"))
            .order(egui::Order::Middle)
            .fixed_pos(screen_rect.min)
            .show(ctx, |ui| {
                let response = ui.allocate_response(screen_rect.size(), egui::Sense::click());
                ui.painter().rect_filled(screen_rect, 0.0, overlay_color);

                // Close on click outside
                if response.clicked() {
                    output.close_requested = true;
                }
            });

        // Settings modal window
        egui::Window::new("⚙ Settings")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .min_width(500.0)
            .max_width(600.0)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                // Handle escape key to close
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    output.close_requested = true;
                }

                ui.horizontal(|ui| {
                    // Left side: Section tabs
                    ui.vertical(|ui| {
                        ui.set_min_width(120.0);

                        for section in [
                            SettingsSection::Appearance,
                            SettingsSection::Editor,
                            SettingsSection::Files,
                        ] {
                            let selected = self.active_section == section;
                            let text = format!("{} {}", section.icon(), section.label());

                            let btn = ui.add_sized(
                                [110.0, 32.0],
                                egui::SelectableLabel::new(
                                    selected,
                                    RichText::new(text).size(14.0),
                                ),
                            );

                            if btn.clicked() {
                                self.active_section = section;
                            }
                        }
                    });

                    ui.separator();

                    // Right side: Section content
                    ui.vertical(|ui| {
                        ui.set_min_width(350.0);
                        ui.set_min_height(320.0);

                        match self.active_section {
                            SettingsSection::Appearance => {
                                if self.show_appearance_section(ui, settings, is_dark) {
                                    output.changed = true;
                                }
                            }
                            SettingsSection::Editor => {
                                if self.show_editor_section(ui, settings) {
                                    output.changed = true;
                                }
                            }
                            SettingsSection::Files => {
                                if self.show_files_section(ui, settings) {
                                    output.changed = true;
                                }
                            }
                        }
                    });
                });

                ui.separator();

                // Bottom buttons
                ui.horizontal(|ui| {
                    // Reset button on the left
                    if ui
                        .button("↺ Reset All")
                        .on_hover_text("Reset all settings to defaults")
                        .clicked()
                    {
                        output.reset_requested = true;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            output.close_requested = true;
                        }
                        ui.label(
                            RichText::new("Settings are saved automatically")
                                .small()
                                .weak(),
                        );
                    });
                });
            });

        output
    }

    /// Show the Appearance settings section.
    ///
    /// Returns true if any setting was changed.
    fn show_appearance_section(
        &mut self,
        ui: &mut Ui,
        settings: &mut Settings,
        _is_dark: bool,
    ) -> bool {
        let mut changed = false;

        ui.heading("Appearance");
        ui.add_space(8.0);

        // Theme selection
        ui.label(RichText::new("Theme").strong());
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            for theme in [Theme::Light, Theme::Dark, Theme::System] {
                let label = match theme {
                    Theme::Light => "☀ Light",
                    Theme::Dark => "🌙 Dark",
                    Theme::System => "💻 System",
                };
                if ui
                    .selectable_value(&mut settings.theme, theme, label)
                    .changed()
                {
                    changed = true;
                }
            }
        });

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Font family selection
        ui.label(RichText::new("Font").strong());
        ui.add_space(4.0);

        for font in EditorFont::all() {
            ui.horizontal(|ui| {
                if ui
                    .selectable_value(&mut settings.font_family, *font, font.display_name())
                    .changed()
                {
                    changed = true;
                }
                ui.label(RichText::new(font.description()).weak().small());
            });
        }

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Font size slider
        ui.horizontal(|ui| {
            ui.label(RichText::new("Font Size").strong());
            ui.add_space(8.0);
            ui.label(format!("{}px", settings.font_size as u32));
        });
        ui.add_space(4.0);

        let font_slider = ui.add(
            egui::Slider::new(
                &mut settings.font_size,
                Settings::MIN_FONT_SIZE..=Settings::MAX_FONT_SIZE,
            )
            .show_value(false)
            .step_by(1.0),
        );
        if font_slider.changed() {
            changed = true;
        }

        // Font size presets
        ui.horizontal(|ui| {
            for (label, size) in [("Small", 12.0), ("Medium", 14.0), ("Large", 18.0)] {
                if ui.small_button(label).clicked() {
                    settings.font_size = size;
                    changed = true;
                }
            }
        });

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Default View Mode selection
        ui.label(RichText::new("Default View Mode").strong());
        ui.add_space(4.0);
        ui.label(
            RichText::new("View mode for new tabs (existing tabs retain their saved view mode)")
                .weak()
                .small(),
        );
        ui.add_space(4.0);

        for view_mode in ViewMode::all() {
            ui.horizontal(|ui| {
                if ui
                    .selectable_value(
                        &mut settings.default_view_mode,
                        *view_mode,
                        format!("{} {}", view_mode.icon(), view_mode.label()),
                    )
                    .changed()
                {
                    changed = true;
                }
                ui.label(RichText::new(view_mode.description()).weak().small());
            });
        }

        changed
    }

    /// Show the Editor settings section.
    ///
    /// Returns true if any setting was changed.
    fn show_editor_section(&mut self, ui: &mut Ui, settings: &mut Settings) -> bool {
        let mut changed = false;

        ui.heading("Editor");
        ui.add_space(8.0);

        // Word wrap toggle
        if ui
            .checkbox(&mut settings.word_wrap, "Word Wrap")
            .on_hover_text("Wrap long lines instead of horizontal scrolling")
            .changed()
        {
            changed = true;
        }

        ui.add_space(4.0);

        // Line numbers toggle
        if ui
            .checkbox(&mut settings.show_line_numbers, "Show Line Numbers")
            .on_hover_text("Display line numbers in the editor gutter")
            .changed()
        {
            changed = true;
        }

        ui.add_space(4.0);

        // Minimap toggle
        if ui
            .checkbox(&mut settings.minimap_enabled, "Show Minimap")
            .on_hover_text("Display a minimap navigation panel on the right side of the editor")
            .changed()
        {
            changed = true;
        }

        ui.add_space(4.0);

        // Bracket matching toggle
        if ui
            .checkbox(&mut settings.highlight_matching_pairs, "Highlight Matching Brackets")
            .on_hover_text("Highlight matching brackets (), [], {}, <> and emphasis pairs ** and __ when cursor is adjacent")
            .changed()
        {
            changed = true;
        }

        ui.add_space(4.0);

        // Auto-close brackets toggle
        if ui
            .checkbox(&mut settings.auto_close_brackets, "Auto-close Brackets & Quotes")
            .on_hover_text("Automatically insert closing brackets and quotes. Wraps selected text when typing an opener.")
            .changed()
        {
            changed = true;
        }

        ui.add_space(4.0);

        // Syntax highlighting toggle
        if ui
            .checkbox(&mut settings.syntax_highlighting_enabled, "Syntax Highlighting")
            .on_hover_text("Enable syntax highlighting for source code files (Rust, Python, JavaScript, etc.)")
            .changed()
        {
            changed = true;
        }

        ui.add_space(4.0);

        // Sync scroll toggle
        if ui
            .checkbox(&mut settings.sync_scroll_enabled, "Sync Scroll")
            .on_hover_text(
                "Synchronize scroll position when switching between Raw and Rendered views",
            )
            .changed()
        {
            changed = true;
        }

        ui.add_space(4.0);

        // Use spaces toggle
        if ui
            .checkbox(&mut settings.use_spaces, "Use Spaces for Indentation")
            .on_hover_text("Use spaces instead of tabs for indentation")
            .changed()
        {
            changed = true;
        }

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Tab size slider
        ui.horizontal(|ui| {
            ui.label(RichText::new("Tab Size").strong());
            ui.add_space(8.0);
            ui.label(format!("{} spaces", settings.tab_size));
        });
        ui.add_space(4.0);

        let mut tab_size_f32 = settings.tab_size as f32;
        let tab_slider = ui.add(
            egui::Slider::new(
                &mut tab_size_f32,
                Settings::MIN_TAB_SIZE as f32..=Settings::MAX_TAB_SIZE as f32,
            )
            .show_value(false)
            .step_by(1.0),
        );
        if tab_slider.changed() {
            settings.tab_size = tab_size_f32 as u8;
            changed = true;
        }

        // Tab size presets
        ui.horizontal(|ui| {
            for size in [2u8, 4, 8] {
                if ui.small_button(format!("{}", size)).clicked() {
                    settings.tab_size = size;
                    changed = true;
                }
            }
        });

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Maximum Line Width section
        ui.label(RichText::new("Maximum Line Width").strong());
        ui.add_space(4.0);
        ui.label(
            RichText::new("Constrain text width and center the text column in the viewport")
                .weak()
                .small(),
        );
        ui.add_space(4.0);

        // Dropdown for preset options
        let current_display = settings.max_line_width.display_name();
        egui::ComboBox::from_id_source("max_line_width_combo")
            .selected_text(current_display)
            .show_ui(ui, |ui| {
                for preset in MaxLineWidth::presets() {
                    let label = format!("{} - {}", preset.display_name(), preset.description());
                    if ui
                        .selectable_value(&mut settings.max_line_width, *preset, label)
                        .changed()
                    {
                        changed = true;
                    }
                }
                // Custom option
                let is_custom = settings.max_line_width.is_custom();
                let custom_label = "Custom - Specify pixel width";
                if ui.selectable_label(is_custom, custom_label).clicked() && !is_custom {
                    // Switch to custom with a sensible default
                    settings.max_line_width = MaxLineWidth::Custom(800);
                    changed = true;
                }
            });

        // Show numeric input when custom is selected
        if let MaxLineWidth::Custom(px) = &mut settings.max_line_width {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Width (pixels):");
                let mut px_value = *px as f32;
                let drag = ui.add(
                    egui::DragValue::new(&mut px_value)
                        .speed(10.0)
                        .range(Settings::MIN_CUSTOM_LINE_WIDTH as f32..=Settings::MAX_CUSTOM_LINE_WIDTH as f32)
                        .suffix("px"),
                );
                if drag.changed() {
                    *px = px_value as u32;
                    changed = true;
                }
            });
            ui.horizontal(|ui| {
                for (label, value) in [("600px", 600u32), ("800px", 800), ("1000px", 1000)] {
                    if ui.small_button(label).clicked() {
                        *px = value;
                        changed = true;
                    }
                }
            });
        }

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Code Folding section
        ui.label(RichText::new("Code Folding").strong());
        ui.add_space(4.0);

        // Master folding toggle
        if ui
            .checkbox(&mut settings.folding_enabled, "Enable Code Folding")
            .on_hover_text("Allow collapsing sections of the document (Ctrl+Shift+[ to fold all, Ctrl+Shift+] to unfold)")
            .changed()
        {
            changed = true;
        }

        // Only show sub-options if folding is enabled
        if settings.folding_enabled {
            ui.add_space(4.0);
            ui.indent("fold_options", |ui| {
                if ui
                    .checkbox(&mut settings.folding_show_indicators, "Show Fold Indicators")
                    .on_hover_text("Display fold indicators in the gutter (visual only - collapse not yet implemented)")
                    .changed()
                {
                    changed = true;
                }

                ui.add_space(4.0);
                ui.label(RichText::new("Fold Types:").small());
                ui.add_space(2.0);

                if ui
                    .checkbox(&mut settings.fold_headings, "Headings")
                    .on_hover_text("Fold markdown headings and their content")
                    .changed()
                {
                    changed = true;
                }

                if ui
                    .checkbox(&mut settings.fold_code_blocks, "Code Blocks")
                    .on_hover_text("Fold fenced code blocks (```...```)")
                    .changed()
                {
                    changed = true;
                }

                if ui
                    .checkbox(&mut settings.fold_lists, "Lists")
                    .on_hover_text("Fold nested list hierarchies")
                    .changed()
                {
                    changed = true;
                }

                if ui
                    .checkbox(&mut settings.fold_indentation, "Indentation (JSON/YAML)")
                    .on_hover_text("Fold indentation-based structures in JSON/YAML files")
                    .changed()
                {
                    changed = true;
                }
            });
        }

        changed
    }

    /// Show the Files settings section.
    ///
    /// Returns true if any setting was changed.
    fn show_files_section(&mut self, ui: &mut Ui, settings: &mut Settings) -> bool {
        let mut changed = false;

        ui.heading("Files");
        ui.add_space(8.0);

        // Auto-save toggle (default for new documents)
        if ui
            .checkbox(&mut settings.auto_save_enabled_default, "Enable Auto-Save by Default")
            .on_hover_text("New documents will have auto-save enabled. Uses temp files to prevent data loss.")
            .changed()
        {
            changed = true;
        }

        ui.add_space(4.0);

        // Auto-save delay
        ui.horizontal(|ui| {
            ui.label("Auto-save delay:");
            ui.add_space(8.0);
            let secs = settings.auto_save_delay_ms / 1000;
            ui.label(format!("{} seconds", secs));
        });
        ui.add_space(4.0);

        // Convert ms to seconds for slider display
        let mut delay_secs = (settings.auto_save_delay_ms / 1000) as f32;
        let delay_slider = ui.add(
            egui::Slider::new(&mut delay_secs, 5.0..=300.0)
                .show_value(false)
                .step_by(5.0),
        );
        if delay_slider.changed() {
            settings.auto_save_delay_ms = (delay_secs as u32) * 1000;
            changed = true;
        }

        // Delay presets
        ui.horizontal(|ui| {
            for (label, ms) in [("15s", 15000), ("30s", 30000), ("1m", 60000)] {
                if ui.small_button(label).clicked() {
                    settings.auto_save_delay_ms = ms;
                    changed = true;
                }
            }
        });

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Recent files count
        ui.horizontal(|ui| {
            ui.label(RichText::new("Recent Files").strong());
            ui.add_space(8.0);
            ui.label(format!("Remember {} files", settings.max_recent_files));
        });
        ui.add_space(4.0);

        let mut recent_count_f32 = settings.max_recent_files as f32;
        let recent_slider = ui.add(
            egui::Slider::new(&mut recent_count_f32, 0.0..=20.0)
                .show_value(false)
                .step_by(1.0),
        );
        if recent_slider.changed() {
            settings.max_recent_files = recent_count_f32 as usize;
            changed = true;
        }

        ui.add_space(8.0);

        // Clear recent files button
        ui.horizontal(|ui| {
            if ui
                .button("Clear Recent Files")
                .on_hover_text("Remove all files from the recent files list")
                .clicked()
            {
                settings.recent_files.clear();
                changed = true;
            }

            if !settings.recent_files.is_empty() {
                ui.label(
                    RichText::new(format!("({} files)", settings.recent_files.len()))
                        .small()
                        .weak(),
                );
            }
        });

        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_panel_new() {
        let panel = SettingsPanel::new();
        assert_eq!(panel.active_section, SettingsSection::Appearance);
    }

    #[test]
    fn test_settings_panel_default() {
        let panel = SettingsPanel::default();
        assert_eq!(panel.active_section, SettingsSection::Appearance);
    }

    #[test]
    fn test_settings_section_label() {
        assert_eq!(SettingsSection::Appearance.label(), "Appearance");
        assert_eq!(SettingsSection::Editor.label(), "Editor");
        assert_eq!(SettingsSection::Files.label(), "Files");
    }

    #[test]
    fn test_settings_section_icon() {
        assert_eq!(SettingsSection::Appearance.icon(), "🎨");
        assert_eq!(SettingsSection::Editor.icon(), "📝");
        assert_eq!(SettingsSection::Files.icon(), "📁");
    }

    #[test]
    fn test_settings_section_default() {
        let section = SettingsSection::default();
        assert_eq!(section, SettingsSection::Appearance);
    }

    #[test]
    fn test_settings_panel_output_default() {
        let output = SettingsPanelOutput::default();
        assert!(!output.changed);
        assert!(!output.close_requested);
        assert!(!output.reset_requested);
    }
}
