//! Progress indicator while the workspace file index is building.

use crate::workspaces::FileIndexProgress;
use eframe::egui::{self, RichText};
use rust_i18n::t;

/// Render an indexing progress row (animated bar while scanning).
pub fn file_index_progress_ui(
    ui: &mut egui::Ui,
    progress: FileIndexProgress,
    secondary_color: egui::Color32,
) {
    ui.horizontal(|ui| {
        ui.add(
            egui::ProgressBar::new(0.0)
                .animate(true)
                .desired_width(ui.available_width() - 120.0),
        );
        ui.label(
            RichText::new(t!("workspace.indexing", count = progress.files_found))
                .color(secondary_color)
                .small(),
        );
    });
}
