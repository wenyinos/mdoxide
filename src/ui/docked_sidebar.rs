//! Shared chrome for docked left/right sidebars (file tree, outline panel).
//!
//! These panels sit between the ribbon (top) and status bar (bottom). Horizontal
//! dividers come from those regions; sidebars only need a vertical edge toward
//! the editor and a slight vertical bleed to avoid sub-pixel gaps.

use eframe::egui::{self, Color32, Margin, Ui};

/// Vertical bleed into ribbon / status bar regions to close 1px layout gaps.
pub const DOCKED_SIDEBAR_BLEED: f32 = 1.0;

/// Which vertical edge gets the divider toward the central panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockedSidebarEdge {
    /// Left-docked panel (file tree): divider on the right.
    Left,
    /// Right-docked panel (outline): divider on the left.
    Right,
}

/// Frame for docked sidebars: fill only, no full rect stroke, slight vertical bleed.
pub fn frame(fill: Color32) -> egui::Frame {
    let bleed = DOCKED_SIDEBAR_BLEED.round() as i8;
    egui::Frame::NONE
        .fill(fill)
        .stroke(egui::Stroke::NONE)
        .outer_margin(Margin {
            top: -bleed,
            bottom: -bleed,
            left: 0,
            right: 0,
        })
}

/// Draw the single vertical divider on the inner edge (call at end of panel content).
pub fn paint_vertical_divider(ui: &Ui, border_color: Color32, edge: DockedSidebarEdge) {
    let rect = ui.max_rect();
    let stroke = egui::Stroke::new(1.0, border_color);
    let (a, b) = match edge {
        DockedSidebarEdge::Left => (rect.right_top(), rect.right_bottom()),
        DockedSidebarEdge::Right => (rect.left_top(), rect.left_bottom()),
    };
    ui.painter().line_segment([a, b], stroke);
}
