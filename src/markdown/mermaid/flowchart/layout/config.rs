//! Layout configuration for flowchart diagrams.

use egui::Vec2;

/// Configuration for flowchart layout.
#[derive(Debug, Clone)]
pub(crate) struct FlowLayoutConfig {
    pub node_padding: Vec2,
    pub node_spacing: Vec2,
    pub max_node_width: f32,
    pub text_width_factor: f32,
    pub margin: f32,
    pub crossing_reduction_iterations: usize,
    /// Padding around subgraph content
    pub subgraph_padding: f32,
    /// Height reserved for subgraph title
    pub subgraph_title_height: f32,
    /// Extra margin between nested subgraph boundaries
    pub nested_subgraph_margin: f32,
}
