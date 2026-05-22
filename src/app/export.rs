//! Export operations for the Ferrite application.
//!
//! This module contains handlers for HTML export and copy-as-HTML.

use super::FerriteApp;
use crate::export::{
    copy_html_to_clipboard, generate_html_document_export, render_markdown_to_pdf,
    resolve_html_theme_for_export, syntax_dark_mode_for_export, PdfTheme,
};
use crate::files::dialogs::detect_linux_desktop;
use crate::state::TabKind;
use eframe::egui;
use log::{debug, info, warn};
use rust_i18n::t;

impl FerriteApp {
    pub(crate) fn handle_open_html_export_dialog(&mut self) {
        if self.state.active_tab().is_none() {
            let time = self.get_app_time();
            self.state
                .show_toast(t!("notification.no_document_export").to_string(), time, 2.0);
            return;
        }
        self.state.ui.show_html_export_dialog = true;
    }

    /// Run HTML export: save dialog, themed generation, toasts.
    pub(crate) fn handle_perform_html_export(&mut self, ctx: &egui::Context) {
        let Some(tab) = self.state.active_tab() else {
            let time = self.get_app_time();
            self.state
                .show_toast(t!("notification.no_document_export").to_string(), time, 2.0);
            return;
        };

        let content = tab.content.clone();
        let source_path = tab.path.clone();

        let initial_dir = source_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .or_else(|| self.state.settings.last_export_directory.clone())
            .or_else(|| {
                self.state
                    .settings
                    .recent_files
                    .first()
                    .and_then(|p| p.parent())
                    .map(|p| p.to_path_buf())
            });

        let default_name = source_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .map(|s| format!("{}.html", s))
            .unwrap_or_else(|| "exported.html".to_string());

        let mut filter = rfd::FileDialog::new()
            .add_filter("HTML Files", &["html", "htm"])
            .set_file_name(&default_name);

        if let Some(dir) = initial_dir.as_ref() {
            filter = filter.set_directory(dir);
        }

        let path = match filter.save_file() {
            Some(p) => p,
            None => {
                let (desktop_env, requires_portal) = detect_linux_desktop();
                if requires_portal {
                    debug!(
                        "Export save dialog returned None on {} (portal-requiring desktop). \
                         If no dialog appeared, check xdg-desktop-portal installation.",
                        desktop_env.as_deref().unwrap_or("unknown")
                    );
                } else {
                    debug!("Export save dialog cancelled");
                }
                return;
            }
        };

        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Exported Document");

        let html_opts = self.state.settings.html_export_options.clone();
        let resolution = resolve_html_theme_for_export(
            html_opts.theme,
            &self.theme_manager,
            self.state.settings.accent_color,
            ctx,
        );
        let syn_dark = syntax_dark_mode_for_export(html_opts.theme, &self.theme_manager, ctx);
        let syn_name = self.state.settings.syntax_theme.clone();

        match generate_html_document_export(
            &content,
            Some(title),
            resolution,
            self.state.settings.paragraph_indent,
            &syn_name,
            syn_dark,
            &html_opts,
            source_path.as_deref(),
        ) {
            Ok(html) => match std::fs::write(&path, html) {
                Ok(()) => {
                    info!("Exported HTML to: {}", path.display());

                    if let Some(parent) = path.parent() {
                        self.state.settings.last_export_directory = Some(parent.to_path_buf());
                        self.state.mark_settings_dirty();
                    }

                    let time = self.get_app_time();
                    self.state.show_toast(
                        t!(
                            "notification.exported_to",
                            path = path.display().to_string()
                        )
                        .to_string(),
                        time,
                        2.5,
                    );

                    if html_opts.open_after_export {
                        if let Err(e) = open::that(&path) {
                            warn!("Failed to open exported file: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to write HTML file: {}", e);
                    let time = self.get_app_time();
                    self.state.show_toast(
                        t!("notification.export_failed", error = e.to_string()).to_string(),
                        time,
                        3.0,
                    );
                }
            },
            Err(e) => {
                warn!("Failed to generate HTML: {}", e);
                let time = self.get_app_time();
                self.state
                    .show_toast(format!("Export failed: {}", e), time, 3.0);
            }
        }
    }

    /// Open the PDF export options dialog. Renders inside `render_dialogs`.
    pub(crate) fn handle_open_pdf_export_dialog(&mut self) {
        if self.state.active_tab().is_none() {
            let time = self.get_app_time();
            self.state
                .show_toast(t!("notification.no_document_export").to_string(), time, 2.0);
            return;
        }
        self.state.ui.show_pdf_export_dialog = true;
    }

    /// Run the PDF export using the currently saved options. Pops the rfd save
    /// dialog, renders, writes, and shows toasts.
    ///
    /// On any failure after a partial file is created, removes the partial
    /// file so the user is not left with a corrupt PDF on disk.
    pub(crate) fn handle_export_pdf(&mut self, ctx: &egui::Context) {
        let Some(tab) = self.state.active_tab() else {
            let time = self.get_app_time();
            self.state
                .show_toast(t!("notification.no_document_export").to_string(), time, 2.0);
            return;
        };

        let content = tab.content.clone();
        let source_path = tab.path.clone();

        let initial_dir = source_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .or_else(|| self.state.settings.last_pdf_export_directory.clone())
            .or_else(|| self.state.settings.last_export_directory.clone())
            .or_else(|| {
                self.state
                    .settings
                    .recent_files
                    .first()
                    .and_then(|p| p.parent())
                    .map(|p| p.to_path_buf())
            });

        let default_name = source_path
            .as_ref()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .map(|s| format!("{}.pdf", s))
            .unwrap_or_else(|| "exported.pdf".to_string());

        let mut dialog = rfd::FileDialog::new()
            .add_filter("PDF Files", &["pdf"])
            .set_file_name(&default_name);
        if let Some(dir) = initial_dir.as_ref() {
            dialog = dialog.set_directory(dir);
        }

        let path = match dialog.save_file() {
            Some(p) => p,
            None => {
                let (desktop_env, requires_portal) = detect_linux_desktop();
                if requires_portal {
                    debug!(
                        "PDF save dialog returned None on {} (portal-requiring desktop). \
                         If no dialog appeared, check xdg-desktop-portal installation.",
                        desktop_env.as_deref().unwrap_or("unknown")
                    );
                } else {
                    debug!("PDF save dialog cancelled");
                }
                return;
            }
        };

        // Resolve theme: use the active editor theme if requested, otherwise
        // a print-friendly default (white page, near-black text).
        let options = self.state.settings.pdf_export_options.clone();
        let theme = if options.use_theme_colors {
            PdfTheme::from_theme_colors(&self.theme_manager.colors(ctx))
        } else {
            PdfTheme::print_default()
        };
        let base_dir = source_path.as_ref().and_then(|p| p.parent());

        match render_markdown_to_pdf(&content, &options, &theme, base_dir) {
            Ok(bytes) => match std::fs::write(&path, &bytes) {
                Ok(()) => {
                    info!(
                        "Exported PDF to: {} ({} bytes)",
                        path.display(),
                        bytes.len()
                    );
                    if let Some(parent) = path.parent() {
                        self.state.settings.last_pdf_export_directory = Some(parent.to_path_buf());
                        self.state.mark_settings_dirty();
                    }
                    let time = self.get_app_time();
                    self.state.show_toast(
                        t!(
                            "notification.exported_to",
                            path = path.display().to_string()
                        )
                        .to_string(),
                        time,
                        2.5,
                    );
                    if options.open_after_export {
                        if let Err(e) = open::that(&path) {
                            warn!("Failed to open exported PDF: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to write PDF file: {}", e);
                    // Best-effort cleanup of any partial file.
                    let _ = std::fs::remove_file(&path);
                    let time = self.get_app_time();
                    self.state.show_toast(
                        t!("notification.export_failed", error = e.to_string()).to_string(),
                        time,
                        3.0,
                    );
                }
            },
            Err(e) => {
                warn!("Failed to render PDF: {}", e);
                let time = self.get_app_time();
                self.state.show_toast(
                    t!("notification.export_failed", error = e.to_string()).to_string(),
                    time,
                    3.0,
                );
            }
        }
    }

    /// Render the active tab through the PDF export pipeline and open the result in
    /// PDF viewer tabs (same code path as **Export PDF**; written to a temp file).
    pub(crate) fn handle_print_preview(&mut self, ctx: &egui::Context) {
        let Some(tab) = self.state.active_tab() else {
            let time = self.get_app_time();
            self.state
                .show_toast(t!("notification.no_document_export").to_string(), time, 2.0);
            return;
        };

        let content = tab.content.clone();
        let source_path = tab.path.clone();
        let options = self.state.settings.pdf_export_options.clone();
        let theme = if options.use_theme_colors {
            PdfTheme::from_theme_colors(&self.theme_manager.colors(ctx))
        } else {
            PdfTheme::print_default()
        };
        let base_dir = source_path.as_ref().and_then(|p| p.parent());

        let pdf_bytes = match render_markdown_to_pdf(&content, &options, &theme, base_dir) {
            Ok(bytes) => bytes,
            Err(e) => {
                warn!("Print preview failed to render PDF: {}", e);
                let time = self.get_app_time();
                self.state.show_toast(
                    t!("notification.export_failed", error = e.to_string()).to_string(),
                    time,
                    3.0,
                );
                return;
            }
        };

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let tmp_path = std::env::temp_dir().join(format!("ferrite-print-preview-{nanos}.pdf"));

        if let Err(e) = std::fs::write(&tmp_path, &pdf_bytes) {
            warn!(
                "Print preview failed to write temp PDF to {}: {}",
                tmp_path.display(),
                e
            );
            let time = self.get_app_time();
            self.state.show_toast(
                t!("notification.export_failed", error = e.to_string()).to_string(),
                time,
                3.0,
            );
            return;
        }

        let display_title = source_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|name| t!("print_preview.tab_title", file = name.to_string()).to_string())
            .unwrap_or_else(|| t!("print_preview.untitled").to_string());

        match self.state.open_pdf_tab(tmp_path.clone(), true) {
            Ok(idx) => {
                if let Some(t) = self.state.tab_mut(idx) {
                    if let TabKind::PdfViewer(ref mut vs) = t.kind {
                        vs.display_title = Some(display_title);
                        vs.ephemeral_temp_file = true;
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Print preview could not open temp PDF {}: {}",
                    tmp_path.display(),
                    e
                );
                let _ = std::fs::remove_file(&tmp_path);
                let time = self.get_app_time();
                self.state.show_toast(
                    t!("notification.export_failed", error = e.to_string()).to_string(),
                    time,
                    3.0,
                );
            }
        }
    }

    /// Handle copying the current document as HTML to clipboard.
    pub(crate) fn handle_copy_as_html(&mut self) {
        // Get the active tab content
        let Some(tab) = self.state.active_tab() else {
            let time = self.get_app_time();
            self.state
                .show_toast(t!("notification.no_document_copy").to_string(), time, 2.0);
            return;
        };

        let content = tab.content.clone();

        // Copy HTML to clipboard
        match copy_html_to_clipboard(&content) {
            Ok(()) => {
                info!("Copied HTML to clipboard");
                let time = self.get_app_time();
                self.state
                    .show_toast(t!("notification.html_copied").to_string(), time, 2.0);
            }
            Err(e) => {
                warn!("Failed to copy HTML to clipboard: {}", e);
                let time = self.get_app_time();
                self.state.show_toast(
                    t!("notification.copy_failed", error = e.to_string()).to_string(),
                    time,
                    3.0,
                );
            }
        }
    }
}
