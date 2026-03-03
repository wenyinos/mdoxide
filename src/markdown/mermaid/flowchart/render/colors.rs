//! Color themes for flowchart rendering.

use egui::Color32;

/// Colors for rendering the flowchart.
#[derive(Debug, Clone)]
pub struct FlowchartColors {
    pub node_fill: Color32,
    pub node_stroke: Color32,
    pub node_text: Color32,
    pub edge_stroke: Color32,
    pub edge_label_bg: Color32,
    pub edge_label_text: Color32,
    pub diamond_fill: Color32,
    pub circle_fill: Color32,
    pub subgraph_fill: Color32,
    pub subgraph_fill_alt: Color32,
    pub subgraph_stroke: Color32,
    pub subgraph_title: Color32,
}

impl FlowchartColors {
    pub fn dark() -> Self {
        Self {
            node_fill: Color32::from_rgb(45, 55, 72),
            node_stroke: Color32::from_rgb(100, 140, 180),
            node_text: Color32::from_rgb(220, 230, 240),
            edge_stroke: Color32::from_rgb(120, 150, 180),
            edge_label_bg: Color32::from_rgb(35, 45, 55),
            edge_label_text: Color32::from_rgb(180, 190, 200),
            diamond_fill: Color32::from_rgb(60, 50, 70),
            circle_fill: Color32::from_rgb(50, 65, 75),
            // Warm cream/gold tones similar to Mermaid's subgraph styling
            subgraph_fill: Color32::from_rgba_unmultiplied(90, 85, 60, 160),
            subgraph_fill_alt: Color32::from_rgba_unmultiplied(75, 70, 50, 140),
            subgraph_stroke: Color32::from_rgb(140, 130, 90),
            subgraph_title: Color32::from_rgb(220, 210, 170),
        }
    }

    pub fn light() -> Self {
        Self {
            node_fill: Color32::from_rgb(240, 245, 250),
            node_stroke: Color32::from_rgb(100, 140, 180),
            node_text: Color32::from_rgb(30, 40, 50),
            edge_stroke: Color32::from_rgb(100, 130, 160),
            edge_label_bg: Color32::from_rgb(255, 255, 255),
            edge_label_text: Color32::from_rgb(60, 70, 80),
            diamond_fill: Color32::from_rgb(255, 250, 240),
            circle_fill: Color32::from_rgb(240, 250, 255),
            // Mermaid-style cream/yellow subgraph background (#ffffde)
            subgraph_fill: Color32::from_rgba_unmultiplied(255, 255, 222, 200),
            subgraph_fill_alt: Color32::from_rgba_unmultiplied(255, 250, 200, 180),
            subgraph_stroke: Color32::from_rgb(180, 170, 100),
            subgraph_title: Color32::from_rgb(100, 90, 50),
        }
    }
}
