//! Title bar rendering for the MDOxide application.
//!
//! MarkText-style frameless title bar: application menu on the left, centred
//! document path/filename, document statistics and window controls on the right.

use super::MDOxideApp;
use crate::config::ViewMode;
use crate::markdown::formatting::MarkdownFormatCommand;
use crate::state::FileType;
use crate::ui::phosphor_icons::{phosphor_font, LIST};
use crate::ui::RibbonAction;
use eframe::egui;
use rust_i18n::t;
use std::path::PathBuf;

/// MarkText's title bar is 32px tall.
const TITLE_BAR_HEIGHT: f32 = 32.0;

/// Left cluster holding the application-menu button.
const MENU_BUTTON_AREA_WIDTH: f32 = 44.0;
/// Right cluster holding word count and minimize/maximize/close.
const WINDOW_BUTTON_AREA_WIDTH: f32 = 190.0;

/// Hit area for minimize / maximize / close.
const WINDOW_CHROME_BTN_SIZE: egui::Vec2 = egui::vec2(29.0, 20.0);
/// Scale factor for chrome icon geometry.
const WINDOW_CHROME_ICON_SCALE: f32 = 0.8;

/// Actions emitted by the title bar so they can run outside the panel closure.
enum TitleBarAction {
    Ribbon(RibbonAction),
    Format(MarkdownFormatCommand),
    SetView(ViewMode),
    ToggleLineNumbers,
    ToggleOutline,
    ToggleTerminal,
    ToggleProductivity,
    TogglePipeline,
    CycleTheme,
    ToggleZen,
    ToggleFullscreen,
    OpenSettings,
    About,
    Exit,
    OpenRecent(PathBuf),
}

impl MDOxideApp {
    /// Render the custom title bar panel.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_title_bar(
        &mut self,
        ui: &mut egui::Ui,
        is_maximized: bool,
        _is_dark: bool,
        _zen_mode: bool,
        title_bar_color: egui::Color32,
        button_hover_color: egui::Color32,
        close_hover_color: egui::Color32,
        text_color: egui::Color32,
    ) {
        let ctx = ui.ctx().clone();
        let accent = self.state.settings.mdoxide_accent_rgb();
        let muted_color = egui::Color32::from_rgba_unmultiplied(
            text_color.r(),
            text_color.g(),
            text_color.b(),
            150,
        );

        // Snapshot state needed by the menus before the UI closure borrows self.
        // CJK text has no spaces, so whitespace word counts are meaningless;
        // show characters for those documents instead (MarkText does the same).
        let doc_metric: Option<(usize, String)> =
            self.state.active_tab_mut().map(|tab| {
                let stats = tab.text_stats();
                if tab.needs_cjk_cached() {
                    (
                        stats.characters_no_spaces,
                        t!("stats.characters").to_string(),
                    )
                } else {
                    (stats.words, t!("stats.words").to_string())
                }
            });
        let is_workspace_mode = self.state.is_workspace_mode();
        let is_markdown = self
            .state
            .active_tab()
            .map(|tab| tab.file_type() == FileType::Markdown)
            .unwrap_or(false);
        let file_type = self
            .state
            .active_tab()
            .map(|tab| tab.file_type())
            .unwrap_or(FileType::Unknown);
        let supports_split = file_type.supports_split();
        let recent_files: Vec<PathBuf> = self
            .state
            .settings
            .recent_files
            .iter()
            .take(10)
            .cloned()
            .collect();
        let (title_text, path_text, modified) = match self.state.active_tab() {
            Some(tab) => (
                tab.title(),
                tab.path
                    .as_ref()
                    .and_then(|p| p.parent())
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().into_owned()),
                tab.is_modified(),
            ),
            None => (t!("app.name").to_string(), None, false),
        };

        let mut action: Option<TitleBarAction> = None;

        egui::Panel::top("title_bar")
            .frame(
                egui::Frame::new()
                    .fill(title_bar_color)
                    .stroke(egui::Stroke::NONE)
                    .inner_margin(egui::Margin::ZERO),
            )
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                ui.set_height(TITLE_BAR_HEIGHT);

                let bar_rect = egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::vec2(ui.available_width(), TITLE_BAR_HEIGHT),
                );
                // Reserve the strip first so the panel auto-sizes to 32px and the
                // menu/window buttons added afterwards stay on top for hover.
                ui.allocate_rect(bar_rect, egui::Sense::hover());
                let left_rect = egui::Rect::from_min_size(
                    bar_rect.min,
                    egui::vec2(MENU_BUTTON_AREA_WIDTH, TITLE_BAR_HEIGHT),
                );
                let right_rect = egui::Rect::from_min_max(
                    egui::pos2(bar_rect.right() - WINDOW_BUTTON_AREA_WIDTH, bar_rect.top()),
                    bar_rect.max,
                );

                // ── Application menu (MarkText's frameless hamburger) ──────────
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(left_rect)
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    |ui| {
                        ui.add_space(8.0);
                        ui.menu_button(
                            egui::RichText::new(LIST)
                                .font(phosphor_font(17.0))
                                .color(text_color),
                            |ui| {
                                ui.set_min_width(180.0);

                                // File
                                ui.menu_button(t!("menu.file.label"), |ui| {
                                    menu_action(
                                        ui,
                                        t!("shortcuts.commands.new"),
                                        || TitleBarAction::Ribbon(RibbonAction::New),
                                        &mut action,
                                    );
                                    menu_action(
                                        ui,
                                        t!("shortcuts.commands.open"),
                                        || TitleBarAction::Ribbon(RibbonAction::Open),
                                        &mut action,
                                    );
                                    if is_workspace_mode {
                                        menu_action(
                                            ui,
                                            t!("shortcuts.commands.close_workspace"),
                                            || TitleBarAction::Ribbon(RibbonAction::CloseWorkspace),
                                            &mut action,
                                        );
                                    } else {
                                        menu_action(
                                            ui,
                                            t!("shortcuts.commands.open_workspace"),
                                            || TitleBarAction::Ribbon(RibbonAction::OpenWorkspace),
                                            &mut action,
                                        );
                                    }
                                    ui.separator();
                                    menu_action(
                                        ui,
                                        t!("menu.file.save"),
                                        || TitleBarAction::Ribbon(RibbonAction::Save),
                                        &mut action,
                                    );
                                    menu_action(
                                        ui,
                                        t!("menu.file.save_as"),
                                        || TitleBarAction::Ribbon(RibbonAction::SaveAs),
                                        &mut action,
                                    );
                                    ui.separator();
                                    menu_action(
                                        ui,
                                        t!("menu.file.export_html"),
                                        || TitleBarAction::Ribbon(RibbonAction::ExportHtml),
                                        &mut action,
                                    );
                                    menu_action(
                                        ui,
                                        t!("menu.file.export_clipboard"),
                                        || TitleBarAction::Ribbon(RibbonAction::CopyAsHtml),
                                        &mut action,
                                    );
                                    menu_action(
                                        ui,
                                        t!("menu.file.export_pdf"),
                                        || TitleBarAction::Ribbon(RibbonAction::ExportPdf),
                                        &mut action,
                                    );
                                    menu_action(
                                        ui,
                                        t!("menu.file.print_preview"),
                                        || TitleBarAction::Ribbon(RibbonAction::PrintPreview),
                                        &mut action,
                                    );
                                    if !recent_files.is_empty() {
                                        ui.separator();
                                        ui.menu_button(t!("menu.file.recent"), |ui| {
                                            for path in &recent_files {
                                                let label = path
                                                    .file_name()
                                                    .map(|n| n.to_string_lossy().into_owned())
                                                    .unwrap_or_else(|| path.display().to_string());
                                                if ui.button(label).clicked() {
                                                    action = Some(TitleBarAction::OpenRecent(
                                                        path.clone(),
                                                    ));
                                                    ui.close();
                                                }
                                            }
                                        });
                                    }
                                    ui.separator();
                                    menu_action(
                                        ui,
                                        t!("shortcuts.commands.open_settings"),
                                        || TitleBarAction::OpenSettings,
                                        &mut action,
                                    );
                                    menu_action(
                                        ui,
                                        t!("menu.file.exit"),
                                        || TitleBarAction::Exit,
                                        &mut action,
                                    );
                                });

                                // Edit
                                ui.menu_button(t!("menu.edit.label"), |ui| {
                                    menu_action(
                                        ui,
                                        t!("shortcuts.commands.undo"),
                                        || TitleBarAction::Ribbon(RibbonAction::Undo),
                                        &mut action,
                                    );
                                    menu_action(
                                        ui,
                                        t!("shortcuts.commands.redo"),
                                        || TitleBarAction::Ribbon(RibbonAction::Redo),
                                        &mut action,
                                    );
                                    ui.separator();
                                    menu_action(
                                        ui,
                                        t!("shortcuts.commands.find_replace"),
                                        || TitleBarAction::Ribbon(RibbonAction::FindReplace),
                                        &mut action,
                                    );
                                });

                                // Format
                                if is_markdown {
                                    ui.menu_button(t!("menu.format.label"), |ui| {
                                        menu_action(
                                            ui,
                                            t!("shortcuts.commands.bold"),
                                            || TitleBarAction::Format(MarkdownFormatCommand::Bold),
                                            &mut action,
                                        );
                                        menu_action(
                                            ui,
                                            t!("shortcuts.commands.italic"),
                                            || {
                                                TitleBarAction::Format(
                                                    MarkdownFormatCommand::Italic,
                                                )
                                            },
                                            &mut action,
                                        );
                                        menu_action(
                                            ui,
                                            t!("shortcuts.commands.inline_code"),
                                            || {
                                                TitleBarAction::Format(
                                                    MarkdownFormatCommand::InlineCode,
                                                )
                                            },
                                            &mut action,
                                        );
                                        menu_action(
                                            ui,
                                            t!("shortcuts.commands.link"),
                                            || TitleBarAction::Format(MarkdownFormatCommand::Link),
                                            &mut action,
                                        );
                                        menu_action(
                                            ui,
                                            t!("shortcuts.commands.image"),
                                            || TitleBarAction::Format(MarkdownFormatCommand::Image),
                                            &mut action,
                                        );
                                        ui.separator();
                                        for (level, label) in [
                                            (1, t!("shortcuts.commands.heading_1")),
                                            (2, t!("shortcuts.commands.heading_2")),
                                            (3, t!("shortcuts.commands.heading_3")),
                                        ] {
                                            menu_action(
                                                ui,
                                                label,
                                                || {
                                                    TitleBarAction::Format(
                                                        MarkdownFormatCommand::Heading(level),
                                                    )
                                                },
                                                &mut action,
                                            );
                                        }
                                        ui.separator();
                                        menu_action(
                                            ui,
                                            t!("shortcuts.commands.bullet_list"),
                                            || {
                                                TitleBarAction::Format(
                                                    MarkdownFormatCommand::BulletList,
                                                )
                                            },
                                            &mut action,
                                        );
                                        menu_action(
                                            ui,
                                            t!("shortcuts.commands.numbered_list"),
                                            || {
                                                TitleBarAction::Format(
                                                    MarkdownFormatCommand::NumberedList,
                                                )
                                            },
                                            &mut action,
                                        );
                                        menu_action(
                                            ui,
                                            t!("shortcuts.commands.blockquote"),
                                            || {
                                                TitleBarAction::Format(
                                                    MarkdownFormatCommand::Blockquote,
                                                )
                                            },
                                            &mut action,
                                        );
                                        menu_action(
                                            ui,
                                            t!("shortcuts.commands.code_block"),
                                            || {
                                                TitleBarAction::Format(
                                                    MarkdownFormatCommand::CodeBlock,
                                                )
                                            },
                                            &mut action,
                                        );
                                    });
                                }

                                // View
                                ui.menu_button(t!("menu.view.label"), |ui| {
                                    menu_action(
                                        ui,
                                        t!("menu.view.raw"),
                                        || TitleBarAction::SetView(ViewMode::Raw),
                                        &mut action,
                                    );
                                    if supports_split {
                                        menu_action(
                                            ui,
                                            t!("menu.view.split"),
                                            || TitleBarAction::SetView(ViewMode::Split),
                                            &mut action,
                                        );
                                    }
                                    menu_action(
                                        ui,
                                        t!("menu.view.rendered"),
                                        || TitleBarAction::SetView(ViewMode::Rendered),
                                        &mut action,
                                    );
                                    ui.separator();
                                    menu_action(
                                        ui,
                                        t!("menu.view.line_numbers"),
                                        || TitleBarAction::ToggleLineNumbers,
                                        &mut action,
                                    );
                                    menu_action(
                                        ui,
                                        t!("shortcuts.commands.toggle_outline"),
                                        || TitleBarAction::ToggleOutline,
                                        &mut action,
                                    );
                                    menu_action(
                                        ui,
                                        t!("shortcuts.commands.toggle_terminal"),
                                        || TitleBarAction::ToggleTerminal,
                                        &mut action,
                                    );
                                    menu_action(
                                        ui,
                                        t!("shortcuts.commands.toggle_productivity_hub"),
                                        || TitleBarAction::ToggleProductivity,
                                        &mut action,
                                    );
                                    if matches!(file_type, FileType::Json | FileType::Yaml) {
                                        menu_action(
                                            ui,
                                            t!("shortcuts.commands.toggle_pipeline"),
                                            || TitleBarAction::TogglePipeline,
                                            &mut action,
                                        );
                                    }
                                    ui.separator();
                                    menu_action(
                                        ui,
                                        t!("shortcuts.commands.cycle_theme"),
                                        || TitleBarAction::CycleTheme,
                                        &mut action,
                                    );
                                    menu_action(
                                        ui,
                                        t!("shortcuts.commands.toggle_zen_mode"),
                                        || TitleBarAction::ToggleZen,
                                        &mut action,
                                    );
                                    menu_action(
                                        ui,
                                        t!("shortcuts.commands.toggle_fullscreen"),
                                        || TitleBarAction::ToggleFullscreen,
                                        &mut action,
                                    );
                                });

                                // Tools (document-level actions from the old ribbon)
                                if is_markdown || file_type.is_structured() {
                                    ui.menu_button(t!("menu.tools.label"), |ui| {
                                        if is_markdown {
                                            menu_action(
                                                ui,
                                                t!("shortcuts.commands.insert_toc"),
                                                || {
                                                    TitleBarAction::Ribbon(
                                                        RibbonAction::InsertToc,
                                                    )
                                                },
                                                &mut action,
                                            );
                                        }
                                        if file_type.is_structured() {
                                            menu_action(
                                                ui,
                                                t!("ribbon.format_document"),
                                                || {
                                                    TitleBarAction::Ribbon(
                                                        RibbonAction::FormatDocument,
                                                    )
                                                },
                                                &mut action,
                                            );
                                            menu_action(
                                                ui,
                                                t!("ribbon.validate_syntax"),
                                                || {
                                                    TitleBarAction::Ribbon(
                                                        RibbonAction::ValidateSyntax,
                                                    )
                                                },
                                                &mut action,
                                            );
                                        }
                                    });
                                }

                                // Help
                                ui.menu_button(t!("menu.help.label"), |ui| {
                                    menu_action(
                                        ui,
                                        t!("shortcuts.commands.open_about"),
                                        || TitleBarAction::About,
                                        &mut action,
                                    );
                                });
                            },
                        );
                    },
                );

                // ── Centred document title ─────────────────────────────────────
                let title_rect = egui::Rect::from_min_max(
                    egui::pos2(left_rect.right() + 6.0, bar_rect.top()),
                    egui::pos2(right_rect.left() - 6.0, bar_rect.bottom()),
                );
                if title_rect.width() > 20.0 {
                    let mut job = egui::text::LayoutJob::default();
                    job.wrap.max_width = title_rect.width();
                    job.wrap.max_rows = 1;
                    job.wrap.break_anywhere = true;
                    job.wrap.overflow_character = Some('…');

                    let title_format = |color: egui::Color32, size: f32| egui::text::TextFormat {
                        font_id: egui::FontId::proportional(size),
                        color,
                        ..Default::default()
                    };
                    if modified {
                        job.append("● ", 0.0, title_format(accent, 10.0));
                    }
                    if let Some(path_text) = &path_text {
                        job.append(
                            &format!("{path_text}   "),
                            0.0,
                            title_format(muted_color, 12.5),
                        );
                    }
                    job.append(&title_text, 0.0, title_format(text_color, 13.0));

                    let galley = ui.fonts_mut(|f| f.layout_job(job));
                    let pos = egui::pos2(
                        title_rect.center().x - galley.size().x / 2.0,
                        title_rect.center().y - galley.size().y / 2.0,
                    );
                    ui.painter().galley(pos, galley, text_color);
                }

                // ── Window drag / double-click region ─────────────────────────
                let drag_rect = egui::Rect::from_min_max(
                    egui::pos2(left_rect.right(), bar_rect.top()),
                    egui::pos2(right_rect.left(), bar_rect.bottom()),
                );
                let (primary_pressed, double_clicked, pointer_pos) = ctx.input(|i| {
                    (
                        i.pointer.primary_pressed(),
                        i.pointer
                            .button_double_clicked(egui::PointerButton::Primary),
                        i.pointer.interact_pos(),
                    )
                });
                let pointer_in_drag_area = pointer_pos
                    .map(|pos| drag_rect.contains(pos))
                    .unwrap_or(false);
                let is_in_resize = self.window_resize_state.current_direction().is_some()
                    || self.window_resize_state.is_resizing();

                if double_clicked && pointer_in_drag_area {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                }
                if primary_pressed && pointer_in_drag_area && !is_in_resize {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                // ── Right cluster: word count + window controls ───────────────
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(right_rect)
                        .layout(egui::Layout::right_to_left(egui::Align::Center)),
                    |ui| {
                        ui.add_space(6.0);

                        // Close
                        let close_btn = chrome_button(ui, close_hover_color);
                        let close_icon_color = if close_btn.hovered() {
                            egui::Color32::WHITE
                        } else {
                            text_color
                        };
                        {
                            let c = close_btn.rect.center();
                            let d = 5.5_f32 * WINDOW_CHROME_ICON_SCALE;
                            let stroke = egui::Stroke::new(1.5, close_icon_color);
                            ui.painter().line_segment(
                                [egui::pos2(c.x - d, c.y - d), egui::pos2(c.x + d, c.y + d)],
                                stroke,
                            );
                            ui.painter().line_segment(
                                [egui::pos2(c.x + d, c.y - d), egui::pos2(c.x - d, c.y + d)],
                                stroke,
                            );
                        }
                        if close_btn.clicked() && self.state.request_exit() {
                            self.should_exit = true;
                        }
                        close_btn.on_hover_text(t!("a11y.close_button").to_string());

                        // Maximize / restore
                        let max_btn = chrome_button(ui, button_hover_color);
                        let max_center = max_btn.rect.center();
                        let stroke = egui::Stroke::new(1.5, text_color);
                        if is_maximized {
                            let size = 4.5 * WINDOW_CHROME_ICON_SCALE;
                            let offset = 2.0 * WINDOW_CHROME_ICON_SCALE;
                            let back_min = egui::pos2(
                                max_center.x - size + offset,
                                max_center.y - size - offset,
                            );
                            let back_max = egui::pos2(
                                max_center.x + size + offset,
                                max_center.y + size - offset,
                            );
                            let back_top_right = egui::pos2(back_max.x, back_min.y);
                            ui.painter().line_segment(
                                [egui::pos2(back_min.x + size, back_min.y), back_top_right],
                                stroke,
                            );
                            ui.painter()
                                .line_segment([back_top_right, back_max], stroke);
                            ui.painter().line_segment(
                                [back_max, egui::pos2(back_min.x + size, back_max.y)],
                                stroke,
                            );
                            let front_rect = egui::Rect::from_center_size(
                                egui::pos2(
                                    max_center.x - offset / 2.0,
                                    max_center.y + offset / 2.0,
                                ),
                                egui::vec2(size * 2.0, size * 2.0),
                            );
                            ui.painter().rect_stroke(
                                front_rect,
                                0.0,
                                stroke,
                                egui::StrokeKind::Inside,
                            );
                        } else {
                            let size = 5.5 * WINDOW_CHROME_ICON_SCALE;
                            let rect = egui::Rect::from_center_size(
                                max_center,
                                egui::vec2(size * 2.0, size * 2.0),
                            );
                            let top_stroke =
                                egui::Stroke::new(2.0 * WINDOW_CHROME_ICON_SCALE, text_color);
                            ui.painter().line_segment(
                                [
                                    egui::pos2(rect.min.x, rect.min.y),
                                    egui::pos2(rect.max.x, rect.min.y),
                                ],
                                top_stroke,
                            );
                            ui.painter().line_segment(
                                [
                                    egui::pos2(rect.max.x, rect.min.y),
                                    egui::pos2(rect.max.x, rect.max.y),
                                ],
                                stroke,
                            );
                            ui.painter().line_segment(
                                [
                                    egui::pos2(rect.max.x, rect.max.y),
                                    egui::pos2(rect.min.x, rect.max.y),
                                ],
                                stroke,
                            );
                            ui.painter().line_segment(
                                [
                                    egui::pos2(rect.min.x, rect.max.y),
                                    egui::pos2(rect.min.x, rect.min.y),
                                ],
                                stroke,
                            );
                        }
                        if max_btn.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                        }
                        let max_tooltip = if is_maximized {
                            t!("title_bar.restore").to_string()
                        } else {
                            t!("title_bar.maximize").to_string()
                        };
                        max_btn.on_hover_text(max_tooltip);

                        // Minimize
                        let min_btn = chrome_button(ui, button_hover_color);
                        let center = min_btn.rect.center();
                        let min_half = 6.0 * WINDOW_CHROME_ICON_SCALE;
                        ui.painter().line_segment(
                            [
                                egui::pos2(center.x - min_half, center.y),
                                egui::pos2(center.x + min_half, center.y),
                            ],
                            egui::Stroke::new(1.5, text_color),
                        );
                        if min_btn.clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                        min_btn.on_hover_text(t!("a11y.minimize_button").to_string());

                        // Document statistic (MarkText shows this here)
                        if let Some((count, label)) = &doc_metric {
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(format!("{count} {label}"))
                                    .size(11.5)
                                    .color(muted_color),
                            );
                        }
                    },
                );
            });

        // Actions run after the panel closure to avoid borrow conflicts.
        if let Some(action) = action {
            self.handle_title_bar_action(action, &ctx);
        }
    }

    fn handle_title_bar_action(&mut self, action: TitleBarAction, ctx: &egui::Context) {
        match action {
            TitleBarAction::Ribbon(ribbon_action) => {
                self.handle_ribbon_action(ribbon_action, ctx);
            }
            TitleBarAction::Format(command) => {
                self.handle_format_command(ctx, command);
            }
            TitleBarAction::SetView(mode) => {
                if let Some(tab) = self.state.active_tab_mut() {
                    tab.view_mode = mode;
                    self.state.mark_settings_dirty();
                }
            }
            TitleBarAction::ToggleLineNumbers => {
                self.state.settings.show_line_numbers = !self.state.settings.show_line_numbers;
                self.state.mark_settings_dirty();
            }
            TitleBarAction::ToggleOutline => self.handle_toggle_outline(),
            TitleBarAction::ToggleTerminal => self.handle_toggle_terminal(),
            TitleBarAction::ToggleProductivity => {
                if self.state.settings.productivity_panel_docked {
                    if self.state.settings.outline_enabled
                        && self.outline_panel.active_tab()
                            == crate::ui::OutlinePanelTab::Productivity
                    {
                        self.state.settings.outline_enabled = false;
                    } else {
                        self.state.settings.outline_enabled = true;
                        self.outline_panel
                            .set_active_tab(crate::ui::OutlinePanelTab::Productivity);
                    }
                } else {
                    self.state.settings.productivity_panel_visible =
                        !self.state.settings.productivity_panel_visible;
                }
                self.state.mark_settings_dirty();
            }
            TitleBarAction::TogglePipeline => self.handle_toggle_pipeline(),
            TitleBarAction::CycleTheme => self.handle_cycle_theme(ctx),
            TitleBarAction::ToggleZen => self.handle_toggle_zen_mode(),
            TitleBarAction::ToggleFullscreen => self.handle_toggle_fullscreen(ctx),
            TitleBarAction::OpenSettings => self.state.open_settings_tab(),
            TitleBarAction::About => self.state.toggle_about(),
            TitleBarAction::Exit => {
                if self.state.request_exit() {
                    self.should_exit = true;
                }
            }
            TitleBarAction::OpenRecent(path) => {
                let time = self.get_app_time();
                if let Err(error) = self.open_file_smart(path, true, Some(time)) {
                    self.state
                        .show_error(format!("Failed to open file:\n{error}"));
                }
            }
        }
    }
}

/// Render one menu row; clicking it records `action` and closes the menu.
fn menu_action(
    ui: &mut egui::Ui,
    label: impl Into<String>,
    action: impl FnOnce() -> TitleBarAction,
    out: &mut Option<TitleBarAction>,
) {
    if ui.button(label.into()).clicked() {
        *out = Some(action());
        ui.close();
    }
}

/// Transparent window-control button with hover fill.
fn chrome_button(ui: &mut egui::Ui, hover_color: egui::Color32) -> egui::Response {
    let response = ui.add(
        egui::Button::new(egui::RichText::new(" ").size(14.0))
            .frame(false)
            .min_size(WINDOW_CHROME_BTN_SIZE),
    );
    if response.hovered() {
        ui.painter()
            .rect_filled(response.rect, egui::CornerRadius::same(4), hover_color);
    }
    response
}
