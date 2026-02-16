//! Title bar rendering for the Ferrite application.
//!
//! This module renders the custom title bar with window controls,
//! app icon, file title, auto-save indicator, view mode segment,
//! and minimize/maximize/close buttons.

use super::FerriteApp;
use crate::config::ViewMode;
use crate::state::FileType;
use crate::ui::{TitleBarButton, ViewModeSegment, ViewSegmentAction};
use eframe::egui;
use log::debug;
use rust_i18n::t;

impl FerriteApp {
    /// Render the custom title bar panel.
    pub(crate) fn render_title_bar(
        &mut self,
        ctx: &egui::Context,
        is_maximized: bool,
        is_dark: bool,
        zen_mode: bool,
        title_bar_color: egui::Color32,
        button_hover_color: egui::Color32,
        close_hover_color: egui::Color32,
        text_color: egui::Color32,
    ) {
        egui::TopBottomPanel::top("title_bar")
            .frame(
                egui::Frame::none()
                    .fill(title_bar_color)
                    .stroke(egui::Stroke::NONE)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show_separator_line(false)
            .show(ctx, |ui| {
                // Remove spacing between elements
                ui.spacing_mut().item_spacing.y = 0.0;

                // Add top padding for title bar
                ui.add_space(5.0);

                // Get state needed for title bar controls
                let has_editor = self.state.active_tab().is_some();
                let is_special_tab = self.state.active_tab()
                    .map(|t| t.is_special())
                    .unwrap_or(false);
                let auto_save_enabled = self.state.active_tab()
                    .map(|t| t.auto_save_enabled)
                    .unwrap_or(false);
                let current_view_mode = self.state.active_tab()
                    .map(|t| t.view_mode)
                    .unwrap_or(ViewMode::Raw);
                let current_file_type = self.state.active_tab()
                    .map(|t| t.file_type())
                    .unwrap_or(FileType::Unknown);
                let zen_mode_active = self.state.is_zen_mode();

                // Track title bar actions
                let mut title_bar_toggle_auto_save = false;
                let mut title_bar_toggle_zen = false;
                let mut title_bar_open_settings = false;
                let mut title_bar_view_action: Option<ViewSegmentAction> = None;

                // Title bar row - set consistent height and center alignment
                let title_bar_height = 28.0;
                ui.set_height(title_bar_height);
                
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);

                    // App icon/logo - display texture if available, fallback to emoji
                    if let Some(texture) = &self.app_logo_texture {
                        let logo_size = 18.0; // Match title bar height nicely
                        ui.add(egui::Image::new(texture).fit_to_exact_size(egui::vec2(logo_size, logo_size)));
                    } else {
                        ui.label(egui::RichText::new("📝").size(14.0));
                    }

                    ui.add_space(4.0); // Reduced spacing between icon and title

                    // Window title (dynamically generated) - use consistent sizing
                    // Offset text slightly upward to better align with icon center
                    let title = self.window_title();
                    ui.add(egui::Label::new(egui::RichText::new(title).size(12.0).color(text_color)).selectable(false));

                    // Auto-save indicator (after filename) - only show for document tabs
                    if has_editor && !is_special_tab {
                        ui.add_space(8.0);
                        if TitleBarButton::show_auto_save(ui, auto_save_enabled, is_dark).clicked() {
                            title_bar_toggle_auto_save = true;
                        }
                    }

                    // Fill remaining space with draggable area, but EXCLUDE the button area
                    // on the right side to prevent drag response from consuming clicks
                    // intended for window control buttons. This fixes Linux hit-testing issues.
                    //
                    // Button area width calculation (right-to-left):
                    // - 4.0 spacing + Close(46) + Max(46) + Min(46) + Fullscreen(46) + 8.0 spacing = 196px
                    // - Settings(28) + 4.0 + Zen(28) + 4.0 = 64px
                    // - ViewModeSegment (3 x 26px) = 78px (or 2 x 26px = 52px for 2-mode)
                    // Total ~338px + extra margin for safety = 400px
                    const WINDOW_BUTTON_AREA_WIDTH: f32 = 400.0;
                    
                    let available = ui.available_rect_before_wrap();
                    let drag_width = (available.width() - WINDOW_BUTTON_AREA_WIDTH).max(0.0);
                    let drag_rect = egui::Rect::from_min_size(
                        available.min,
                        egui::vec2(drag_width, available.height()),
                    );
                    
                    // IMPORTANT: We use Sense::hover() and handle drag detection manually via
                    // raw input state. This is necessary because:
                    //
                    // 1. When StartDrag is sent, the window manager takes over the drag operation
                    // 2. egui doesn't receive the mouse release event (WM handles it)
                    // 3. egui's widget interaction state gets confused, thinking the widget
                    //    is still being interacted with
                    // 4. On the next click, drag_started() doesn't fire because egui thinks
                    //    we're continuing an existing interaction
                    //
                    // By using raw input state (primary_pressed), we bypass egui's widget-level
                    // tracking entirely and get reliable drag detection every time.
                    let drag_response = ui.allocate_rect(drag_rect, egui::Sense::hover());
                    
                    // Get raw pointer state - this is always accurate regardless of widget state
                    let (primary_pressed, double_clicked, pointer_pos) = ctx.input(|i| (
                        i.pointer.primary_pressed(),
                        i.pointer.button_double_clicked(egui::PointerButton::Primary),
                        i.pointer.interact_pos(),
                    ));
                    
                    // Check if pointer is in the drag area
                    let pointer_in_drag_area = pointer_pos
                        .map(|pos| drag_rect.contains(pos))
                        .unwrap_or(false);

                    // Handle double-click to maximize/restore
                    if double_clicked && pointer_in_drag_area {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                    }

                    // Handle drag to move window (but not if we're in a resize zone)
                    //
                    // We use primary_pressed() which is only true on the FRAME the button
                    // is pressed down. This ensures StartDrag is sent exactly once per click,
                    // preventing the "mouse stuck" bug on Linux.
                    let is_in_resize = self.window_resize_state.current_direction().is_some()
                        || self.window_resize_state.is_resizing();
                    
                    if primary_pressed && pointer_in_drag_area && !is_in_resize {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                    
                    // Still use the response for hover effects if needed
                    let _ = drag_response;

                    // Window control buttons (right-to-left)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(4.0);

                        // Close button (×)
                        let close_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("×").size(16.0).color(text_color),
                            )
                            .frame(false)
                            .min_size(egui::vec2(46.0, 28.0)),
                        );
                        if close_btn.hovered() {
                            ui.painter()
                                .rect_filled(close_btn.rect, 0.0, close_hover_color);
                            ui.painter().text(
                                close_btn.rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "×",
                                egui::FontId::proportional(16.0),
                                egui::Color32::WHITE,
                            );
                        }
                        if close_btn.clicked() && self.state.request_exit() {
                            self.should_exit = true;
                        }
                        close_btn.on_hover_text(t!("a11y.close_button").to_string());

                        // Maximize/Restore button - draw custom icon like minimize/fullscreen
                        // to prevent hover from hiding the icon
                        let max_btn = ui.add(
                            egui::Button::new(egui::RichText::new(" ").size(14.0))
                                .frame(false)
                                .min_size(egui::vec2(46.0, 28.0)),
                        );
                        if max_btn.hovered() {
                            ui.painter()
                                .rect_filled(max_btn.rect, 0.0, button_hover_color);
                        }
                        // Draw maximize/restore icon manually (always visible, even on hover)
                        let max_center = max_btn.rect.center();
                        let stroke = egui::Stroke::new(1.5, text_color);
                        if is_maximized {
                            // Restore icon: two overlapping rectangles
                            let size = 4.5;
                            let offset = 2.0;
                            // Back rectangle (offset up-right)
                            let back_min = egui::pos2(max_center.x - size + offset, max_center.y - size - offset);
                            let back_max = egui::pos2(max_center.x + size + offset, max_center.y + size - offset);
                            let back_top_right = egui::pos2(back_max.x, back_min.y);
                            // Only draw the visible parts of the back rectangle (top and right edges)
                            ui.painter().line_segment([egui::pos2(back_min.x + size, back_min.y), back_top_right], stroke);
                            ui.painter().line_segment([back_top_right, back_max], stroke);
                            ui.painter().line_segment([back_max, egui::pos2(back_min.x + size, back_max.y)], stroke);
                            // Front rectangle (main)
                            let front_rect = egui::Rect::from_center_size(
                                egui::pos2(max_center.x - offset / 2.0, max_center.y + offset / 2.0),
                                egui::vec2(size * 2.0, size * 2.0),
                            );
                            ui.painter().rect_stroke(front_rect, 0.0, stroke);
                        } else {
                            // Maximize icon: single rectangle
                            let size = 5.0;
                            let rect = egui::Rect::from_center_size(
                                max_center,
                                egui::vec2(size * 2.0, size * 2.0),
                            );
                            ui.painter().rect_stroke(rect, 0.0, stroke);
                        }
                        if max_btn.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                        }
                        let max_tooltip = if is_maximized { "Restore" } else { "Maximize" };
                        max_btn.on_hover_text(max_tooltip);

                        // Minimize button - draw a line
                        let min_btn = ui.add(
                            egui::Button::new(egui::RichText::new(" ").size(14.0))
                                .frame(false)
                                .min_size(egui::vec2(46.0, 28.0)),
                        );
                        if min_btn.hovered() {
                            ui.painter()
                                .rect_filled(min_btn.rect, 0.0, button_hover_color);
                        }
                        let center = min_btn.rect.center();
                        ui.painter().line_segment(
                            [
                                egui::pos2(center.x - 5.0, center.y),
                                egui::pos2(center.x + 5.0, center.y),
                            ],
                            egui::Stroke::new(1.5, text_color),
                        );
                        if min_btn.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                        min_btn.on_hover_text(t!("a11y.minimize_button").to_string());

                        // Fullscreen button - draw expand arrows icon
                        let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                        let fullscreen_btn = ui.add(
                            egui::Button::new(egui::RichText::new(" ").size(14.0))
                                .frame(false)
                                .min_size(egui::vec2(46.0, 28.0)),
                        );
                        if fullscreen_btn.hovered() || is_fullscreen {
                            ui.painter()
                                .rect_filled(fullscreen_btn.rect, 0.0, button_hover_color);
                        }
                        // Draw fullscreen icon (4 arrows pointing outward, or inward when in fullscreen)
                        let fs_center = fullscreen_btn.rect.center();
                        let arrow_len = 3.5;
                        let arrow_offset = 4.0;
                        let stroke = egui::Stroke::new(1.5, text_color);
                        if is_fullscreen {
                            // Inward arrows (exit fullscreen) - draw 4 arrows pointing to center
                            ui.painter().line_segment([egui::pos2(fs_center.x - arrow_offset, fs_center.y - arrow_offset), egui::pos2(fs_center.x - arrow_offset + arrow_len, fs_center.y - arrow_offset + arrow_len)], stroke);
                            ui.painter().line_segment([egui::pos2(fs_center.x + arrow_offset, fs_center.y - arrow_offset), egui::pos2(fs_center.x + arrow_offset - arrow_len, fs_center.y - arrow_offset + arrow_len)], stroke);
                            ui.painter().line_segment([egui::pos2(fs_center.x - arrow_offset, fs_center.y + arrow_offset), egui::pos2(fs_center.x - arrow_offset + arrow_len, fs_center.y + arrow_offset - arrow_len)], stroke);
                            ui.painter().line_segment([egui::pos2(fs_center.x + arrow_offset, fs_center.y + arrow_offset), egui::pos2(fs_center.x + arrow_offset - arrow_len, fs_center.y + arrow_offset - arrow_len)], stroke);
                        } else {
                            // Outward arrows (enter fullscreen) - draw 4 arrows pointing away from center
                            ui.painter().line_segment([egui::pos2(fs_center.x - arrow_offset + arrow_len, fs_center.y - arrow_offset + arrow_len), egui::pos2(fs_center.x - arrow_offset, fs_center.y - arrow_offset)], stroke);
                            ui.painter().line_segment([egui::pos2(fs_center.x + arrow_offset - arrow_len, fs_center.y - arrow_offset + arrow_len), egui::pos2(fs_center.x + arrow_offset, fs_center.y - arrow_offset)], stroke);
                            ui.painter().line_segment([egui::pos2(fs_center.x - arrow_offset + arrow_len, fs_center.y + arrow_offset - arrow_len), egui::pos2(fs_center.x - arrow_offset, fs_center.y + arrow_offset)], stroke);
                            ui.painter().line_segment([egui::pos2(fs_center.x + arrow_offset - arrow_len, fs_center.y + arrow_offset - arrow_len), egui::pos2(fs_center.x + arrow_offset, fs_center.y + arrow_offset)], stroke);
                        }
                        if fullscreen_btn.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!is_fullscreen));
                        }
                        let fs_tooltip = if is_fullscreen { t!("tooltip.fullscreen_exit") } else { t!("tooltip.fullscreen_enter") };
                        fullscreen_btn.on_hover_text(fs_tooltip.to_string());

                        ui.add_space(8.0);

                        // ΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉ
                        // Title Bar Controls (before window buttons, right-to-left)
                        // Settings ΓåÆ Zen Mode ΓåÆ View Mode Segment
                        // ΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉΓòÉ

                        // Settings button
                        if TitleBarButton::show(ui, "⚙", &t!("tooltip.settings").to_string(), false, is_dark).clicked() {
                            title_bar_open_settings = true;
                        }

                        ui.add_space(4.0);

                        // Zen Mode toggle - use simple "Z" icon for cross-platform compatibility
                        let zen_icon = if zen_mode_active { "Z" } else { "Z" };
                        let zen_tooltip = if zen_mode_active {
                            t!("zen.exit")
                        } else {
                            t!("zen.enter")
                        };
                        if TitleBarButton::show(ui, zen_icon, &format!("{} (F11)", zen_tooltip), zen_mode_active, is_dark).clicked() {
                            title_bar_toggle_zen = true;
                        }

                        ui.add_space(4.0);

                        // View Mode segmented control (only for document tabs with renderable content)
                        if has_editor && !is_special_tab && (current_file_type.is_markdown() || current_file_type.is_structured() || current_file_type.is_tabular()) {
                            // Show the segmented pill control for view mode selection
                            let segment = ViewModeSegment::new();
                            
                            // Use 3-mode segment for markdown/tabular, 2-mode for structured files
                            if current_file_type.is_markdown() || current_file_type.is_tabular() {
                                if let Some(action) = segment.show(ui, current_view_mode, current_file_type, is_dark) {
                                    title_bar_view_action = Some(action);
                                }
                            } else {
                                // Structured files (JSON/YAML/TOML): only Raw <-> Rendered
                                if let Some(action) = segment.show_two_mode(ui, current_view_mode, is_dark) {
                                    title_bar_view_action = Some(action);
                                }
                            }
                        }
                    });
                });

                ui.add_space(2.0);

                // Handle title bar actions (deferred to avoid borrow conflicts)
                if title_bar_toggle_auto_save {
                    if let Some(tab) = self.state.active_tab_mut() {
                        tab.toggle_auto_save();
                        debug!("Title bar: Toggle auto-save -> {}", tab.auto_save_enabled);
                    }
                }
                if title_bar_toggle_zen {
                    self.state.toggle_zen_mode();
                    debug!("Title bar: Toggle Zen Mode");
                }
                if title_bar_open_settings {
                    self.state.open_settings_tab();
                    debug!("Title bar: Open Settings tab");
                }
                if let Some(view_action) = title_bar_view_action {
                    if let Some(tab) = self.state.active_tab_mut() {
                        let new_mode = match view_action {
                            ViewSegmentAction::SetRaw => ViewMode::Raw,
                            ViewSegmentAction::SetSplit => ViewMode::Split,
                            ViewSegmentAction::SetRendered => ViewMode::Rendered,
                        };
                        tab.view_mode = new_mode;
                        debug!("Title bar: Set view mode to {:?}", new_mode);
                    }
                }
            });
    }
}
