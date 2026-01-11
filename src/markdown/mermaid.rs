//! Native Mermaid Diagram Rendering
//!
//! This module provides native rendering of MermaidJS diagrams without external
//! dependencies. Diagrams are parsed and rendered directly using egui primitives.
//!
//! # Supported Diagram Types
//!
//! - **Flowchart** (TD, TB, LR, RL, BT) - Nodes and edges with various shapes
//! - **Sequence Diagram** - Participants and message flows (planned)
//!
//! # Architecture
//!
//! 1. `parser` - Parse mermaid source into AST
//! 2. `layout` - Compute node positions using layout algorithms
//! 3. `renderer` - Draw the diagram using egui painter
//!
//! # Example
//!
//! ```ignore
//! use crate::markdown::mermaid::{parse_flowchart, layout_flowchart, render_flowchart, EguiTextMeasurer};
//!
//! let source = "flowchart TD\n  A[Start] --> B[End]";
//! if let Ok(flowchart) = parse_flowchart(source) {
//!     let text_measurer = EguiTextMeasurer::new(ui);
//!     let layout = layout_flowchart(&flowchart, available_width, font_size, &text_measurer);
//!     render_flowchart(ui, &flowchart, &layout, colors, font_size);
//! }
//! ```

use egui::{Color32, FontId, Pos2, Rect, Rounding, Stroke, Ui, Vec2};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// Text Measurement
// ─────────────────────────────────────────────────────────────────────────────

/// Result of measuring text dimensions.
#[derive(Debug, Clone, Copy)]
pub struct TextSize {
    pub width: f32,
    pub height: f32,
}

impl TextSize {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// Trait for measuring text dimensions.
///
/// This enables backend-agnostic text measurement, supporting future
/// SVG/PNG backends when extracting to a standalone crate.
pub trait TextMeasurer {
    /// Measure the dimensions of text with the given font size.
    fn measure(&self, text: &str, font_size: f32) -> TextSize;

    /// Get the row height for a font at the given size.
    fn row_height(&self, font_size: f32) -> f32;

    /// Measure text with wrapping at max_width. Returns size of wrapped text.
    fn measure_wrapped(&self, text: &str, font_size: f32, max_width: f32) -> TextSize {
        let single_line = self.measure(text, font_size);
        if single_line.width <= max_width || max_width <= 0.0 {
            return single_line;
        }

        // Estimate wrapped height based on number of lines needed
        let lines_needed = (single_line.width / max_width).ceil();
        TextSize::new(max_width, single_line.height * lines_needed)
    }

    /// Truncate text to fit within max_width, adding ellipsis if needed.
    fn truncate_with_ellipsis(&self, text: &str, font_size: f32, max_width: f32) -> String {
        let size = self.measure(text, font_size);
        if size.width <= max_width || max_width <= 0.0 {
            return text.to_string();
        }

        let ellipsis = "…";
        let ellipsis_width = self.measure(ellipsis, font_size).width;
        let available_width = max_width - ellipsis_width;

        if available_width <= 0.0 {
            return ellipsis.to_string();
        }

        // Binary search for the right truncation point
        let chars: Vec<char> = text.chars().collect();
        let mut low = 0;
        let mut high = chars.len();

        while low < high {
            let mid = (low + high + 1) / 2;
            let truncated: String = chars[..mid].iter().collect();
            let width = self.measure(&truncated, font_size).width;

            if width <= available_width {
                low = mid;
            } else {
                high = mid - 1;
            }
        }

        if low == 0 {
            ellipsis.to_string()
        } else {
            let truncated: String = chars[..low].iter().collect();
            format!("{}{}", truncated, ellipsis)
        }
    }
}

/// Text measurer implementation using egui's font system.
pub struct EguiTextMeasurer<'a> {
    ui: &'a Ui,
}

impl<'a> EguiTextMeasurer<'a> {
    pub fn new(ui: &'a Ui) -> Self {
        Self { ui }
    }
}

impl TextMeasurer for EguiTextMeasurer<'_> {
    fn measure(&self, text: &str, font_size: f32) -> TextSize {
        let font_id = FontId::proportional(font_size);
        let galley = self.ui.fonts(|fonts| {
            fonts.layout_no_wrap(text.to_string(), font_id, Color32::PLACEHOLDER)
        });
        TextSize::new(galley.rect.width(), galley.rect.height())
    }

    fn row_height(&self, font_size: f32) -> f32 {
        let font_id = FontId::proportional(font_size);
        self.ui.fonts(|fonts| fonts.row_height(&font_id))
    }

    fn measure_wrapped(&self, text: &str, font_size: f32, max_width: f32) -> TextSize {
        if max_width <= 0.0 {
            return self.measure(text, font_size);
        }

        let font_id = FontId::proportional(font_size);
        let galley = self.ui.fonts(|fonts| {
            let layout_job = egui::text::LayoutJob::simple(
                text.to_string(),
                font_id,
                Color32::PLACEHOLDER,
                max_width,
            );
            fonts.layout_job(layout_job)
        });
        TextSize::new(galley.rect.width(), galley.rect.height())
    }
}

/// Fallback text measurer using character-based estimation.
/// Used when egui context is not available (e.g., in tests).
#[derive(Debug, Clone, Copy, Default)]
pub struct EstimatedTextMeasurer {
    /// Approximate width per character as a fraction of font size.
    char_width_factor: f32,
}

impl EstimatedTextMeasurer {
    pub fn new() -> Self {
        Self {
            char_width_factor: 0.55, // Slightly better than the old 0.6
        }
    }
}

impl TextMeasurer for EstimatedTextMeasurer {
    fn measure(&self, text: &str, font_size: f32) -> TextSize {
        let width = text.len() as f32 * font_size * self.char_width_factor;
        TextSize::new(width, font_size)
    }

    fn row_height(&self, font_size: f32) -> f32 {
        font_size * 1.2 // Standard line height
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Flowchart AST Types
// ─────────────────────────────────────────────────────────────────────────────

/// Direction of the flowchart layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlowDirection {
    #[default]
    TopDown,  // TD or TB
    BottomUp, // BT
    LeftRight, // LR
    RightLeft, // RL
}

/// Shape of a flowchart node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeShape {
    #[default]
    Rectangle,    // [text]
    RoundRect,    // (text)
    Stadium,      // ([text])
    Diamond,      // {text}
    Hexagon,      // {{text}}
    Parallelogram, // [/text/]
    Circle,       // ((text))
    Cylinder,     // [(text)]
    Subroutine,   // [[text]]
    Asymmetric,   // >text]
}

/// A node in the flowchart.
#[derive(Debug, Clone)]
pub struct FlowNode {
    pub id: String,
    pub label: String,
    pub shape: NodeShape,
}

/// Style of an edge line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EdgeStyle {
    #[default]
    Solid,   // ---
    Dotted,  // -.-
    Thick,   // ===
}

/// Type of arrow head.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrowHead {
    #[default]
    Arrow,  // >
    Circle, // o
    Cross,  // x
    None,
}

/// An edge connecting two nodes.
#[derive(Debug, Clone)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub style: EdgeStyle,
    pub arrow_start: ArrowHead,
    pub arrow_end: ArrowHead,
}

/// A subgraph (cluster) in a flowchart.
#[derive(Debug, Clone)]
pub struct FlowSubgraph {
    /// Unique identifier for the subgraph
    pub id: String,
    /// Display title (may differ from id)
    pub title: Option<String>,
    /// IDs of nodes directly contained in this subgraph
    pub node_ids: Vec<String>,
    /// IDs of nested subgraphs
    pub child_subgraph_ids: Vec<String>,
    /// Optional direction override for this subgraph (for future use)
    #[allow(dead_code)]
    pub direction: Option<FlowDirection>,
}

/// A parsed flowchart.
#[derive(Debug, Clone, Default)]
pub struct Flowchart {
    pub direction: FlowDirection,
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
    /// Subgraphs in the flowchart (order matters for rendering - parents before children)
    pub subgraphs: Vec<FlowSubgraph>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Flowchart Parser
// ─────────────────────────────────────────────────────────────────────────────

/// Parse mermaid flowchart source into a Flowchart AST.
pub fn parse_flowchart(source: &str) -> Result<Flowchart, String> {
    let mut flowchart = Flowchart::default();
    let lines: Vec<&str> = source.lines().collect();
    let mut node_map: HashMap<String, usize> = HashMap::new();
    let mut line_idx = 0;

    // Parse header line (skip comments and empty lines)
    let mut found_header = false;
    while line_idx < lines.len() {
        let header_trimmed = lines[line_idx].trim();
        line_idx += 1;
        
        // Skip empty lines and comments
        if header_trimmed.is_empty() || header_trimmed.starts_with("%%") {
            continue;
        }
        let header_lower = header_trimmed.to_lowercase();
        if header_lower.starts_with("flowchart") || header_lower.starts_with("graph") {
            flowchart.direction = parse_direction(&header_lower);
            found_header = true;
            break;
        } else {
            return Err("Expected 'flowchart' or 'graph' declaration".to_string());
        }
    }
    
    if !found_header {
        return Err("Empty flowchart source".to_string());
    }

    // Parse body with subgraph support
    let mut subgraph_stack: Vec<SubgraphBuilder> = Vec::new();
    let mut subgraph_counter = 0;
    let mut subgraph_map: HashMap<String, usize> = HashMap::new();

    while line_idx < lines.len() {
        let line = lines[line_idx].trim();
        line_idx += 1;
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

        let line_lower = line.to_lowercase();

        // Skip styling directives (not yet implemented, but shouldn't create nodes)
        if line_lower.starts_with("classdef ")
            || line_lower.starts_with("class ")
            || line_lower.starts_with("style ")
            || line_lower.starts_with("linkstyle ")
            || line_lower.starts_with("click ")
        {
            continue;
        }

        // Check for subgraph start
        if line_lower.starts_with("subgraph") {
            let (id, title) = parse_subgraph_header(line, &mut subgraph_counter);
            subgraph_stack.push(SubgraphBuilder {
                id: id.clone(),
                title,
                node_ids: Vec::new(),
                child_subgraph_ids: Vec::new(),
                direction: None,
            });
            continue;
        }

        // Check for subgraph end
        if line_lower == "end" {
            if let Some(builder) = subgraph_stack.pop() {
                let subgraph = FlowSubgraph {
                    id: builder.id.clone(),
                    title: builder.title,
                    node_ids: builder.node_ids,
                    child_subgraph_ids: builder.child_subgraph_ids,
                    direction: builder.direction,
                };
                
                // Register this subgraph as a child of the parent (if any)
                if let Some(parent) = subgraph_stack.last_mut() {
                    parent.child_subgraph_ids.push(builder.id.clone());
                }
                
                subgraph_map.insert(builder.id, flowchart.subgraphs.len());
                flowchart.subgraphs.push(subgraph);
            }
            continue;
        }

        // Check for direction override inside subgraph
        if !subgraph_stack.is_empty() && line_lower.starts_with("direction") {
            if let Some(current) = subgraph_stack.last_mut() {
                current.direction = Some(parse_direction(&line_lower));
            }
            continue;
        }

        // Try to parse as edge (contains arrow)
        if let Some((nodes, edge)) = parse_edge_line(line) {
            for (id, label, shape) in nodes {
                if let Some(&idx) = node_map.get(&id) {
                    // Node exists - update if new definition has more info
                    let existing = &mut flowchart.nodes[idx];
                    // Update label if new one is not just the ID (has actual content)
                    if label != id && existing.label == existing.id {
                        existing.label = label;
                        existing.shape = shape;
                    }
                } else {
                    node_map.insert(id.clone(), flowchart.nodes.len());
                    flowchart.nodes.push(FlowNode { id: id.clone(), label, shape });
                    
                    // Associate with current subgraph if any
                    if let Some(current) = subgraph_stack.last_mut() {
                        current.node_ids.push(id);
                    }
                }
            }
            if let Some(e) = edge {
                flowchart.edges.push(e);
            }
        } else if let Some(node) = parse_node_definition(line) {
            // Standalone node definition
            if let Some(&idx) = node_map.get(&node.id) {
                // Node exists - update if new definition has more info
                let existing = &mut flowchart.nodes[idx];
                if node.label != node.id && existing.label == existing.id {
                    existing.label = node.label;
                    existing.shape = node.shape;
                }
            } else {
                let id = node.id.clone();
                node_map.insert(id.clone(), flowchart.nodes.len());
                flowchart.nodes.push(node);
                
                // Associate with current subgraph if any
                if let Some(current) = subgraph_stack.last_mut() {
                    current.node_ids.push(id);
                }
            }
        }
    }

    // Handle any unclosed subgraphs (close them at end of diagram)
    while let Some(builder) = subgraph_stack.pop() {
        let subgraph = FlowSubgraph {
            id: builder.id.clone(),
            title: builder.title,
            node_ids: builder.node_ids,
            child_subgraph_ids: builder.child_subgraph_ids,
            direction: builder.direction,
        };
        
        if let Some(parent) = subgraph_stack.last_mut() {
            parent.child_subgraph_ids.push(builder.id.clone());
        }
        
        flowchart.subgraphs.push(subgraph);
    }

    Ok(flowchart)
}

/// Helper struct for building subgraphs during parsing.
struct SubgraphBuilder {
    id: String,
    title: Option<String>,
    node_ids: Vec<String>,
    child_subgraph_ids: Vec<String>,
    direction: Option<FlowDirection>,
}

/// Parse subgraph header line to extract id and title.
/// Supports: `subgraph title` and `subgraph id [title]`
fn parse_subgraph_header(line: &str, counter: &mut usize) -> (String, Option<String>) {
    let rest = line.trim_start_matches(|c: char| c.is_ascii_alphabetic())
        .trim_start(); // Remove "subgraph" and leading whitespace
    
    if rest.is_empty() {
        // No id or title, generate id
        *counter += 1;
        return (format!("subgraph_{}", counter), None);
    }

    // Check if rest contains brackets (explicit title)
    if let Some(bracket_start) = rest.find('[') {
        if let Some(bracket_end) = rest.rfind(']') {
            let id = rest[..bracket_start].trim().to_string();
            let title = rest[bracket_start + 1..bracket_end].trim().to_string();
            let id = if id.is_empty() {
                *counter += 1;
                format!("subgraph_{}", counter)
            } else {
                id
            };
            return (id, Some(title));
        }
    }

    // Check for quoted title
    if rest.starts_with('"') || rest.starts_with('\'') {
        let quote = rest.chars().next().unwrap();
        if let Some(end_quote) = rest[1..].find(quote) {
            let title = rest[1..end_quote + 1].to_string();
            *counter += 1;
            return (format!("subgraph_{}", counter), Some(title));
        }
    }

    // Check if first token looks like an ID (alphanumeric, no spaces)
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    if tokens.len() == 1 {
        // Single token - could be ID or title
        // If it contains spaces when trimmed differently, it's a title
        // Otherwise treat as both ID and title
        let token = tokens[0].to_string();
        return (token.clone(), Some(token));
    } else if tokens.len() >= 2 {
        // First token is ID, rest is title
        let id = tokens[0].to_string();
        let title = tokens[1..].join(" ");
        return (id, Some(title));
    }

    // Fallback: generate ID, use rest as title
    *counter += 1;
    (format!("subgraph_{}", counter), Some(rest.to_string()))
}

fn parse_direction(header: &str) -> FlowDirection {
    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts.len() > 1 {
        match parts[1].to_uppercase().as_str() {
            "TD" | "TB" => FlowDirection::TopDown,
            "BT" => FlowDirection::BottomUp,
            "LR" => FlowDirection::LeftRight,
            "RL" => FlowDirection::RightLeft,
            _ => FlowDirection::TopDown,
        }
    } else {
        FlowDirection::TopDown
    }
}

fn parse_edge_line(line: &str) -> Option<(Vec<(String, String, NodeShape)>, Option<FlowEdge>)> {
    // Find arrow patterns
    let arrow_patterns = [
        ("-->", EdgeStyle::Solid, ArrowHead::None, ArrowHead::Arrow),
        ("--->", EdgeStyle::Solid, ArrowHead::None, ArrowHead::Arrow),
        ("---", EdgeStyle::Solid, ArrowHead::None, ArrowHead::None),
        ("-.->", EdgeStyle::Dotted, ArrowHead::None, ArrowHead::Arrow),
        ("-.-", EdgeStyle::Dotted, ArrowHead::None, ArrowHead::None),
        ("==>", EdgeStyle::Thick, ArrowHead::None, ArrowHead::Arrow),
        ("===", EdgeStyle::Thick, ArrowHead::None, ArrowHead::None),
        ("--o", EdgeStyle::Solid, ArrowHead::None, ArrowHead::Circle),
        ("--x", EdgeStyle::Solid, ArrowHead::None, ArrowHead::Cross),
        ("<-->", EdgeStyle::Solid, ArrowHead::Arrow, ArrowHead::Arrow),
        ("o--o", EdgeStyle::Solid, ArrowHead::Circle, ArrowHead::Circle),
        ("x--x", EdgeStyle::Solid, ArrowHead::Cross, ArrowHead::Cross),
    ];

    for (pattern, style, arrow_start, arrow_end) in arrow_patterns {
        // Check for labeled edges: A -->|label| B
        if let Some(pos) = line.find(pattern) {
            let left = line[..pos].trim();
            let right_part = &line[pos + pattern.len()..];
            
            // Check for label
            let (label, right) = if let Some(label_start) = right_part.find('|') {
                if let Some(label_end) = right_part[label_start + 1..].find('|') {
                    let label = right_part[label_start + 1..label_start + 1 + label_end].trim();
                    let label = clean_label(label);
                    let rest = right_part[label_start + 2 + label_end..].trim();
                    (Some(label), rest)
                } else {
                    (None, right_part.trim())
                }
            } else {
                (None, right_part.trim())
            };

            // Parse left and right nodes
            let left_node = parse_node_from_text(left);
            let right_node = parse_node_from_text(right);

            if let (Some((from_id, from_label, from_shape)), Some((to_id, to_label, to_shape))) = 
                (left_node, right_node) 
            {
                let nodes = vec![
                    (from_id.clone(), from_label, from_shape),
                    (to_id.clone(), to_label, to_shape),
                ];
                let edge = FlowEdge {
                    from: from_id,
                    to: to_id,
                    label,
                    style,
                    arrow_start,
                    arrow_end,
                };
                return Some((nodes, Some(edge)));
            }
        }
    }

    None
}

fn parse_node_from_text(text: &str) -> Option<(String, String, NodeShape)> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }

    // Try various shape patterns
    // Stadium: ([text])
    if text.contains("([") && text.contains("])") {
        if let Some(start) = text.find("([") {
            let id = text[..start].trim();
            let id = if id.is_empty() { &text[..start.max(1)] } else { id };
            if let Some(end) = text.find("])") {
                let label = text[start + 2..end].trim();
                return Some((extract_id(id, text), clean_label(label), NodeShape::Stadium));
            }
        }
    }

    // Circle: ((text))
    if text.contains("((") && text.contains("))") {
        if let Some(start) = text.find("((") {
            let id = text[..start].trim();
            if let Some(end) = text.find("))") {
                let label = text[start + 2..end].trim();
                return Some((extract_id(id, text), clean_label(label), NodeShape::Circle));
            }
        }
    }

    // Cylinder: [(text)]
    if text.contains("[(") && text.contains(")]") {
        if let Some(start) = text.find("[(") {
            let id = text[..start].trim();
            if let Some(end) = text.find(")]") {
                let label = text[start + 2..end].trim();
                return Some((extract_id(id, text), clean_label(label), NodeShape::Cylinder));
            }
        }
    }

    // Subroutine: [[text]]
    if text.contains("[[") && text.contains("]]") {
        if let Some(start) = text.find("[[") {
            let id = text[..start].trim();
            if let Some(end) = text.find("]]") {
                let label = text[start + 2..end].trim();
                return Some((extract_id(id, text), clean_label(label), NodeShape::Subroutine));
            }
        }
    }

    // Hexagon: {{text}}
    if text.contains("{{") && text.contains("}}") {
        if let Some(start) = text.find("{{") {
            let id = text[..start].trim();
            if let Some(end) = text.find("}}") {
                let label = text[start + 2..end].trim();
                return Some((extract_id(id, text), clean_label(label), NodeShape::Hexagon));
            }
        }
    }

    // Diamond: {text}
    if text.contains('{') && text.contains('}') && !text.contains("{{") {
        if let Some(start) = text.find('{') {
            let id = text[..start].trim();
            if let Some(end) = text.rfind('}') {
                let label = text[start + 1..end].trim();
                return Some((extract_id(id, text), clean_label(label), NodeShape::Diamond));
            }
        }
    }

    // Round rect: (text)
    if text.contains('(') && text.contains(')') && !text.contains("((") && !text.contains("([") && !text.contains("[(") {
        if let Some(start) = text.find('(') {
            let id = text[..start].trim();
            if let Some(end) = text.rfind(')') {
                let label = text[start + 1..end].trim();
                return Some((extract_id(id, text), clean_label(label), NodeShape::RoundRect));
            }
        }
    }

    // Rectangle: [text]
    if text.contains('[') && text.contains(']') && !text.contains("[[") && !text.contains("[(") && !text.contains("([") {
        if let Some(start) = text.find('[') {
            let id = text[..start].trim();
            if let Some(end) = text.rfind(']') {
                let label = text[start + 1..end].trim();
                return Some((extract_id(id, text), clean_label(label), NodeShape::Rectangle));
            }
        }
    }

    // Asymmetric: >text]
    if text.contains('>') && text.contains(']') {
        if let Some(start) = text.find('>') {
            let id = text[..start].trim();
            if let Some(end) = text.rfind(']') {
                let label = text[start + 1..end].trim();
                return Some((extract_id(id, text), clean_label(label), NodeShape::Asymmetric));
            }
        }
    }

    // Just an ID (no shape specified)
    let id = text.split_whitespace().next().unwrap_or(text);
    Some((id.to_string(), id.to_string(), NodeShape::Rectangle))
}

fn extract_id(id: &str, full_text: &str) -> String {
    if id.is_empty() {
        // Generate ID from first part of text
        full_text.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect()
    } else {
        id.to_string()
    }
}

/// Clean up label text by converting HTML line breaks to newlines.
fn clean_label(label: &str) -> String {
    label
        .replace("<br/>", "\n")
        .replace("<br>", "\n")
        .replace("<br />", "\n")
}

fn parse_node_definition(line: &str) -> Option<FlowNode> {
    parse_node_from_text(line).map(|(id, label, shape)| FlowNode { id, label, shape })
}

// ─────────────────────────────────────────────────────────────────────────────
// Layout Engine
// ─────────────────────────────────────────────────────────────────────────────

/// Layout information for a node.
#[derive(Debug, Clone)]
pub struct NodeLayout {
    pub pos: Pos2,
    pub size: Vec2,
}

/// Layout information for a subgraph.
#[derive(Debug, Clone)]
pub struct SubgraphLayout {
    /// Bounding box position (top-left corner)
    pub pos: Pos2,
    /// Bounding box size
    pub size: Vec2,
    /// Title to display (if any)
    pub title: Option<String>,
}

/// Complete layout for a flowchart.
#[derive(Debug, Clone, Default)]
pub struct FlowchartLayout {
    pub nodes: HashMap<String, NodeLayout>,
    pub subgraphs: HashMap<String, SubgraphLayout>,
    pub total_size: Vec2,
    /// Set of back-edges (cycles): (from_node_id, to_node_id)
    pub back_edges: std::collections::HashSet<(String, String)>,
}

/// Compute layout for a flowchart using a Sugiyama-style layered graph algorithm.
///
/// This algorithm supports:
/// - Proper branching with side-by-side node placement
/// - Cycle detection and back-edge handling
/// - Edge crossing minimization using barycenter heuristic
/// - Subgraph bounding boxes with padding
/// - All flow directions (TD, BT, LR, RL)
///
/// The `text_measurer` parameter enables accurate text sizing. Use `EguiTextMeasurer`
/// when a UI context is available, or `EstimatedTextMeasurer` for testing.
pub fn layout_flowchart(
    flowchart: &Flowchart,
    available_width: f32,
    font_size: f32,
    text_measurer: &impl TextMeasurer,
) -> FlowchartLayout {
    if flowchart.nodes.is_empty() {
        return FlowchartLayout::default();
    }

    // Layout configuration
    let config = FlowLayoutConfig {
        node_padding: Vec2::new(24.0, 12.0),
        node_spacing: Vec2::new(50.0, 60.0),
        max_node_width: (available_width * 0.4).max(150.0),
        text_width_factor: 1.15,
        margin: 20.0,
        crossing_reduction_iterations: 4,
        subgraph_padding: 15.0,
        subgraph_title_height: 24.0,
    };

    // Build internal graph representation
    let graph = FlowGraph::from_flowchart(flowchart, font_size, text_measurer, &config);

    // Run the Sugiyama layout algorithm
    let sugiyama = SugiyamaLayout::new(graph, flowchart.direction, config.clone(), available_width);
    let mut layout = sugiyama.compute();

    // Compute subgraph bounding boxes
    compute_subgraph_layouts(&mut layout, flowchart, &config);

    layout
}

/// Compute bounding boxes for all subgraphs based on positioned nodes.
fn compute_subgraph_layouts(
    layout: &mut FlowchartLayout,
    flowchart: &Flowchart,
    config: &FlowLayoutConfig,
) {
    // Process subgraphs in reverse order (children before parents for nested bounds)
    // But store results keyed by ID so we can look up child bounds
    let mut subgraph_bounds: HashMap<String, (Pos2, Pos2)> = HashMap::new();

    for subgraph in flowchart.subgraphs.iter().rev() {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut has_content = false;

        // Include direct node members
        for node_id in &subgraph.node_ids {
            if let Some(node_layout) = layout.nodes.get(node_id) {
                min_x = min_x.min(node_layout.pos.x);
                min_y = min_y.min(node_layout.pos.y);
                max_x = max_x.max(node_layout.pos.x + node_layout.size.x);
                max_y = max_y.max(node_layout.pos.y + node_layout.size.y);
                has_content = true;
            }
        }

        // Include nested subgraph bounds
        for child_id in &subgraph.child_subgraph_ids {
            if let Some(&(child_min, child_max)) = subgraph_bounds.get(child_id) {
                min_x = min_x.min(child_min.x);
                min_y = min_y.min(child_min.y);
                max_x = max_x.max(child_max.x);
                max_y = max_y.max(child_max.y);
                has_content = true;
            }
        }

        if has_content {
            // Add padding around content
            let padded_min = Pos2::new(
                min_x - config.subgraph_padding,
                min_y - config.subgraph_padding - config.subgraph_title_height,
            );
            let padded_max = Pos2::new(
                max_x + config.subgraph_padding,
                max_y + config.subgraph_padding,
            );

            subgraph_bounds.insert(subgraph.id.clone(), (padded_min, padded_max));

            let size = Vec2::new(padded_max.x - padded_min.x, padded_max.y - padded_min.y);
            layout.subgraphs.insert(
                subgraph.id.clone(),
                SubgraphLayout {
                    pos: padded_min,
                    size,
                    title: subgraph.title.clone(),
                },
            );
        }
    }

    // Update total_size to account for subgraph bounds
    for sg_layout in layout.subgraphs.values() {
        layout.total_size.x = layout.total_size.x.max(sg_layout.pos.x + sg_layout.size.x + config.margin);
        layout.total_size.y = layout.total_size.y.max(sg_layout.pos.y + sg_layout.size.y + config.margin);
    }
}

/// Configuration for flowchart layout.
#[derive(Debug, Clone)]
struct FlowLayoutConfig {
    node_padding: Vec2,
    node_spacing: Vec2,
    max_node_width: f32,
    text_width_factor: f32,
    margin: f32,
    crossing_reduction_iterations: usize,
    /// Padding around subgraph content
    subgraph_padding: f32,
    /// Height reserved for subgraph title
    subgraph_title_height: f32,
}

/// Internal graph representation for layout algorithms.
#[derive(Debug)]
struct FlowGraph {
    /// Node IDs in order
    node_ids: Vec<String>,
    /// Map from node ID to index (kept for potential future edge routing enhancements)
    #[allow(dead_code)]
    id_to_index: HashMap<String, usize>,
    /// Node sizes (indexed by node index)
    node_sizes: Vec<Vec2>,
    /// Outgoing edges: node_index -> Vec<target_index>
    outgoing: Vec<Vec<usize>>,
    /// Incoming edges: node_index -> Vec<source_index>
    incoming: Vec<Vec<usize>>,
    /// Back-edges detected during cycle breaking (source, target)
    back_edges: Vec<(usize, usize)>,
}

impl FlowGraph {
    /// Build graph from flowchart AST with text measurement.
    fn from_flowchart(
        flowchart: &Flowchart,
        font_size: f32,
        text_measurer: &impl TextMeasurer,
        config: &FlowLayoutConfig,
    ) -> Self {
        let n = flowchart.nodes.len();
        let mut node_ids = Vec::with_capacity(n);
        let mut id_to_index = HashMap::with_capacity(n);
        let mut node_sizes = Vec::with_capacity(n);
        let mut outgoing = vec![Vec::new(); n];
        let mut incoming = vec![Vec::new(); n];

        // Build node index mapping and compute sizes
        for (idx, node) in flowchart.nodes.iter().enumerate() {
            node_ids.push(node.id.clone());
            id_to_index.insert(node.id.clone(), idx);

            // Measure text and compute node size
            let text_size = text_measurer.measure(&node.label, font_size);
            let adjusted_width = text_size.width * config.text_width_factor;

            let (text_width, text_height) = if adjusted_width + config.node_padding.x * 2.0 > config.max_node_width {
                let wrap_width = config.max_node_width - config.node_padding.x * 2.0;
                let wrapped = text_measurer.measure_wrapped(&node.label, font_size, wrap_width);
                (wrapped.width * config.text_width_factor, wrapped.height)
            } else {
                (adjusted_width, text_size.height)
            };

            let size = Vec2::new(
                (text_width + config.node_padding.x * 2.0).max(80.0),
                (text_height + config.node_padding.y * 2.0).max(40.0),
            );
            node_sizes.push(size);
        }

        // Build adjacency lists
        for edge in &flowchart.edges {
            if let (Some(&from_idx), Some(&to_idx)) = (id_to_index.get(&edge.from), id_to_index.get(&edge.to)) {
                outgoing[from_idx].push(to_idx);
                incoming[to_idx].push(from_idx);
            }
        }

        FlowGraph {
            node_ids,
            id_to_index,
            node_sizes,
            outgoing,
            incoming,
            back_edges: Vec::new(),
        }
    }

    fn node_count(&self) -> usize {
        self.node_ids.len()
    }
}

/// Sugiyama-style layered graph layout algorithm.
struct SugiyamaLayout {
    graph: FlowGraph,
    direction: FlowDirection,
    config: FlowLayoutConfig,
    available_width: f32,
    /// Assigned layer for each node (indexed by node index)
    node_layers: Vec<usize>,
    /// Nodes in each layer, ordered for crossing minimization
    layers: Vec<Vec<usize>>,
}

impl SugiyamaLayout {
    fn new(
        graph: FlowGraph,
        direction: FlowDirection,
        config: FlowLayoutConfig,
        available_width: f32,
    ) -> Self {
        let n = graph.node_count();
        SugiyamaLayout {
            graph,
            direction,
            config,
            available_width,
            node_layers: vec![0; n],
            layers: Vec::new(),
        }
    }

    /// Run the complete layout algorithm.
    fn compute(mut self) -> FlowchartLayout {
        if self.graph.node_count() == 0 {
            return FlowchartLayout::default();
        }

        // Step 1: Detect cycles and mark back-edges
        self.detect_cycles_and_mark_back_edges();

        // Step 2: Assign layers using longest-path algorithm
        self.assign_layers();

        // Step 3: Build initial layer structure
        self.build_layers();

        // Step 4: Reduce edge crossings
        self.reduce_crossings();

        // Step 5: Assign coordinates
        self.assign_coordinates()
    }

    /// Detect cycles using DFS and mark back-edges.
    /// Uses a simple DFS-based approach to find back-edges.
    fn detect_cycles_and_mark_back_edges(&mut self) {
        let n = self.graph.node_count();
        let mut visited = vec![false; n];
        let mut in_stack = vec![false; n];
        let mut back_edges = Vec::new();

        for start in 0..n {
            if !visited[start] {
                self.dfs_find_back_edges(start, &mut visited, &mut in_stack, &mut back_edges);
            }
        }

        self.graph.back_edges = back_edges;
    }

    fn dfs_find_back_edges(
        &self,
        node: usize,
        visited: &mut [bool],
        in_stack: &mut [bool],
        back_edges: &mut Vec<(usize, usize)>,
    ) {
        visited[node] = true;
        in_stack[node] = true;

        for &neighbor in &self.graph.outgoing[node] {
            if !visited[neighbor] {
                self.dfs_find_back_edges(neighbor, visited, in_stack, back_edges);
            } else if in_stack[neighbor] {
                // Found a back-edge (cycle)
                back_edges.push((node, neighbor));
            }
        }

        in_stack[node] = false;
    }

    /// Assign layers using longest-path algorithm.
    /// Nodes with no incoming edges (ignoring back-edges) go to layer 0,
    /// others are placed at max(predecessor_layer) + 1.
    fn assign_layers(&mut self) {
        let n = self.graph.node_count();
        
        // Build effective incoming edges (excluding back-edges)
        let back_edge_set: std::collections::HashSet<(usize, usize)> = 
            self.graph.back_edges.iter().cloned().collect();
        
        let mut effective_incoming: Vec<Vec<usize>> = vec![Vec::new(); n];
        for (from_idx, targets) in self.graph.outgoing.iter().enumerate() {
            for &to_idx in targets {
                if !back_edge_set.contains(&(from_idx, to_idx)) {
                    effective_incoming[to_idx].push(from_idx);
                }
            }
        }

        // Compute layers using longest-path (BFS-based topological approach)
        let mut in_degree: Vec<usize> = effective_incoming.iter().map(|v| v.len()).collect();
        let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();

        // Start with nodes that have no incoming edges
        for (idx, &deg) in in_degree.iter().enumerate() {
            if deg == 0 {
                queue.push_back(idx);
                self.node_layers[idx] = 0;
            }
        }

        // If no root nodes found (all nodes in cycles), pick the first node
        if queue.is_empty() && n > 0 {
            queue.push_back(0);
            self.node_layers[0] = 0;
            in_degree[0] = 0;
        }

        while let Some(node) = queue.pop_front() {
            let current_layer = self.node_layers[node];
            
            for &neighbor in &self.graph.outgoing[node] {
                if !back_edge_set.contains(&(node, neighbor)) {
                    // Update layer to be at least one more than current
                    self.node_layers[neighbor] = self.node_layers[neighbor].max(current_layer + 1);
                    
                    in_degree[neighbor] = in_degree[neighbor].saturating_sub(1);
                    if in_degree[neighbor] == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        // Handle any remaining nodes (disconnected or in complex cycles)
        for idx in 0..n {
            if in_degree[idx] > 0 {
                // Place in a layer based on any assigned predecessor, or layer 0
                let max_pred_layer = effective_incoming[idx]
                    .iter()
                    .filter(|&&pred| in_degree[pred] == 0 || self.node_layers[pred] > 0)
                    .map(|&pred| self.node_layers[pred])
                    .max()
                    .unwrap_or(0);
                self.node_layers[idx] = max_pred_layer + 1;
            }
        }
    }

    /// Build the layers structure from node_layers assignments.
    fn build_layers(&mut self) {
        let max_layer = self.node_layers.iter().copied().max().unwrap_or(0);
        self.layers = vec![Vec::new(); max_layer + 1];

        // Initial ordering: by original node order (stable)
        for (node_idx, &layer) in self.node_layers.iter().enumerate() {
            self.layers[layer].push(node_idx);
        }

        // Pre-compute edge positions for ALL nodes to avoid borrow issues
        let all_edge_positions: HashMap<usize, usize> = (0..self.graph.node_count())
            .map(|node| (node, self.get_min_incoming_edge_position(node)))
            .collect();

        // For each layer, sort by the order edges were declared from predecessors
        // Mermaid convention: SECOND-declared edge target goes LEFT
        for layer in &mut self.layers {
            // Sort by position in outgoing edge list of predecessor
            // HIGHER position = later in edge declarations = goes LEFT
            layer.sort_by(|&a, &b| {
                let pos_a = all_edge_positions.get(&a).copied().unwrap_or(a);
                let pos_b = all_edge_positions.get(&b).copied().unwrap_or(b);
                pos_b.cmp(&pos_a)  // FLIPPED: higher position first
            });
        }
    }

    /// Get the minimum position of a node in any predecessor's outgoing edge list.
    /// This reflects edge declaration order.
    fn get_min_incoming_edge_position(&self, node: usize) -> usize {
        let mut min_pos = usize::MAX;
        for &pred in &self.graph.incoming[node] {
            if let Some(pos) = self.graph.outgoing[pred].iter().position(|&n| n == node) {
                min_pos = min_pos.min(pos);
            }
        }
        if min_pos == usize::MAX { node } else { min_pos }
    }

    /// Reduce edge crossings using the barycenter heuristic.
    /// Iterates top-down and bottom-up to minimize crossings.
    fn reduce_crossings(&mut self) {
        let back_edge_set: std::collections::HashSet<(usize, usize)> = 
            self.graph.back_edges.iter().cloned().collect();

        for _ in 0..self.config.crossing_reduction_iterations {
            // Top-down pass
            for layer_idx in 1..self.layers.len() {
                self.order_layer_by_barycenter(layer_idx, true, &back_edge_set);
            }
            // Bottom-up pass
            for layer_idx in (0..self.layers.len().saturating_sub(1)).rev() {
                self.order_layer_by_barycenter(layer_idx, false, &back_edge_set);
            }
        }
    }

    /// Order a single layer using barycenter of connected nodes in adjacent layer.
    fn order_layer_by_barycenter(
        &mut self,
        layer_idx: usize,
        use_predecessors: bool,
        back_edge_set: &std::collections::HashSet<(usize, usize)>,
    ) {
        let adjacent_layer_idx = if use_predecessors {
            layer_idx.saturating_sub(1)
        } else {
            (layer_idx + 1).min(self.layers.len().saturating_sub(1))
        };

        if adjacent_layer_idx == layer_idx {
            return;
        }

        // Build position map for adjacent layer
        let adjacent_positions: HashMap<usize, usize> = self.layers[adjacent_layer_idx]
            .iter()
            .enumerate()
            .map(|(pos, &node)| (node, pos))
            .collect();

        // Calculate barycenter for each node in current layer
        // Store: (node_index, barycenter) - we'll use node_index directly as tiebreaker
        let mut barycenters: Vec<(usize, f32)> = Vec::new();

        for &node in &self.layers[layer_idx] {
            let neighbors: Vec<usize> = if use_predecessors {
                self.graph.incoming[node]
                    .iter()
                    .filter(|&&pred| !back_edge_set.contains(&(pred, node)))
                    .copied()
                    .collect()
            } else {
                self.graph.outgoing[node]
                    .iter()
                    .filter(|&&succ| !back_edge_set.contains(&(node, succ)))
                    .copied()
                    .collect()
            };

            let barycenter = if neighbors.is_empty() {
                // Keep relative position if no connections - use node index
                node as f32
            } else {
                let sum: f32 = neighbors
                    .iter()
                    .filter_map(|n| adjacent_positions.get(n))
                    .map(|&pos| pos as f32)
                    .sum();
                let count = neighbors
                    .iter()
                    .filter(|n| adjacent_positions.contains_key(n))
                    .count();
                if count > 0 {
                    sum / count as f32
                } else {
                    node as f32
                }
            };

            barycenters.push((node, barycenter));
        }

        // Sort by barycenter, with edge position as tiebreaker
        // Mermaid convention: SECOND-declared edge target goes LEFT
        let edge_positions: HashMap<usize, usize> = barycenters
            .iter()
            .map(|&(node, _)| (node, self.get_min_incoming_edge_position(node)))
            .collect();

        barycenters.sort_by(|a, b| {
            match a.1.partial_cmp(&b.1) {
                Some(std::cmp::Ordering::Equal) | None => {
                    // HIGHER position = later declared = goes left
                    let pos_a = edge_positions.get(&a.0).copied().unwrap_or(a.0);
                    let pos_b = edge_positions.get(&b.0).copied().unwrap_or(b.0);
                    pos_b.cmp(&pos_a)  // FLIPPED
                }
                Some(ord) => ord,
            }
        });

        // Update layer order
        self.layers[layer_idx] = barycenters.into_iter().map(|(node, _)| node).collect();
    }

    /// Assign final coordinates to all nodes.
    fn assign_coordinates(self) -> FlowchartLayout {
        let is_horizontal = matches!(self.direction, FlowDirection::LeftRight | FlowDirection::RightLeft);
        let is_reversed = matches!(self.direction, FlowDirection::BottomUp | FlowDirection::RightLeft);

        let mut layout = FlowchartLayout::default();
        let margin = self.config.margin;

        // Calculate the maximum cross-axis size for centering
        let mut layer_cross_sizes: Vec<f32> = Vec::new();
        for layer in &self.layers {
            let mut size: f32 = 0.0;
            for &node_idx in layer {
                let node_size = self.graph.node_sizes[node_idx];
                size += if is_horizontal { node_size.y } else { node_size.x };
            }
            size += (layer.len().saturating_sub(1)) as f32 
                * if is_horizontal { self.config.node_spacing.y } else { self.config.node_spacing.x };
            layer_cross_sizes.push(size);
        }
        let max_cross_size = layer_cross_sizes.iter().copied().fold(0.0_f32, f32::max);

        // Calculate layer main-axis sizes (for positioning)
        let layer_main_sizes: Vec<f32> = self.layers
            .iter()
            .map(|layer| {
                layer
                    .iter()
                    .map(|&idx| {
                        let size = self.graph.node_sizes[idx];
                        if is_horizontal { size.x } else { size.y }
                    })
                    .fold(0.0_f32, f32::max)
            })
            .collect();

        // Position nodes layer by layer
        let mut current_main = margin;
        let mut max_x: f32 = 0.0;
        let mut max_y: f32 = 0.0;

        for (layer_idx, layer) in self.layers.iter().enumerate() {
            let layer_cross_size = layer_cross_sizes[layer_idx];
            
            // Center the layer in cross-axis
            let start_cross = if is_horizontal {
                margin + (max_cross_size - layer_cross_size) / 2.0
            } else {
                (self.available_width - layer_cross_size).max(margin * 2.0) / 2.0
            };

            let mut current_cross = start_cross;

            for &node_idx in layer {
                let node_id = &self.graph.node_ids[node_idx];
                let size = self.graph.node_sizes[node_idx];

                let pos = if is_horizontal {
                    Pos2::new(current_main, current_cross)
                } else {
                    Pos2::new(current_cross, current_main)
                };

                layout.nodes.insert(node_id.clone(), NodeLayout { pos, size });

                max_x = max_x.max(pos.x + size.x);
                max_y = max_y.max(pos.y + size.y);

                current_cross += if is_horizontal {
                    size.y + self.config.node_spacing.y
                } else {
                    size.x + self.config.node_spacing.x
                };
            }

            // Advance to next layer
            current_main += layer_main_sizes[layer_idx] 
                + if is_horizontal { self.config.node_spacing.x } else { self.config.node_spacing.y };
        }

        // Handle reversed directions (BT, RL)
        if is_reversed {
            let total = if is_horizontal { max_x } else { max_y };
            for node_layout in layout.nodes.values_mut() {
                if is_horizontal {
                    node_layout.pos.x = total - node_layout.pos.x - node_layout.size.x + margin;
                } else {
                    node_layout.pos.y = total - node_layout.pos.y - node_layout.size.y + margin;
                }
            }
        }

        // Convert back-edge indices to node IDs
        for &(from_idx, to_idx) in &self.graph.back_edges {
            let from_id = self.graph.node_ids[from_idx].clone();
            let to_id = self.graph.node_ids[to_idx].clone();
            layout.back_edges.insert((from_id, to_id));
        }

        layout.total_size = Vec2::new(max_x + margin, max_y + margin);
        layout
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Renderer
// ─────────────────────────────────────────────────────────────────────────────

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
            subgraph_fill: Color32::from_rgba_unmultiplied(60, 70, 90, 40),
            subgraph_stroke: Color32::from_rgb(80, 100, 130),
            subgraph_title: Color32::from_rgb(160, 175, 195),
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
            subgraph_fill: Color32::from_rgba_unmultiplied(200, 210, 230, 60),
            subgraph_stroke: Color32::from_rgb(150, 170, 200),
            subgraph_title: Color32::from_rgb(80, 95, 120),
        }
    }
}

/// Pre-computed edge label information for rendering.
struct EdgeLabelInfo {
    display_text: String,
    size: Vec2,
}

/// Render a flowchart to the UI.
pub fn render_flowchart(
    ui: &mut Ui,
    flowchart: &Flowchart,
    layout: &FlowchartLayout,
    colors: &FlowchartColors,
    font_size: f32,
) {
    if flowchart.nodes.is_empty() {
        return;
    }

    // Pre-compute edge label sizes before allocating painter
    // This avoids borrow checker issues with text measurement during drawing
    let label_font_size = font_size - 2.0;
    let edge_labels: HashMap<usize, EdgeLabelInfo> = {
        let text_measurer = EguiTextMeasurer::new(ui);
        flowchart.edges.iter().enumerate()
            .filter_map(|(idx, edge)| {
                edge.label.as_ref().map(|label| {
                    // Calculate max label width based on edge geometry
                    let (from_layout, to_layout) = match (layout.nodes.get(&edge.from), layout.nodes.get(&edge.to)) {
                        (Some(f), Some(t)) => (f, t),
                        _ => return None,
                    };
                    let from_center = from_layout.pos + from_layout.size / 2.0;
                    let to_center = to_layout.pos + to_layout.size / 2.0;
                    let edge_length = ((to_center.x - from_center.x).powi(2) +
                                       (to_center.y - from_center.y).powi(2)).sqrt();
                    let max_label_width = edge_length.max(60.0).min(200.0) * 0.8;

                    // Measure and potentially truncate
                    let text_size = text_measurer.measure(label, label_font_size);
                    let display_text = if text_size.width > max_label_width {
                        text_measurer.truncate_with_ellipsis(label, label_font_size, max_label_width)
                    } else {
                        label.clone()
                    };

                    let display_size = text_measurer.measure(&display_text, label_font_size);
                    let label_padding = Vec2::new(8.0, 4.0);
                    let size = Vec2::new(
                        display_size.width + label_padding.x,
                        display_size.height + label_padding.y,
                    );

                    Some((idx, EdgeLabelInfo { display_text, size }))
                })?
            })
            .collect()
    };

    // Allocate space for the diagram
    let (response, painter) = ui.allocate_painter(
        layout.total_size,
        egui::Sense::hover(),
    );
    let offset = response.rect.min.to_vec2();

    // Draw subgraphs first (behind everything else)
    // Draw in reverse order so parent subgraphs are behind children
    for subgraph in flowchart.subgraphs.iter().rev() {
        if let Some(sg_layout) = layout.subgraphs.get(&subgraph.id) {
            draw_subgraph(&painter, sg_layout, offset, colors, font_size);
        }
    }

    // Draw edges (behind nodes but above subgraphs)
    for (idx, edge) in flowchart.edges.iter().enumerate() {
        if let (Some(from_layout), Some(to_layout)) = (layout.nodes.get(&edge.from), layout.nodes.get(&edge.to)) {
            let label_info = edge_labels.get(&idx);
            let is_back_edge = layout.back_edges.contains(&(edge.from.clone(), edge.to.clone()));
            draw_edge(&painter, edge, from_layout, to_layout, offset, colors, label_font_size, flowchart.direction, label_info, is_back_edge, &layout.total_size);
        }
    }

    // Draw nodes (on top)
    for node in &flowchart.nodes {
        if let Some(node_layout) = layout.nodes.get(&node.id) {
            draw_node(&painter, node, node_layout, offset, colors, font_size);
        }
    }
}

fn draw_subgraph(
    painter: &egui::Painter,
    layout: &SubgraphLayout,
    offset: Vec2,
    colors: &FlowchartColors,
    font_size: f32,
) {
    let rect = Rect::from_min_size(layout.pos + offset, layout.size);
    
    // Draw semi-transparent background with rounded corners
    painter.rect(
        rect,
        Rounding::same(8.0),
        colors.subgraph_fill,
        Stroke::new(1.5, colors.subgraph_stroke),
    );

    // Draw title if present
    if let Some(title) = &layout.title {
        let title_pos = Pos2::new(
            rect.left() + 12.0,
            rect.top() + 6.0,
        );
        painter.text(
            title_pos,
            egui::Align2::LEFT_TOP,
            title,
            FontId::proportional(font_size - 2.0),
            colors.subgraph_title,
        );
    }
}

fn draw_node(
    painter: &egui::Painter,
    node: &FlowNode,
    layout: &NodeLayout,
    offset: Vec2,
    colors: &FlowchartColors,
    font_size: f32,
) {
    let rect = Rect::from_min_size(layout.pos + offset, layout.size);
    let center = rect.center();
    let stroke = Stroke::new(2.0, colors.node_stroke);

    match node.shape {
        NodeShape::Rectangle | NodeShape::Subroutine => {
            painter.rect(rect, Rounding::same(4.0), colors.node_fill, stroke);
            if matches!(node.shape, NodeShape::Subroutine) {
                // Draw double vertical lines
                let inset = 8.0;
                painter.line_segment(
                    [Pos2::new(rect.left() + inset, rect.top()), Pos2::new(rect.left() + inset, rect.bottom())],
                    stroke,
                );
                painter.line_segment(
                    [Pos2::new(rect.right() - inset, rect.top()), Pos2::new(rect.right() - inset, rect.bottom())],
                    stroke,
                );
            }
        }
        NodeShape::RoundRect | NodeShape::Stadium => {
            let rounding = if matches!(node.shape, NodeShape::Stadium) {
                Rounding::same(layout.size.y / 2.0)
            } else {
                Rounding::same(12.0)
            };
            painter.rect(rect, rounding, colors.node_fill, stroke);
        }
        NodeShape::Diamond => {
            let points = [
                Pos2::new(center.x, rect.top()),
                Pos2::new(rect.right(), center.y),
                Pos2::new(center.x, rect.bottom()),
                Pos2::new(rect.left(), center.y),
            ];
            painter.add(egui::Shape::convex_polygon(
                points.to_vec(),
                colors.diamond_fill,
                stroke,
            ));
        }
        NodeShape::Circle => {
            let radius = layout.size.x.min(layout.size.y) / 2.0;
            painter.circle(center, radius, colors.circle_fill, stroke);
        }
        NodeShape::Hexagon => {
            let inset = layout.size.x * 0.15;
            let points = [
                Pos2::new(rect.left() + inset, rect.top()),
                Pos2::new(rect.right() - inset, rect.top()),
                Pos2::new(rect.right(), center.y),
                Pos2::new(rect.right() - inset, rect.bottom()),
                Pos2::new(rect.left() + inset, rect.bottom()),
                Pos2::new(rect.left(), center.y),
            ];
            painter.add(egui::Shape::convex_polygon(
                points.to_vec(),
                colors.node_fill,
                stroke,
            ));
        }
        NodeShape::Cylinder => {
            // Simplified cylinder as rounded rect with ellipse hints
            painter.rect(rect, Rounding::same(4.0), colors.node_fill, stroke);
            let ellipse_height = 8.0;
            painter.line_segment(
                [
                    Pos2::new(rect.left(), rect.top() + ellipse_height),
                    Pos2::new(rect.right(), rect.top() + ellipse_height),
                ],
                Stroke::new(1.0, colors.node_stroke.gamma_multiply(0.5)),
            );
        }
        NodeShape::Parallelogram => {
            let skew = layout.size.x * 0.15;
            let points = [
                Pos2::new(rect.left() + skew, rect.top()),
                Pos2::new(rect.right(), rect.top()),
                Pos2::new(rect.right() - skew, rect.bottom()),
                Pos2::new(rect.left(), rect.bottom()),
            ];
            painter.add(egui::Shape::convex_polygon(
                points.to_vec(),
                colors.node_fill,
                stroke,
            ));
        }
        NodeShape::Asymmetric => {
            let indent = layout.size.y * 0.3;
            let points = [
                Pos2::new(rect.left() + indent, rect.top()),
                Pos2::new(rect.right(), rect.top()),
                Pos2::new(rect.right(), rect.bottom()),
                Pos2::new(rect.left() + indent, rect.bottom()),
                Pos2::new(rect.left(), center.y),
            ];
            painter.add(egui::Shape::convex_polygon(
                points.to_vec(),
                colors.node_fill,
                stroke,
            ));
        }
    }

    // Draw text
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        &node.label,
        FontId::proportional(font_size),
        colors.node_text,
    );
}

fn draw_edge(
    painter: &egui::Painter,
    edge: &FlowEdge,
    from_layout: &NodeLayout,
    to_layout: &NodeLayout,
    offset: Vec2,
    colors: &FlowchartColors,
    label_font_size: f32,
    direction: FlowDirection,
    label_info: Option<&EdgeLabelInfo>,
    is_back_edge: bool,
    total_size: &Vec2,
) {
    let from_rect = Rect::from_min_size(from_layout.pos + offset, from_layout.size);
    let to_rect = Rect::from_min_size(to_layout.pos + offset, to_layout.size);

    // Edge style
    let stroke_width = match edge.style {
        EdgeStyle::Solid => 2.0,
        EdgeStyle::Dotted => 1.5,
        EdgeStyle::Thick => 3.0,
    };

    let stroke = Stroke::new(stroke_width, colors.edge_stroke);

    // Handle back-edges with curved routing (like Mermaid)
    if is_back_edge {
        let (start, end, ctrl1, ctrl2) = match direction {
            FlowDirection::TopDown => {
                // Back-edge goes up: start from top of source, end at BOTTOM-CENTER of target
                let start = Pos2::new(from_rect.left(), from_rect.center().y);
                let end = Pos2::new(to_rect.center().x, to_rect.bottom());
                // Curve: go left from start, then curve up and right to bottom of target
                let curve_x = start.x - 40.0;
                let ctrl1 = Pos2::new(curve_x, start.y);
                let ctrl2 = Pos2::new(curve_x, end.y + 30.0);
                (start, end, ctrl1, ctrl2)
            }
            FlowDirection::BottomUp => {
                let start = Pos2::new(from_rect.right(), from_rect.center().y);
                let end = Pos2::new(to_rect.right(), to_rect.center().y);
                let curve_x = start.x.max(end.x) + 30.0;
                let ctrl1 = Pos2::new(curve_x, start.y);
                let ctrl2 = Pos2::new(curve_x, end.y);
                (start, end, ctrl1, ctrl2)
            }
            FlowDirection::LeftRight => {
                let start = Pos2::new(from_rect.center().x, from_rect.top());
                let end = Pos2::new(to_rect.center().x, to_rect.top());
                let curve_y = start.y.min(end.y) - 30.0;
                let ctrl1 = Pos2::new(start.x, curve_y);
                let ctrl2 = Pos2::new(end.x, curve_y);
                (start, end, ctrl1, ctrl2)
            }
            FlowDirection::RightLeft => {
                let start = Pos2::new(from_rect.center().x, from_rect.bottom());
                let end = Pos2::new(to_rect.center().x, to_rect.bottom());
                let curve_y = start.y.max(end.y) + 30.0;
                let ctrl1 = Pos2::new(start.x, curve_y);
                let ctrl2 = Pos2::new(end.x, curve_y);
                (start, end, ctrl1, ctrl2)
            }
        };

        // Draw cubic bezier curve
        draw_bezier_curve(painter, start, ctrl1, ctrl2, end, stroke);

        // Arrow at end - calculate direction from last curve segment
        if !matches!(edge.arrow_end, ArrowHead::None) {
            // Approximate arrow direction from control point to end
            draw_arrow_head(painter, ctrl2, end, &edge.arrow_end, colors.edge_stroke, stroke_width);
        }

        // Label at midpoint of the curve
        if let Some(info) = label_info {
            // Approximate midpoint of bezier
            let t = 0.5;
            let mid = bezier_point(start, ctrl1, ctrl2, end, t);
            let label_pos = Pos2::new(mid.x - info.size.x / 2.0 - 8.0, mid.y);
            let label_rect = Rect::from_center_size(label_pos, info.size);
            painter.rect_filled(label_rect, Rounding::same(3.0), colors.edge_label_bg);
            painter.text(
                label_pos,
                egui::Align2::CENTER_CENTER,
                &info.display_text,
                FontId::proportional(label_font_size),
                colors.edge_label_text,
            );
        }
    } else {
        // Normal edge - use smart routing based on relative positions
        // For diamond/decision nodes, exit from corner closest to target
        let (start, end) = match direction {
            FlowDirection::TopDown => {
                let from_center_x = from_rect.center().x;
                let to_center_x = to_rect.center().x;
                
                // Determine exit point based on target position relative to source
                // This prevents crossing edges from decision nodes
                let start_x = if (to_center_x - from_center_x).abs() < 10.0 {
                    // Target is roughly centered - exit from center
                    from_center_x
                } else if to_center_x < from_center_x {
                    // Target is to the left - exit from left side of bottom
                    from_rect.center().x - from_rect.width() * 0.25
                } else {
                    // Target is to the right - exit from right side of bottom  
                    from_rect.center().x + from_rect.width() * 0.25
                };
                
                (
                    Pos2::new(start_x, from_rect.bottom()),
                    Pos2::new(to_rect.center().x, to_rect.top()),
                )
            }
            FlowDirection::BottomUp => {
                let from_center_x = from_rect.center().x;
                let to_center_x = to_rect.center().x;
                
                let start_x = if (to_center_x - from_center_x).abs() < 10.0 {
                    from_center_x
                } else if to_center_x < from_center_x {
                    from_rect.center().x - from_rect.width() * 0.25
                } else {
                    from_rect.center().x + from_rect.width() * 0.25
                };
                
                (
                    Pos2::new(start_x, from_rect.top()),
                    Pos2::new(to_rect.center().x, to_rect.bottom()),
                )
            }
            FlowDirection::LeftRight => {
                let from_center_y = from_rect.center().y;
                let to_center_y = to_rect.center().y;
                
                let start_y = if (to_center_y - from_center_y).abs() < 10.0 {
                    from_center_y
                } else if to_center_y < from_center_y {
                    from_rect.center().y - from_rect.height() * 0.25
                } else {
                    from_rect.center().y + from_rect.height() * 0.25
                };
                
                (
                    Pos2::new(from_rect.right(), start_y),
                    Pos2::new(to_rect.left(), to_rect.center().y),
                )
            }
            FlowDirection::RightLeft => {
                let from_center_y = from_rect.center().y;
                let to_center_y = to_rect.center().y;
                
                let start_y = if (to_center_y - from_center_y).abs() < 10.0 {
                    from_center_y
                } else if to_center_y < from_center_y {
                    from_rect.center().y - from_rect.height() * 0.25
                } else {
                    from_rect.center().y + from_rect.height() * 0.25
                };
                
                (
                    Pos2::new(from_rect.left(), start_y),
                    Pos2::new(to_rect.right(), to_rect.center().y),
                )
            }
        };

        // Draw the line
        if matches!(edge.style, EdgeStyle::Dotted) {
            draw_dashed_line(painter, start, end, stroke, 5.0, 3.0);
        } else {
            painter.line_segment([start, end], stroke);
        }

        // Draw arrow head at end
        if !matches!(edge.arrow_end, ArrowHead::None) {
            draw_arrow_head(painter, start, end, &edge.arrow_end, colors.edge_stroke, stroke_width);
        }

        // Draw arrow head at start (for bidirectional)
        if !matches!(edge.arrow_start, ArrowHead::None) {
            draw_arrow_head(painter, end, start, &edge.arrow_start, colors.edge_stroke, stroke_width);
        }

        // Draw edge label using pre-computed sizes
        if let Some(info) = label_info {
            let mid = Pos2::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
            let label_rect = Rect::from_center_size(mid, info.size);

            painter.rect_filled(label_rect, Rounding::same(3.0), colors.edge_label_bg);
            painter.text(
                mid,
                egui::Align2::CENTER_CENTER,
                &info.display_text,
                FontId::proportional(label_font_size),
                colors.edge_label_text,
            );
        }
    }
}

fn draw_dashed_line(painter: &egui::Painter, start: Pos2, end: Pos2, stroke: Stroke, dash_len: f32, gap_len: f32) {
    let dir = (end - start).normalized();
    let total_len = (end - start).length();
    let mut pos = 0.0;
    
    while pos < total_len {
        let seg_start = start + dir * pos;
        let seg_end_pos = (pos + dash_len).min(total_len);
        let seg_end = start + dir * seg_end_pos;
        painter.line_segment([seg_start, seg_end], stroke);
        pos = seg_end_pos + gap_len;
    }
}

/// Calculate a point on a cubic bezier curve at parameter t (0..1)
fn bezier_point(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    
    Pos2::new(
        mt3 * p0.x + 3.0 * mt2 * t * p1.x + 3.0 * mt * t2 * p2.x + t3 * p3.x,
        mt3 * p0.y + 3.0 * mt2 * t * p1.y + 3.0 * mt * t2 * p2.y + t3 * p3.y,
    )
}

/// Draw a cubic bezier curve by sampling points
fn draw_bezier_curve(painter: &egui::Painter, p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, stroke: Stroke) {
    let segments = 20; // Number of line segments to approximate the curve
    let mut prev = p0;
    
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let curr = bezier_point(p0, p1, p2, p3, t);
        painter.line_segment([prev, curr], stroke);
        prev = curr;
    }
}

fn draw_arrow_head(painter: &egui::Painter, from: Pos2, to: Pos2, head_type: &ArrowHead, color: Color32, stroke_width: f32) {
    let dir = (to - from).normalized();
    let perp = Vec2::new(-dir.y, dir.x);
    let arrow_size = 8.0 + stroke_width;
    
    match head_type {
        ArrowHead::Arrow => {
            let tip = to;
            let left = to - dir * arrow_size + perp * (arrow_size * 0.5);
            let right = to - dir * arrow_size - perp * (arrow_size * 0.5);
            painter.add(egui::Shape::convex_polygon(
                vec![tip, left, right],
                color,
                Stroke::NONE,
            ));
        }
        ArrowHead::Circle => {
            painter.circle_filled(to - dir * 4.0, 4.0, color);
        }
        ArrowHead::Cross => {
            let center = to - dir * 4.0;
            let size = 4.0;
            painter.line_segment(
                [center - perp * size - dir * size, center + perp * size + dir * size],
                Stroke::new(stroke_width, color),
            );
            painter.line_segment(
                [center + perp * size - dir * size, center - perp * size + dir * size],
                Stroke::new(stroke_width, color),
            );
        }
        ArrowHead::None => {}
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Sequence Diagram Types
// ─────────────────────────────────────────────────────────────────────────────

/// A participant in a sequence diagram.
#[derive(Debug, Clone)]
pub struct Participant {
    pub id: String,
    pub label: String,
    pub is_actor: bool,
}

/// Type of message arrow in sequence diagram.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MessageType {
    #[default]
    Solid,       // ->>
    SolidOpen,   // ->
    Dotted,      // -->>
    DottedOpen,  // -->
}

/// A message between participants.
#[derive(Debug, Clone)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub label: String,
    pub message_type: MessageType,
    /// Activate the target participant when this message is sent
    pub activate_target: bool,
    /// Deactivate the target participant when this message is sent
    pub deactivate_target: bool,
}

/// Position for a note in a sequence diagram.
#[derive(Debug, Clone)]
pub enum NotePosition {
    LeftOf(String),
    RightOf(String),
    Over(Vec<String>),
}

/// A note in a sequence diagram.
#[derive(Debug, Clone)]
pub struct SeqNote {
    pub position: NotePosition,
    pub text: String,
}

/// Type of control-flow block in sequence diagram.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqBlockKind {
    Loop,
    Alt,
    Opt,
    Par,
}

impl SeqBlockKind {
    fn from_keyword(kw: &str) -> Option<Self> {
        match kw {
            "loop" => Some(Self::Loop),
            "alt" => Some(Self::Alt),
            "opt" => Some(Self::Opt),
            "par" => Some(Self::Par),
            _ => None,
        }
    }
    
    fn display_name(&self) -> &'static str {
        match self {
            Self::Loop => "loop",
            Self::Alt => "alt",
            Self::Opt => "opt",
            Self::Par => "par",
        }
    }
}

/// A segment within a control-flow block (e.g., alt branches, par branches).
#[derive(Debug, Clone)]
pub struct SeqBlockSegment {
    pub segment_label: Option<String>,
    pub statements: Vec<SeqStatement>,
}

/// A control-flow block in a sequence diagram (loop, alt, opt, par).
#[derive(Debug, Clone)]
pub struct SeqBlock {
    pub kind: SeqBlockKind,
    pub label: String,
    pub segments: Vec<SeqBlockSegment>,
}

/// A statement in a sequence diagram - message, block, note, or activation directive.
#[derive(Debug, Clone)]
pub enum SeqStatement {
    Message(Message),
    Block(SeqBlock),
    Note(SeqNote),
    Activate(String),
    Deactivate(String),
}

/// A parsed sequence diagram.
#[derive(Debug, Clone, Default)]
pub struct SequenceDiagram {
    pub participants: Vec<Participant>,
    pub statements: Vec<SeqStatement>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Sequence Diagram Parser
// ─────────────────────────────────────────────────────────────────────────────

/// Helper struct for building control-flow blocks during parsing.
struct SeqBlockBuilder {
    kind: SeqBlockKind,
    label: String,
    segments: Vec<SeqBlockSegment>,
    current_segment_label: Option<String>,
    current_segment_statements: Vec<SeqStatement>,
}

impl SeqBlockBuilder {
    fn new(kind: SeqBlockKind, label: String) -> Self {
        Self {
            kind,
            label,
            segments: Vec::new(),
            // First segment label is shown in the block header, not as a segment label
            current_segment_label: None,
            current_segment_statements: Vec::new(),
        }
    }
    
    fn start_new_segment(&mut self, label: Option<String>) {
        // Finalize current segment
        if !self.current_segment_statements.is_empty() || self.current_segment_label.is_some() {
            self.segments.push(SeqBlockSegment {
                segment_label: self.current_segment_label.take(),
                statements: std::mem::take(&mut self.current_segment_statements),
            });
        }
        self.current_segment_label = label;
    }
    
    fn add_statement(&mut self, stmt: SeqStatement) {
        self.current_segment_statements.push(stmt);
    }
    
    fn finalize(mut self) -> SeqBlock {
        // Finalize the last segment
        self.segments.push(SeqBlockSegment {
            segment_label: self.current_segment_label,
            statements: self.current_segment_statements,
        });
        
        SeqBlock {
            kind: self.kind,
            label: self.label,
            segments: self.segments,
        }
    }
}

/// Parse mermaid sequence diagram source.
pub fn parse_sequence_diagram(source: &str) -> Result<SequenceDiagram, String> {
    let mut diagram = SequenceDiagram::default();
    let mut participant_map: HashMap<String, usize> = HashMap::new();
    let mut block_stack: Vec<SeqBlockBuilder> = Vec::new();
    let lines: Vec<&str> = source.lines().skip(1).collect(); // Skip "sequenceDiagram" header

    for (line_num, line) in lines.iter().enumerate() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

        // Parse participant declaration
        if line.starts_with("participant ") || line.starts_with("actor ") {
            let is_actor = line.starts_with("actor ");
            let rest = if is_actor { &line[6..] } else { &line[12..] };
            let rest = rest.trim();
            
            // Check for "as" alias: participant A as Alice
            let (id, label) = if let Some(as_pos) = rest.find(" as ") {
                let id = rest[..as_pos].trim().to_string();
                let label = rest[as_pos + 4..].trim().to_string();
                (id, label)
            } else {
                (rest.to_string(), rest.to_string())
            };
            
            if !participant_map.contains_key(&id) {
                participant_map.insert(id.clone(), diagram.participants.len());
                diagram.participants.push(Participant { id, label, is_actor });
            }
            continue;
        }

        // Parse control-flow block keywords
        let first_word = line.split_whitespace().next().unwrap_or("");
        
        // Handle block start: loop, alt, opt, par
        if let Some(kind) = SeqBlockKind::from_keyword(first_word) {
            let label = line[first_word.len()..].trim().to_string();
            block_stack.push(SeqBlockBuilder::new(kind, label));
            continue;
        }
        
        // Handle else (for alt blocks)
        if first_word == "else" {
            if let Some(builder) = block_stack.last_mut() {
                if builder.kind == SeqBlockKind::Alt {
                    let label = line[4..].trim();
                    let label = if label.is_empty() { None } else { Some(label.to_string()) };
                    builder.start_new_segment(label);
                } else {
                    return Err(format!(
                        "Line {}: 'else' can only be used inside 'alt' blocks, found inside '{}'",
                        line_num + 2, builder.kind.display_name()
                    ));
                }
            } else {
                return Err(format!("Line {}: 'else' without matching 'alt' block", line_num + 2));
            }
            continue;
        }
        
        // Handle and (for par blocks)
        if first_word == "and" {
            if let Some(builder) = block_stack.last_mut() {
                if builder.kind == SeqBlockKind::Par {
                    let label = line[3..].trim();
                    let label = if label.is_empty() { None } else { Some(label.to_string()) };
                    builder.start_new_segment(label);
                } else {
                    return Err(format!(
                        "Line {}: 'and' can only be used inside 'par' blocks, found inside '{}'",
                        line_num + 2, builder.kind.display_name()
                    ));
                }
            } else {
                return Err(format!("Line {}: 'and' without matching 'par' block", line_num + 2));
            }
            continue;
        }
        
        // Handle end (close current block)
        if first_word == "end" {
            if let Some(builder) = block_stack.pop() {
                let block = builder.finalize();
                let stmt = SeqStatement::Block(block);
                
                // Add to parent block or top-level
                if let Some(parent) = block_stack.last_mut() {
                    parent.add_statement(stmt);
                } else {
                    diagram.statements.push(stmt);
                }
            } else {
                return Err(format!("Line {}: 'end' without matching block opener", line_num + 2));
            }
            continue;
        }

        // Parse activate/deactivate commands
        if first_word == "activate" {
            let participant_id = line[8..].trim().to_string();
            if !participant_id.is_empty() {
                // Auto-add participant if not declared
                if !participant_map.contains_key(&participant_id) {
                    participant_map.insert(participant_id.clone(), diagram.participants.len());
                    diagram.participants.push(Participant {
                        id: participant_id.clone(),
                        label: participant_id.clone(),
                        is_actor: false,
                    });
                }
                
                let stmt = SeqStatement::Activate(participant_id);
                if let Some(builder) = block_stack.last_mut() {
                    builder.add_statement(stmt);
                } else {
                    diagram.statements.push(stmt);
                }
            }
            continue;
        }
        
        if first_word == "deactivate" {
            let participant_id = line[10..].trim().to_string();
            if !participant_id.is_empty() {
                let stmt = SeqStatement::Deactivate(participant_id);
                if let Some(builder) = block_stack.last_mut() {
                    builder.add_statement(stmt);
                } else {
                    diagram.statements.push(stmt);
                }
            }
            continue;
        }

        // Parse Note syntax
        if let Some(note) = parse_sequence_note(line) {
            // Auto-add participants referenced in note
            match &note.position {
                NotePosition::LeftOf(id) | NotePosition::RightOf(id) => {
                    if !participant_map.contains_key(id) {
                        participant_map.insert(id.clone(), diagram.participants.len());
                        diagram.participants.push(Participant {
                            id: id.clone(),
                            label: id.clone(),
                            is_actor: false,
                        });
                    }
                }
                NotePosition::Over(ids) => {
                    for id in ids {
                        if !participant_map.contains_key(id) {
                            participant_map.insert(id.clone(), diagram.participants.len());
                            diagram.participants.push(Participant {
                                id: id.clone(),
                                label: id.clone(),
                                is_actor: false,
                            });
                        }
                    }
                }
            }
            
            let stmt = SeqStatement::Note(note);
            if let Some(builder) = block_stack.last_mut() {
                builder.add_statement(stmt);
            } else {
                diagram.statements.push(stmt);
            }
            continue;
        }

        // Parse message: A->>B: Message or A-->>B: Message (including +/- shorthand)
        if let Some(msg) = parse_sequence_message(line) {
            // Auto-add participants if not declared
            for id in [&msg.from, &msg.to] {
                if !participant_map.contains_key(id) {
                    participant_map.insert(id.clone(), diagram.participants.len());
                    diagram.participants.push(Participant {
                        id: id.clone(),
                        label: id.clone(),
                        is_actor: false,
                    });
                }
            }
            
            let stmt = SeqStatement::Message(msg);
            
            // Add to current block or top-level
            if let Some(builder) = block_stack.last_mut() {
                builder.add_statement(stmt);
            } else {
                diagram.statements.push(stmt);
            }
        }
    }
    
    // Check for unclosed blocks
    if let Some(builder) = block_stack.last() {
        return Err(format!(
            "Unclosed '{}' block at end of diagram",
            builder.kind.display_name()
        ));
    }

    if diagram.participants.is_empty() {
        return Err("No participants found in sequence diagram".to_string());
    }

    Ok(diagram)
}

fn parse_sequence_message(line: &str) -> Option<Message> {
    // Arrow patterns to check (order matters - check longer patterns first)
    // Include patterns with +/- for activation shorthand
    let arrow_patterns = [
        ("-->>+", MessageType::Dotted, false, true),
        ("-->>-", MessageType::Dotted, false, false), // deactivate handled separately
        ("->>+", MessageType::Solid, false, true),
        ("->>-", MessageType::Solid, false, false),
        ("-->+", MessageType::DottedOpen, false, true),
        ("-->-", MessageType::DottedOpen, false, false),
        ("->+", MessageType::SolidOpen, false, true),
        ("->-", MessageType::SolidOpen, false, false),
        ("-->>", MessageType::Dotted, false, false),
        ("->>", MessageType::Solid, false, false),
        ("-->", MessageType::DottedOpen, false, false),
        ("->", MessageType::SolidOpen, false, false),
    ];

    for (pattern, msg_type, _, is_activate) in arrow_patterns {
        if let Some(arrow_pos) = line.find(pattern) {
            let from = line[..arrow_pos].trim();
            let rest = &line[arrow_pos + pattern.len()..];
            
            // Check for deactivate (pattern ends with -)
            let is_deactivate = pattern.ends_with('-') && !pattern.ends_with("->") && !pattern.ends_with(">>");
            let activate_target = is_activate || pattern.ends_with('+');
            let deactivate_target = is_deactivate;
            
            // Find the colon for the message label
            let (to, label) = if let Some(colon_pos) = rest.find(':') {
                let to = rest[..colon_pos].trim();
                let label = rest[colon_pos + 1..].trim();
                (to, label)
            } else {
                (rest.trim(), "")
            };

            if !from.is_empty() && !to.is_empty() {
                return Some(Message {
                    from: from.to_string(),
                    to: to.to_string(),
                    label: label.to_string(),
                    message_type: msg_type,
                    activate_target,
                    deactivate_target,
                });
            }
        }
    }

    None
}

/// Parse a Note statement in a sequence diagram.
/// Supports: Note left of X:, Note right of X:, Note over X:, Note over X,Y:
fn parse_sequence_note(line: &str) -> Option<SeqNote> {
    let line_lower = line.to_lowercase();
    
    if !line_lower.starts_with("note ") {
        return None;
    }
    
    let rest = &line[5..]; // Skip "Note "
    
    // Find the colon separator
    let colon_pos = rest.find(':')?;
    let position_part = rest[..colon_pos].trim();
    let text = rest[colon_pos + 1..].trim().to_string();
    
    let position_lower = position_part.to_lowercase();
    
    let position = if position_lower.starts_with("left of ") {
        let participant = position_part[8..].trim().to_string();
        NotePosition::LeftOf(participant)
    } else if position_lower.starts_with("right of ") {
        let participant = position_part[9..].trim().to_string();
        NotePosition::RightOf(participant)
    } else if position_lower.starts_with("over ") {
        let participants_str = position_part[5..].trim();
        let participants: Vec<String> = participants_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if participants.is_empty() {
            return None;
        }
        NotePosition::Over(participants)
    } else {
        return None;
    };
    
    Some(SeqNote { position, text })
}

// ─────────────────────────────────────────────────────────────────────────────
// Sequence Diagram Renderer
// ─────────────────────────────────────────────────────────────────────────────

/// Colors for sequence diagram rendering.
struct SeqColors {
    bg: Color32,
    stroke: Color32,
    text: Color32,
    lifeline: Color32,
    actor: Color32,
    block_loop: Color32,
    block_alt: Color32,
    block_opt: Color32,
    block_par: Color32,
    block_stroke: Color32,
    block_text: Color32,
    activation_fill: Color32,
    activation_stroke: Color32,
    note_fill: Color32,
    note_stroke: Color32,
    note_text: Color32,
}

impl SeqColors {
    fn new(dark_mode: bool) -> Self {
        if dark_mode {
            Self {
                bg: Color32::from_rgb(45, 55, 72),
                stroke: Color32::from_rgb(100, 140, 180),
                text: Color32::from_rgb(220, 230, 240),
                lifeline: Color32::from_rgb(80, 100, 120),
                actor: Color32::from_rgb(100, 160, 220),
                block_loop: Color32::from_rgba_unmultiplied(100, 180, 100, 30),
                block_alt: Color32::from_rgba_unmultiplied(180, 140, 100, 30),
                block_opt: Color32::from_rgba_unmultiplied(100, 140, 180, 30),
                block_par: Color32::from_rgba_unmultiplied(180, 100, 180, 30),
                block_stroke: Color32::from_rgb(120, 140, 160),
                block_text: Color32::from_rgb(200, 210, 220),
                activation_fill: Color32::from_rgb(70, 90, 110),
                activation_stroke: Color32::from_rgb(100, 140, 180),
                note_fill: Color32::from_rgb(80, 80, 60),
                note_stroke: Color32::from_rgb(140, 140, 100),
                note_text: Color32::from_rgb(220, 220, 200),
            }
        } else {
            Self {
                bg: Color32::from_rgb(240, 245, 250),
                stroke: Color32::from_rgb(100, 140, 180),
                text: Color32::from_rgb(30, 40, 50),
                lifeline: Color32::from_rgb(180, 190, 200),
                actor: Color32::from_rgb(50, 120, 180),
                block_loop: Color32::from_rgba_unmultiplied(100, 180, 100, 40),
                block_alt: Color32::from_rgba_unmultiplied(220, 180, 100, 40),
                block_opt: Color32::from_rgba_unmultiplied(100, 140, 220, 40),
                block_par: Color32::from_rgba_unmultiplied(200, 100, 200, 40),
                block_stroke: Color32::from_rgb(100, 120, 140),
                block_text: Color32::from_rgb(40, 50, 60),
                activation_fill: Color32::from_rgb(200, 220, 240),
                activation_stroke: Color32::from_rgb(100, 140, 180),
                note_fill: Color32::from_rgb(255, 255, 220),
                note_stroke: Color32::from_rgb(180, 180, 140),
                note_text: Color32::from_rgb(60, 60, 40),
            }
        }
    }
    
    fn block_fill(&self, kind: &SeqBlockKind) -> Color32 {
        match kind {
            SeqBlockKind::Loop => self.block_loop,
            SeqBlockKind::Alt => self.block_alt,
            SeqBlockKind::Opt => self.block_opt,
            SeqBlockKind::Par => self.block_par,
        }
    }
}

/// Count total message slots needed for a list of statements (recursive for blocks).
fn count_statement_slots(statements: &[SeqStatement]) -> usize {
    let mut count = 0;
    for stmt in statements {
        match stmt {
            SeqStatement::Message(_) => count += 1,
            SeqStatement::Note(_) => count += 1,
            SeqStatement::Activate(_) | SeqStatement::Deactivate(_) => {
                // Activation directives don't take visual slots
            }
            SeqStatement::Block(block) => {
                // Add header slot for the block label
                count += 1;
                for segment in &block.segments {
                    count += count_statement_slots(&segment.statements);
                    // Add separator slot between segments (except for last)
                }
                // Add footer slot for the block
                count += 1;
            }
        }
    }
    count
}

/// Layout constants for sequence diagram.
struct SeqLayout {
    min_participant_width: f32,
    participant_height: f32,
    participant_spacing: f32,
    message_height: f32,
    margin: f32,
    lifeline_extend: f32,
    participant_padding: f32,
    block_padding: f32,
    block_label_height: f32,
    activation_width: f32,
    activation_offset: f32,
    note_width: f32,
    note_padding: f32,
    note_corner_size: f32,
}

impl Default for SeqLayout {
    fn default() -> Self {
        Self {
            min_participant_width: 80.0,
            participant_height: 40.0,
            participant_spacing: 50.0,
            message_height: 40.0,
            margin: 20.0,
            lifeline_extend: 30.0,
            participant_padding: 24.0,
            block_padding: 8.0,
            block_label_height: 20.0,
            activation_width: 10.0,
            activation_offset: 4.0,
            note_width: 100.0,
            note_padding: 8.0,
            note_corner_size: 8.0,
        }
    }
}

/// Render a sequence diagram to the UI.
pub fn render_sequence_diagram(
    ui: &mut Ui,
    diagram: &SequenceDiagram,
    dark_mode: bool,
    font_size: f32,
) {
    if diagram.participants.is_empty() {
        return;
    }

    let layout = SeqLayout::default();
    let colors = SeqColors::new(dark_mode);

    // Pre-measure participant widths
    let participant_widths: HashMap<String, f32> = {
        let text_measurer = EguiTextMeasurer::new(ui);
        diagram.participants.iter()
            .map(|p| {
                let text_size = text_measurer.measure(&p.label, font_size);
                let width = (text_size.width * 1.15 + layout.participant_padding).max(layout.min_participant_width);
                (p.id.clone(), width)
            })
            .collect()
    };

    // Calculate total width based on measured participant widths
    let total_participants_width: f32 = participant_widths.values().sum();
    let total_width = layout.margin * 2.0 + 
        total_participants_width +
        (diagram.participants.len().saturating_sub(1)) as f32 * layout.participant_spacing;
    
    // Count total slots needed
    let total_slots = count_statement_slots(&diagram.statements);
    let total_height = layout.margin * 2.0 + 
        layout.participant_height + 
        total_slots as f32 * layout.message_height +
        layout.lifeline_extend;

    // Allocate space
    let (response, painter) = ui.allocate_painter(
        Vec2::new(total_width, total_height),
        egui::Sense::hover(),
    );
    let offset = response.rect.min.to_vec2();

    // Calculate participant positions based on measured widths
    let mut participant_x: HashMap<String, f32> = HashMap::new();
    let mut current_x = layout.margin;

    for participant in &diagram.participants {
        let width = participant_widths.get(&participant.id).copied().unwrap_or(layout.min_participant_width);
        participant_x.insert(participant.id.clone(), current_x + width / 2.0);
        current_x += width + layout.participant_spacing;
    }

    // Get lifeline region boundaries (leftmost to rightmost participant)
    let lifeline_left = diagram.participants.first()
        .and_then(|p| participant_x.get(&p.id))
        .copied()
        .unwrap_or(layout.margin);
    let lifeline_right = diagram.participants.last()
        .and_then(|p| {
            let x = participant_x.get(&p.id)?;
            let w = participant_widths.get(&p.id)?;
            Some(x + w / 2.0)
        })
        .unwrap_or(total_width - layout.margin);

    // Draw lifelines first (behind everything)
    let lifeline_start_y = layout.margin + layout.participant_height;
    let lifeline_end_y = total_height - layout.margin;
    
    for participant in &diagram.participants {
        if let Some(&x) = participant_x.get(&participant.id) {
            painter.line_segment(
                [
                    Pos2::new(x, lifeline_start_y) + offset,
                    Pos2::new(x, lifeline_end_y) + offset,
                ],
                Stroke::new(1.0, colors.lifeline),
            );
        }
    }

    // Draw participants
    for participant in &diagram.participants {
        if let Some(&center_x) = participant_x.get(&participant.id) {
            let width = participant_widths.get(&participant.id).copied().unwrap_or(layout.min_participant_width);
            let rect = Rect::from_center_size(
                Pos2::new(center_x, layout.margin + layout.participant_height / 2.0) + offset,
                Vec2::new(width, layout.participant_height),
            );

            if participant.is_actor {
                draw_actor(&painter, center_x, &rect, offset, &colors, font_size, &participant.label);
            } else {
                painter.rect(rect, Rounding::same(4.0), colors.bg, Stroke::new(2.0, colors.stroke));
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &participant.label,
                    FontId::proportional(font_size),
                    colors.text,
                );
            }
        }
    }

    // Draw statements (messages, blocks, notes, activations)
    let mut current_y = layout.margin + layout.participant_height + layout.message_height / 2.0;
    let mut activation_state: HashMap<String, ActivationState> = HashMap::new();
    
    draw_statements(
        &painter,
        &diagram.statements,
        &participant_x,
        &mut current_y,
        offset,
        &layout,
        &colors,
        font_size,
        lifeline_left,
        lifeline_right,
        0, // nesting depth
        &mut activation_state,
    );
    
    // Draw any remaining open activations (extend to end of diagram)
    let diagram_end_y = current_y;
    for (participant_id, state) in activation_state.iter() {
        if let Some(&x) = participant_x.get(participant_id) {
            for (i, &start_y) in state.start_ys.iter().enumerate() {
                let depth_offset = i as f32 * layout.activation_offset;
                draw_activation_box(
                    &painter,
                    x + depth_offset,
                    start_y,
                    diagram_end_y,
                    offset,
                    &layout,
                    &colors,
                );
            }
        }
    }
}

/// Draw an actor (stick figure).
fn draw_actor(
    painter: &egui::Painter,
    center_x: f32,
    rect: &Rect,
    offset: Vec2,
    colors: &SeqColors,
    font_size: f32,
    label: &str,
) {
    let head_y = rect.top() + 10.0;
    let body_y = head_y + 15.0;
    let legs_y = body_y + 12.0;
    
    // Head
    painter.circle_stroke(Pos2::new(center_x + offset.x, head_y), 6.0, Stroke::new(2.0, colors.actor));
    // Body
    painter.line_segment(
        [Pos2::new(center_x + offset.x, head_y + 6.0), Pos2::new(center_x + offset.x, body_y)],
        Stroke::new(2.0, colors.actor),
    );
    // Arms
    painter.line_segment(
        [Pos2::new(center_x - 10.0 + offset.x, head_y + 12.0), Pos2::new(center_x + 10.0 + offset.x, head_y + 12.0)],
        Stroke::new(2.0, colors.actor),
    );
    // Legs
    painter.line_segment(
        [Pos2::new(center_x + offset.x, body_y), Pos2::new(center_x - 8.0 + offset.x, legs_y)],
        Stroke::new(2.0, colors.actor),
    );
    painter.line_segment(
        [Pos2::new(center_x + offset.x, body_y), Pos2::new(center_x + 8.0 + offset.x, legs_y)],
        Stroke::new(2.0, colors.actor),
    );
    // Label below
    painter.text(
        Pos2::new(center_x + offset.x, rect.bottom() - 2.0),
        egui::Align2::CENTER_BOTTOM,
        label,
        FontId::proportional(font_size - 2.0),
        colors.text,
    );
}

/// Draw a message arrow between participants.
fn draw_message(
    painter: &egui::Painter,
    message: &Message,
    participant_x: &HashMap<String, f32>,
    y: f32,
    offset: Vec2,
    colors: &SeqColors,
    font_size: f32,
) {
    if let (Some(&from_x), Some(&to_x)) = (participant_x.get(&message.from), participant_x.get(&message.to)) {
        let from_pos = Pos2::new(from_x + offset.x, y);
        let to_pos = Pos2::new(to_x + offset.x, y);
        
        // Determine stroke style
        let stroke = Stroke::new(1.5, colors.stroke);

        // Draw arrow line
        if matches!(message.message_type, MessageType::Dotted | MessageType::DottedOpen) {
            draw_dashed_line(painter, from_pos, to_pos, stroke, 5.0, 3.0);
        } else {
            painter.line_segment([from_pos, to_pos], stroke);
        }

        // Draw arrow head (solid or open)
        let is_solid_head = matches!(message.message_type, MessageType::Solid | MessageType::Dotted);
        let dir = (to_pos - from_pos).normalized();
        let arrow_size = 8.0;
        let perp = Vec2::new(-dir.y, dir.x);
        
        let arrow_tip = to_pos;
        let arrow_left = to_pos - dir * arrow_size + perp * (arrow_size * 0.4);
        let arrow_right = to_pos - dir * arrow_size - perp * (arrow_size * 0.4);

        if is_solid_head {
            painter.add(egui::Shape::convex_polygon(
                vec![arrow_tip, arrow_left, arrow_right],
                colors.stroke,
                Stroke::NONE,
            ));
        } else {
            painter.line_segment([arrow_tip, arrow_left], stroke);
            painter.line_segment([arrow_tip, arrow_right], stroke);
        }

        // Draw message label
        if !message.label.is_empty() {
            let label_pos = Pos2::new((from_x + to_x) / 2.0 + offset.x, y - 8.0);
            painter.text(
                label_pos,
                egui::Align2::CENTER_BOTTOM,
                &message.label,
                FontId::proportional(font_size - 2.0),
                colors.text,
            );
        }
    }
}

/// Track activation state for a participant.
#[derive(Default)]
struct ActivationState {
    /// Stack of activation start y-coordinates (for nesting)
    start_ys: Vec<f32>,
    /// Nesting depth offset for drawing
    depth: usize,
}

/// Draw statements recursively (handles nested blocks).
#[allow(clippy::too_many_arguments)]
fn draw_statements(
    painter: &egui::Painter,
    statements: &[SeqStatement],
    participant_x: &HashMap<String, f32>,
    current_y: &mut f32,
    offset: Vec2,
    layout: &SeqLayout,
    colors: &SeqColors,
    font_size: f32,
    lifeline_left: f32,
    lifeline_right: f32,
    depth: usize,
    activation_state: &mut HashMap<String, ActivationState>,
) {
    for stmt in statements {
        match stmt {
            SeqStatement::Message(message) => {
                let y = *current_y + offset.y;
                
                // Handle activation on message target
                if message.activate_target {
                    let state = activation_state.entry(message.to.clone()).or_default();
                    state.start_ys.push(*current_y);
                    state.depth += 1;
                }
                
                draw_message(painter, message, participant_x, y, offset, colors, font_size);
                *current_y += layout.message_height;
                
                // Handle deactivation on message target
                if message.deactivate_target {
                    if let Some(state) = activation_state.get_mut(&message.to) {
                        if let Some(start_y) = state.start_ys.pop() {
                            if let Some(&x) = participant_x.get(&message.to) {
                                let depth_offset = state.depth.saturating_sub(1) as f32 * layout.activation_offset;
                                draw_activation_box(
                                    painter,
                                    x + depth_offset,
                                    start_y,
                                    *current_y,
                                    offset,
                                    layout,
                                    colors,
                                );
                            }
                            state.depth = state.depth.saturating_sub(1);
                        }
                    }
                }
            }
            SeqStatement::Note(note) => {
                let y = *current_y + offset.y;
                draw_note(painter, note, participant_x, y, offset, layout, colors, font_size);
                *current_y += layout.message_height;
            }
            SeqStatement::Activate(participant_id) => {
                let state = activation_state.entry(participant_id.clone()).or_default();
                state.start_ys.push(*current_y);
                state.depth += 1;
            }
            SeqStatement::Deactivate(participant_id) => {
                if let Some(state) = activation_state.get_mut(participant_id) {
                    if let Some(start_y) = state.start_ys.pop() {
                        if let Some(&x) = participant_x.get(participant_id) {
                            let depth_offset = state.depth.saturating_sub(1) as f32 * layout.activation_offset;
                            draw_activation_box(
                                painter,
                                x + depth_offset,
                                start_y,
                                *current_y,
                                offset,
                                layout,
                                colors,
                            );
                        }
                        state.depth = state.depth.saturating_sub(1);
                    }
                }
            }
            SeqStatement::Block(block) => {
                draw_block(
                    painter,
                    block,
                    participant_x,
                    current_y,
                    offset,
                    layout,
                    colors,
                    font_size,
                    lifeline_left,
                    lifeline_right,
                    depth,
                    activation_state,
                );
            }
        }
    }
}

/// Draw an activation box on a lifeline.
fn draw_activation_box(
    painter: &egui::Painter,
    x: f32,
    start_y: f32,
    end_y: f32,
    offset: Vec2,
    layout: &SeqLayout,
    colors: &SeqColors,
) {
    let rect = Rect::from_min_max(
        Pos2::new(x - layout.activation_width / 2.0, start_y) + offset,
        Pos2::new(x + layout.activation_width / 2.0, end_y) + offset,
    );
    
    painter.rect(
        rect,
        Rounding::same(2.0),
        colors.activation_fill,
        Stroke::new(1.5, colors.activation_stroke),
    );
}

/// Draw a note in a sequence diagram.
fn draw_note(
    painter: &egui::Painter,
    note: &SeqNote,
    participant_x: &HashMap<String, f32>,
    y: f32,
    offset: Vec2,
    layout: &SeqLayout,
    colors: &SeqColors,
    font_size: f32,
) {
    // Calculate note position based on NotePosition
    let (note_x, note_width) = match &note.position {
        NotePosition::LeftOf(participant) => {
            if let Some(&x) = participant_x.get(participant) {
                (x - layout.note_width - layout.participant_spacing / 2.0, layout.note_width)
            } else {
                return;
            }
        }
        NotePosition::RightOf(participant) => {
            if let Some(&x) = participant_x.get(participant) {
                (x + layout.participant_spacing / 2.0, layout.note_width)
            } else {
                return;
            }
        }
        NotePosition::Over(participants) => {
            if participants.is_empty() {
                return;
            }
            
            let xs: Vec<f32> = participants.iter()
                .filter_map(|id| participant_x.get(id).copied())
                .collect();
            
            if xs.is_empty() {
                return;
            }
            
            let min_x = xs.iter().copied().fold(f32::INFINITY, f32::min);
            let max_x = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let width = (max_x - min_x).max(layout.note_width);
            let center_x = (min_x + max_x) / 2.0;
            
            (center_x - width / 2.0, width)
        }
    };
    
    let note_height = layout.message_height - layout.note_padding;
    let note_rect = Rect::from_min_size(
        Pos2::new(note_x, y - note_height / 2.0) + offset,
        Vec2::new(note_width, note_height),
    );
    
    // Draw note background with dog-ear corner
    let corner = layout.note_corner_size;
    let points = vec![
        note_rect.left_top(),
        Pos2::new(note_rect.right() - corner, note_rect.top()),
        Pos2::new(note_rect.right(), note_rect.top() + corner),
        note_rect.right_bottom(),
        note_rect.left_bottom(),
    ];
    
    painter.add(egui::Shape::convex_polygon(
        points,
        colors.note_fill,
        Stroke::new(1.0, colors.note_stroke),
    ));
    
    // Draw the dog-ear fold line
    painter.line_segment(
        [
            Pos2::new(note_rect.right() - corner, note_rect.top()),
            Pos2::new(note_rect.right() - corner, note_rect.top() + corner),
        ],
        Stroke::new(1.0, colors.note_stroke),
    );
    painter.line_segment(
        [
            Pos2::new(note_rect.right() - corner, note_rect.top() + corner),
            Pos2::new(note_rect.right(), note_rect.top() + corner),
        ],
        Stroke::new(1.0, colors.note_stroke),
    );
    
    // Draw note text
    painter.text(
        note_rect.center(),
        egui::Align2::CENTER_CENTER,
        &note.text,
        FontId::proportional(font_size - 2.0),
        colors.note_text,
    );
}

/// Draw a control-flow block (loop, alt, opt, par).
#[allow(clippy::too_many_arguments)]
fn draw_block(
    painter: &egui::Painter,
    block: &SeqBlock,
    participant_x: &HashMap<String, f32>,
    current_y: &mut f32,
    offset: Vec2,
    layout: &SeqLayout,
    colors: &SeqColors,
    font_size: f32,
    lifeline_left: f32,
    lifeline_right: f32,
    depth: usize,
    activation_state: &mut HashMap<String, ActivationState>,
) {
    // Calculate block height by counting all slots
    let mut block_height = layout.block_label_height; // Header
    for (i, segment) in block.segments.iter().enumerate() {
        let segment_slots = count_statement_slots(&segment.statements);
        block_height += segment_slots as f32 * layout.message_height;
        // Add separator height between segments (except for last)
        if i < block.segments.len() - 1 {
            block_height += layout.block_label_height;
        }
    }
    block_height += layout.block_padding * 2.0; // Footer padding
    
    // Inset the block based on nesting depth
    let inset = depth as f32 * layout.block_padding * 2.0;
    let block_left = lifeline_left - layout.block_padding * 4.0 + inset;
    let block_right = lifeline_right + layout.block_padding * 4.0 - inset;
    let block_width = block_right - block_left;
    
    let block_top_y = *current_y - layout.message_height / 2.0 + layout.block_padding;
    
    // Draw block background
    let block_rect = Rect::from_min_size(
        Pos2::new(block_left, block_top_y) + offset,
        Vec2::new(block_width, block_height),
    );
    
    let fill_color = colors.block_fill(&block.kind);
    painter.rect(
        block_rect,
        Rounding::same(4.0),
        fill_color,
        Stroke::new(1.5, colors.block_stroke),
    );
    
    // Draw block label in top-left corner with background
    let label_text = format!("{}", block.kind.display_name());
    let label_with_bracket = if block.label.is_empty() {
        label_text
    } else {
        format!("{} [{}]", label_text, block.label)
    };
    
    let label_rect = Rect::from_min_size(
        Pos2::new(block_left, block_top_y) + offset,
        Vec2::new(label_with_bracket.len() as f32 * 7.0 + 12.0, layout.block_label_height),
    );
    
    // Draw label background (pentagon shape approximated as rectangle with clipped corner)
    painter.rect_filled(
        label_rect,
        Rounding::ZERO,
        colors.block_stroke,
    );
    
    // Draw label text
    painter.text(
        label_rect.left_center() + Vec2::new(6.0, 0.0),
        egui::Align2::LEFT_CENTER,
        &label_with_bracket,
        FontId::proportional(font_size - 3.0),
        Color32::WHITE,
    );
    
    // Move past the header
    *current_y += layout.block_label_height;
    
    // Draw segments
    for (i, segment) in block.segments.iter().enumerate() {
        // Draw segment label (for alt/par segments after the first)
        if i > 0 {
            // Draw separator line
            let sep_y = *current_y - layout.message_height / 2.0 + offset.y;
            painter.line_segment(
                [
                    Pos2::new(block_left + offset.x, sep_y),
                    Pos2::new(block_right + offset.x, sep_y),
                ],
                Stroke::new(1.0, colors.block_stroke),
            );
            
            // Draw segment label
            let segment_label_text = if let Some(label) = &segment.segment_label {
                Some(format!("[{}]", label))
            } else {
                // Default labels for block types
                match block.kind {
                    SeqBlockKind::Alt => Some("[else]".to_string()),
                    SeqBlockKind::Par => Some("[and]".to_string()),
                    _ => None,
                }
            };
            
            if let Some(text) = segment_label_text {
                painter.text(
                    Pos2::new(block_left + 10.0 + offset.x, sep_y + 2.0),
                    egui::Align2::LEFT_TOP,
                    &text,
                    FontId::proportional(font_size - 3.0),
                    colors.block_text,
                );
            }
            
            *current_y += layout.block_label_height;
        }
        
        // Draw statements in this segment
        draw_statements(
            painter,
            &segment.statements,
            participant_x,
            current_y,
            offset,
            layout,
            colors,
            font_size,
            lifeline_left,
            lifeline_right,
            depth + 1,
            activation_state,
        );
    }
    
    // Add footer padding
    *current_y += layout.block_padding * 2.0;
}

// ─────────────────────────────────────────────────────────────────────────────
// Pie Chart Types and Renderer
// ─────────────────────────────────────────────────────────────────────────────

/// A slice of a pie chart.
#[derive(Debug, Clone)]
pub struct PieSlice {
    pub label: String,
    pub value: f32,
}

/// A parsed pie chart.
#[derive(Debug, Clone, Default)]
pub struct PieChart {
    pub title: Option<String>,
    pub slices: Vec<PieSlice>,
}

/// Parse mermaid pie chart source.
pub fn parse_pie_chart(source: &str) -> Result<PieChart, String> {
    let mut chart = PieChart::default();
    
    for line in source.lines().skip(1) {
        let line = line.trim();
        
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

        // Parse title
        if line.starts_with("title ") {
            chart.title = Some(line[6..].trim().to_string());
            continue;
        }

        // Parse slice: "Label" : value
        if let Some(colon_pos) = line.find(':') {
            let label = line[..colon_pos].trim().trim_matches('"').to_string();
            let value_str = line[colon_pos + 1..].trim();
            if let Ok(value) = value_str.parse::<f32>() {
                chart.slices.push(PieSlice { label, value });
            }
        }
    }

    if chart.slices.is_empty() {
        return Err("No data found in pie chart".to_string());
    }

    Ok(chart)
}

/// Render a pie chart to the UI.
pub fn render_pie_chart(
    ui: &mut Ui,
    chart: &PieChart,
    dark_mode: bool,
    font_size: f32,
) {
    use std::f32::consts::PI;

    let margin = 20.0_f32;
    let pie_radius = 80.0_f32;
    let legend_width = 120.0_f32;
    
    let total_width = margin * 3.0 + pie_radius * 2.0 + legend_width;
    let total_height = margin * 2.0 + pie_radius * 2.0 + if chart.title.is_some() { 30.0 } else { 0.0 };

    let text_color = if dark_mode {
        Color32::from_rgb(220, 230, 240)
    } else {
        Color32::from_rgb(30, 40, 50)
    };

    // Pie colors
    let colors = [
        Color32::from_rgb(66, 133, 244),   // Blue
        Color32::from_rgb(234, 67, 53),    // Red
        Color32::from_rgb(251, 188, 4),    // Yellow
        Color32::from_rgb(52, 168, 83),    // Green
        Color32::from_rgb(155, 89, 182),   // Purple
        Color32::from_rgb(230, 126, 34),   // Orange
        Color32::from_rgb(26, 188, 156),   // Teal
        Color32::from_rgb(241, 196, 15),   // Gold
    ];

    let (response, painter) = ui.allocate_painter(
        Vec2::new(total_width, total_height),
        egui::Sense::hover(),
    );
    let offset = response.rect.min.to_vec2();

    // Draw title
    let mut y_offset = margin;
    if let Some(title) = &chart.title {
        painter.text(
            Pos2::new(total_width / 2.0, margin / 2.0) + offset,
            egui::Align2::CENTER_CENTER,
            title,
            FontId::proportional(font_size + 2.0),
            text_color,
        );
        y_offset += 20.0;
    }

    // Calculate total and draw pie
    let total: f32 = chart.slices.iter().map(|s| s.value).sum();
    if total <= 0.0 {
        return;
    }

    let center = Pos2::new(margin + pie_radius, y_offset + pie_radius) + offset;
    let mut start_angle = -PI / 2.0; // Start from top
    let border_color = if dark_mode { Color32::from_rgb(25, 30, 40) } else { Color32::WHITE };

    // Draw each slice as a filled path
    for (i, slice) in chart.slices.iter().enumerate() {
        let sweep_angle = (slice.value / total) * 2.0 * PI;
        let color = colors[i % colors.len()];
        let end_angle = start_angle + sweep_angle;

        // Build path for this slice
        let mut path = vec![center];
        
        // Add arc points - use enough segments for smooth curve
        let arc_segments = ((sweep_angle / (2.0 * PI)) * 100.0).max(8.0) as usize;
        for j in 0..=arc_segments {
            let t = j as f32 / arc_segments as f32;
            let angle = start_angle + sweep_angle * t;
            path.push(center + Vec2::new(angle.cos(), angle.sin()) * pie_radius);
        }

        // Draw the slice as a filled mesh (handles non-convex shapes)
        // We'll use triangles from center to each pair of adjacent arc points
        for j in 0..path.len() - 2 {
            let p0 = path[0]; // center
            let p1 = path[j + 1];
            let p2 = path[j + 2];
            
            // Create a mesh for this triangle
            let mesh = egui::Mesh {
                indices: vec![0, 1, 2],
                vertices: vec![
                    egui::epaint::Vertex { pos: p0, uv: egui::epaint::WHITE_UV, color },
                    egui::epaint::Vertex { pos: p1, uv: egui::epaint::WHITE_UV, color },
                    egui::epaint::Vertex { pos: p2, uv: egui::epaint::WHITE_UV, color },
                ],
                texture_id: egui::TextureId::default(),
            };
            painter.add(egui::Shape::mesh(mesh));
        }

        // Draw slice border lines
        let start_edge = center + Vec2::new(start_angle.cos(), start_angle.sin()) * pie_radius;
        let end_edge = center + Vec2::new(end_angle.cos(), end_angle.sin()) * pie_radius;
        painter.line_segment([center, start_edge], Stroke::new(1.5, border_color));
        painter.line_segment([center, end_edge], Stroke::new(1.5, border_color));

        start_angle = end_angle;
    }
    
    // Draw outer circle border
    painter.circle_stroke(center, pie_radius, Stroke::new(1.5, border_color));

    // Draw legend
    let legend_x = margin * 2.0 + pie_radius * 2.0 + offset.x;
    let mut legend_y = y_offset + 10.0 + offset.y;

    for (i, slice) in chart.slices.iter().enumerate() {
        let color = colors[i % colors.len()];
        let percentage = (slice.value / total * 100.0).round();

        // Color box
        painter.rect_filled(
            Rect::from_min_size(Pos2::new(legend_x, legend_y), Vec2::new(12.0, 12.0)),
            2.0,
            color,
        );

        // Label
        painter.text(
            Pos2::new(legend_x + 18.0, legend_y + 6.0),
            egui::Align2::LEFT_CENTER,
            format!("{} ({}%)", slice.label, percentage),
            FontId::proportional(font_size - 2.0),
            text_color,
        );

        legend_y += 20.0;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// State Diagram Types and Renderer
// ─────────────────────────────────────────────────────────────────────────────

/// A state in a state diagram, supporting composite/nested states.
#[derive(Debug, Clone)]
pub struct State {
    pub id: String,
    pub label: String,
    pub is_start: bool,
    pub is_end: bool,
    /// Child states for composite states (empty for simple states)
    pub children: Vec<State>,
    /// Internal transitions within this composite state
    pub internal_transitions: Vec<Transition>,
    /// Parent state ID if this is a nested state
    pub parent: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            is_start: false,
            is_end: false,
            children: Vec::new(),
            internal_transitions: Vec::new(),
            parent: None,
        }
    }
}

impl State {
    /// Create a new simple state
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        let id = id.into();
        let label = label.into();
        Self {
            id,
            label,
            ..Default::default()
        }
    }

    /// Create a start state
    pub fn start() -> Self {
        Self {
            id: "__start__".to_string(),
            label: "●".to_string(),
            is_start: true,
            ..Default::default()
        }
    }

    /// Create an end state
    pub fn end() -> Self {
        Self {
            id: "__end__".to_string(),
            label: "◉".to_string(),
            is_end: true,
            ..Default::default()
        }
    }

    /// Check if this is a composite (has children)
    pub fn is_composite(&self) -> bool {
        !self.children.is_empty()
    }

    /// Get all nested states recursively (including self)
    pub fn all_states(&self) -> Vec<&State> {
        let mut result = vec![self];
        for child in &self.children {
            result.extend(child.all_states());
        }
        result
    }

    /// Find a state by ID within this state's subtree
    pub fn find_state(&self, id: &str) -> Option<&State> {
        if self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_state(id) {
                return Some(found);
            }
        }
        None
    }

    /// Find a mutable state by ID within this state's subtree
    pub fn find_state_mut(&mut self, id: &str) -> Option<&mut State> {
        if self.id == id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_state_mut(id) {
                return Some(found);
            }
        }
        None
    }
}

/// The kind of transition based on hierarchy relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionKind {
    /// Both source and target are in the same scope (same composite or both top-level)
    Internal,
    /// Transition enters a composite state from outside
    Enter,
    /// Transition exits a composite state to outside
    Exit,
    /// Transition crosses between different branches of the hierarchy
    CrossHierarchy,
}

impl Default for TransitionKind {
    fn default() -> Self {
        Self::Internal
    }
}

/// A transition between states.
#[derive(Debug, Clone)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    /// The kind of transition (computed during normalization)
    pub kind: TransitionKind,
    /// Source state's enclosing composite (None if top-level)
    pub source_scope: Option<String>,
    /// Target state's enclosing composite (None if top-level)
    pub target_scope: Option<String>,
    /// Lowest common ancestor composite for source and target (None if both top-level)
    pub lca_scope: Option<String>,
}

impl Transition {
    /// Create a new transition with default kind (will be computed later)
    pub fn new(from: String, to: String, label: Option<String>) -> Self {
        Self {
            from,
            to,
            label,
            kind: TransitionKind::Internal,
            source_scope: None,
            target_scope: None,
            lca_scope: None,
        }
    }
}

/// Configuration options for state diagram layout.
#[derive(Debug, Clone)]
pub struct StateDiagramConfig {
    /// Padding inside composite states
    pub composite_padding: f32,
    /// Height of the header/title bar in composite states
    pub header_height: f32,
    /// Horizontal spacing between child states
    pub child_spacing_x: f32,
    /// Vertical spacing between child states
    pub child_spacing_y: f32,
    /// Minimum width for simple states
    pub min_state_width: f32,
    /// Height for simple states
    pub state_height: f32,
    /// Horizontal spacing between top-level states
    pub spacing_x: f32,
    /// Vertical spacing between top-level states
    pub spacing_y: f32,
    /// Margin around the entire diagram
    pub margin: f32,
    /// Whether to use orthogonal routing for cross-hierarchy transitions
    pub orthogonal_cross_routing: bool,
    /// Preferred anchor side for cross-hierarchy exits (true = left/right, false = top/bottom)
    pub prefer_horizontal_anchors: bool,
}

impl Default for StateDiagramConfig {
    fn default() -> Self {
        Self {
            composite_padding: 20.0,
            header_height: 28.0,
            child_spacing_x: 80.0,
            child_spacing_y: 56.0,
            min_state_width: 80.0,
            state_height: 36.0,
            spacing_x: 100.0,
            spacing_y: 70.0,
            margin: 40.0,
            orthogonal_cross_routing: true,
            prefer_horizontal_anchors: true,
        }
    }
}

/// A parsed state diagram.
#[derive(Debug, Clone, Default)]
pub struct StateDiagram {
    /// Top-level states (may contain nested children)
    pub states: Vec<State>,
    /// Top-level transitions (between top-level states or across hierarchy)
    pub transitions: Vec<Transition>,
    /// Layout configuration
    pub config: StateDiagramConfig,
}

/// Parse mermaid state diagram source.
pub fn parse_state_diagram(source: &str) -> Result<StateDiagram, String> {
    let lines: Vec<&str> = source.lines().skip(1).collect();
    let mut diagram = StateDiagram::default();
    let mut idx = 0;

    parse_state_diagram_block(&lines, &mut idx, &mut diagram.states, &mut diagram.transitions, None)?;

    if diagram.states.is_empty() {
        return Err("No states found in state diagram".to_string());
    }

    // Normalize transitions: compute kinds, scopes, and LCA
    normalize_state_diagram(&mut diagram);

    Ok(diagram)
}

/// Normalize a state diagram by computing transition metadata.
fn normalize_state_diagram(diagram: &mut StateDiagram) {
    // Build a map of state ID to its ancestry chain (list of parent IDs from root to immediate parent)
    let mut ancestry_map: HashMap<String, Vec<String>> = HashMap::new();
    build_ancestry_map(&diagram.states, &[], &mut ancestry_map);

    // Normalize top-level transitions
    for transition in &mut diagram.transitions {
        normalize_transition(transition, &ancestry_map);
    }

    // Normalize internal transitions recursively
    normalize_internal_transitions(&mut diagram.states, &ancestry_map);
}

/// Build ancestry map for all states recursively.
fn build_ancestry_map(
    states: &[State],
    current_ancestry: &[String],
    ancestry_map: &mut HashMap<String, Vec<String>>,
) {
    for state in states {
        ancestry_map.insert(state.id.clone(), current_ancestry.to_vec());
        
        if state.is_composite() {
            // Build ancestry for children - add current state to ancestry chain
            let mut child_ancestry = current_ancestry.to_vec();
            child_ancestry.push(state.id.clone());
            build_ancestry_map(&state.children, &child_ancestry, ancestry_map);
        }
    }
}

/// Normalize internal transitions for all composite states recursively.
fn normalize_internal_transitions(
    states: &mut [State],
    ancestry_map: &HashMap<String, Vec<String>>,
) {
    for state in states {
        for transition in &mut state.internal_transitions {
            normalize_transition(transition, ancestry_map);
        }
        normalize_internal_transitions(&mut state.children, ancestry_map);
    }
}

/// Normalize a single transition by computing its kind and scope metadata.
fn normalize_transition(
    transition: &mut Transition,
    ancestry_map: &HashMap<String, Vec<String>>,
) {
    let source_ancestry = ancestry_map.get(&transition.from).cloned().unwrap_or_default();
    let target_ancestry = ancestry_map.get(&transition.to).cloned().unwrap_or_default();

    // Set source and target scopes (immediate parent)
    transition.source_scope = source_ancestry.last().cloned();
    transition.target_scope = target_ancestry.last().cloned();

    // Compute LCA (lowest common ancestor)
    transition.lca_scope = find_lca(&source_ancestry, &target_ancestry);

    // Determine transition kind
    transition.kind = if transition.source_scope == transition.target_scope {
        // Same scope - either both top-level or both in same composite
        TransitionKind::Internal
    } else if source_ancestry.is_empty() && !target_ancestry.is_empty() {
        // Source is top-level, target is inside a composite
        TransitionKind::Enter
    } else if !source_ancestry.is_empty() && target_ancestry.is_empty() {
        // Source is inside a composite, target is top-level
        TransitionKind::Exit
    } else if is_ancestor_of(&source_ancestry, &transition.to) {
        // Source is ancestor of target - entering deeper
        TransitionKind::Enter
    } else if is_ancestor_of(&target_ancestry, &transition.from) {
        // Target is ancestor of source - exiting
        TransitionKind::Exit
    } else {
        // Different branches of hierarchy
        TransitionKind::CrossHierarchy
    };
}

/// Find the lowest common ancestor of two ancestry chains.
fn find_lca(ancestry_a: &[String], ancestry_b: &[String]) -> Option<String> {
    let mut lca = None;
    for (a, b) in ancestry_a.iter().zip(ancestry_b.iter()) {
        if a == b {
            lca = Some(a.clone());
        } else {
            break;
        }
    }
    lca
}

/// Check if the given ancestry chain contains a specific state ID.
fn is_ancestor_of(ancestry: &[String], state_id: &str) -> bool {
    ancestry.iter().any(|a| a == state_id)
}

/// Parse a block of state diagram content (recursive for nested states).
fn parse_state_diagram_block(
    lines: &[&str],
    idx: &mut usize,
    states: &mut Vec<State>,
    transitions: &mut Vec<Transition>,
    parent_id: Option<&str>,
) -> Result<(), String> {
    let mut state_map: HashMap<String, usize> = HashMap::new();

    // Build initial state map from existing states
    for (i, state) in states.iter().enumerate() {
        state_map.insert(state.id.clone(), i);
    }

    while *idx < lines.len() {
        let line = lines[*idx].trim();
        *idx += 1;

        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

        // End of composite block
        if line == "}" {
            return Ok(());
        }

        // Parse composite state: state StateName { or state "Label" as StateName {
        if line.starts_with("state ") && line.ends_with('{') {
            let rest = line[6..].trim_end_matches('{').trim();
            
            // Parse state name/label - handle both "state Name {" and "state "Label" as Name {"
            let (state_id, state_label) = if let Some(as_pos) = rest.find(" as ") {
                let label = rest[..as_pos].trim().trim_matches('"').to_string();
                let id = rest[as_pos + 4..].trim().to_string();
                (id, label)
            } else {
                let id = rest.trim_matches('"').to_string();
                (id.clone(), id)
            };

            // Create composite state
            let mut composite = State {
                id: state_id.clone(),
                label: state_label,
                parent: parent_id.map(|s| s.to_string()),
                ..Default::default()
            };

            // Recursively parse the composite block
            parse_state_diagram_block(
                lines,
                idx,
                &mut composite.children,
                &mut composite.internal_transitions,
                Some(&state_id),
            )?;

            // Set parent reference for all children
            for child in &mut composite.children {
                child.parent = Some(state_id.clone());
            }

            let state_idx = states.len();
            state_map.insert(state_id.clone(), state_idx);
            states.push(composite);
            continue;
        }

        // Parse transitions: State1 --> State2 or State1 --> State2: label
        if line.contains("-->") {
            if let Some(arrow_pos) = line.find("-->") {
                let from_part = line[..arrow_pos].trim();
                let rest = &line[arrow_pos + 3..];
                
                // Check for label after colon
                let (to_part, label) = if let Some(colon_pos) = rest.find(':') {
                    (rest[..colon_pos].trim(), Some(rest[colon_pos + 1..].trim().to_string()))
                } else {
                    (rest.trim(), None)
                };

                // Handle [*] for start/end states
                let from_id = if from_part == "[*]" { "__start__".to_string() } else { from_part.to_string() };
                let to_id = if to_part == "[*]" { "__end__".to_string() } else { to_part.to_string() };

                // Add states if not exists
                for (id, is_start, is_end) in [
                    (&from_id, from_part == "[*]", false),
                    (&to_id, false, to_part == "[*]"),
                ] {
                    if !state_map.contains_key(id) {
                        let state_label = if is_start { "●".to_string() } 
                                   else if is_end { "◉".to_string() }
                                   else { id.clone() };
                        let state_idx = states.len();
                        state_map.insert(id.clone(), state_idx);
                        states.push(State {
                            id: id.clone(),
                            label: state_label,
                            is_start,
                            is_end,
                            parent: parent_id.map(|s| s.to_string()),
                            ..Default::default()
                        });
                    }
                }

                transitions.push(Transition::new(from_id, to_id, label));
            }
            continue;
        }

        // Parse simple state definition: state "Label" as StateName
        if line.starts_with("state ") {
            let rest = line[6..].trim();
            if let Some(as_pos) = rest.find(" as ") {
                let label = rest[..as_pos].trim().trim_matches('"').to_string();
                let id = rest[as_pos + 4..].trim().to_string();
                if !state_map.contains_key(&id) {
                    let state_idx = states.len();
                    state_map.insert(id.clone(), state_idx);
                    states.push(State {
                        id: id.clone(),
                        label,
                        parent: parent_id.map(|s| s.to_string()),
                        ..Default::default()
                    });
                }
            } else {
                // Simple state definition: state StateName
                let id = rest.trim_matches('"').to_string();
                if !state_map.contains_key(&id) {
                    let state_idx = states.len();
                    state_map.insert(id.clone(), state_idx);
                    states.push(State {
                        id: id.clone(),
                        label: id.clone(),
                        parent: parent_id.map(|s| s.to_string()),
                        ..Default::default()
                    });
                }
            }
        }
    }

    Ok(())
}

/// Layout information for a state.
#[derive(Debug, Clone)]
struct StateLayout {
    /// Center position of the state
    center: Pos2,
    /// Size of the state (width, height) - larger for composites
    size: Vec2,
    /// For composites: bounding rect including title bar
    bounds: Rect,
    /// Whether this is a composite state
    is_composite: bool,
}

/// Colors for state diagram rendering.
struct StateDiagramColors {
    state_fill: Color32,
    state_stroke: Color32,
    composite_fill: Color32,
    composite_title_bg: Color32,
    text_color: Color32,
    arrow_color: Color32,
    /// Color for cross-hierarchy transitions (visually distinct)
    cross_arrow_color: Color32,
    /// Color for enter/exit transitions
    boundary_arrow_color: Color32,
    start_color: Color32,
    label_bg: Color32,
}

impl StateDiagramColors {
    fn new(dark_mode: bool) -> Self {
        if dark_mode {
            Self {
                state_fill: Color32::from_rgb(45, 55, 72),
                state_stroke: Color32::from_rgb(100, 140, 180),
                composite_fill: Color32::from_rgba_unmultiplied(40, 50, 65, 200),
                composite_title_bg: Color32::from_rgb(55, 70, 90),
                text_color: Color32::from_rgb(220, 230, 240),
                arrow_color: Color32::from_rgb(120, 150, 180),
                cross_arrow_color: Color32::from_rgb(180, 120, 150), // Slightly pink for cross-hierarchy
                boundary_arrow_color: Color32::from_rgb(150, 180, 120), // Slightly green for boundary crossing
                start_color: Color32::from_rgb(80, 180, 120),
                label_bg: Color32::from_rgb(35, 40, 50),
            }
        } else {
            Self {
                state_fill: Color32::from_rgb(240, 245, 250),
                state_stroke: Color32::from_rgb(100, 140, 180),
                composite_fill: Color32::from_rgba_unmultiplied(235, 240, 250, 220),
                composite_title_bg: Color32::from_rgb(220, 230, 245),
                text_color: Color32::from_rgb(30, 40, 50),
                arrow_color: Color32::from_rgb(100, 130, 160),
                cross_arrow_color: Color32::from_rgb(160, 100, 130), // Slightly pink for cross-hierarchy
                boundary_arrow_color: Color32::from_rgb(100, 140, 100), // Slightly green for boundary crossing
                start_color: Color32::from_rgb(50, 150, 80),
                label_bg: Color32::from_rgb(255, 255, 255),
            }
        }
    }

    /// Get the appropriate arrow color for a transition kind.
    fn arrow_color_for_kind(&self, kind: TransitionKind) -> Color32 {
        match kind {
            TransitionKind::Internal => self.arrow_color,
            TransitionKind::Enter | TransitionKind::Exit => self.boundary_arrow_color,
            TransitionKind::CrossHierarchy => self.cross_arrow_color,
        }
    }
}

/// Render a state diagram to the UI.
pub fn render_state_diagram(
    ui: &mut Ui,
    diagram: &StateDiagram,
    dark_mode: bool,
    font_size: f32,
) {
    if diagram.states.is_empty() {
        return;
    }

    let colors = StateDiagramColors::new(dark_mode);
    let config = &diagram.config;
    let label_font_size = font_size - 2.0;
    let state_padding = Vec2::new(24.0, 12.0);
    let min_state_width = config.min_state_width;
    let state_height = config.state_height;
    let spacing_x = config.spacing_x;
    let spacing_y = config.spacing_y;
    let margin = config.margin;
    let composite_padding = config.composite_padding;
    let title_bar_height = config.header_height;

    // Collect all states recursively for measurement
    fn collect_all_states(states: &[State]) -> Vec<&State> {
        let mut result = Vec::new();
        for state in states {
            result.push(state);
            result.extend(collect_all_states(&state.children));
        }
        result
    }

    let all_states = collect_all_states(&diagram.states);

    // Measure state labels to determine proper widths
    let state_widths: HashMap<String, f32> = {
        let text_measurer = EguiTextMeasurer::new(ui);
        all_states.iter()
            .filter(|s| !s.is_start && !s.is_end)
            .map(|state| {
                let text_size = text_measurer.measure(&state.label, font_size);
                let width = (text_size.width * 1.15 + state_padding.x).max(min_state_width);
                (state.id.clone(), width)
            })
            .collect()
    };

    // Collect all transitions (top-level and internal)
    fn collect_all_transitions<'a>(states: &'a [State], transitions: &'a [Transition]) -> Vec<&'a Transition> {
        let mut result: Vec<&'a Transition> = transitions.iter().collect();
        for state in states {
            result.extend(state.internal_transitions.iter());
            result.extend(collect_all_transitions(&state.children, &state.internal_transitions));
        }
        result
    }

    let all_transitions = collect_all_transitions(&diagram.states, &diagram.transitions);

    // Measure transition labels
    let transition_labels: HashMap<(String, String), (String, Vec2)> = {
        let text_measurer = EguiTextMeasurer::new(ui);
        all_transitions.iter()
            .filter_map(|trans| {
                trans.label.as_ref().map(|label| {
                    let text_size = text_measurer.measure(label, label_font_size);
                    let label_padding = Vec2::new(24.0, 10.0);
                    let size = Vec2::new(
                        text_size.width * 1.15 + label_padding.x,
                        text_size.height + label_padding.y,
                    );
                    ((trans.from.clone(), trans.to.clone()), (label.clone(), size))
                })
            })
            .collect()
    };

    // Layout states using recursive approach for nested states
    let mut state_layouts: HashMap<String, StateLayout> = HashMap::new();

    /// Recursive layout function for a set of states
    fn layout_states(
        states: &[State],
        transitions: &[Transition],
        state_widths: &HashMap<String, f32>,
        min_state_width: f32,
        state_height: f32,
        spacing_x: f32,
        spacing_y: f32,
        composite_padding: f32,
        title_bar_height: f32,
        start_pos: Pos2,
        layouts: &mut HashMap<String, StateLayout>,
    ) -> Vec2 {
        if states.is_empty() {
            return Vec2::ZERO;
        }

        // First, recursively layout children of composite states
        let mut composite_sizes: HashMap<String, Vec2> = HashMap::new();
        for state in states {
            if state.is_composite() {
                let child_size = layout_states(
                    &state.children,
                    &state.internal_transitions,
                    state_widths,
                    min_state_width,
                    state_height,
                    spacing_x * 0.8, // Tighter spacing for nested
                    spacing_y * 0.8,
                    composite_padding,
                    title_bar_height,
                    Pos2::ZERO, // Will be repositioned later
                    layouts,
                );
                // Composite size = child bounds + padding + title bar
                let comp_width = (child_size.x + composite_padding * 2.0).max(state_widths.get(&state.id).copied().unwrap_or(min_state_width) + composite_padding * 2.0);
                let comp_height = child_size.y + composite_padding * 2.0 + title_bar_height;
                composite_sizes.insert(state.id.clone(), Vec2::new(comp_width, comp_height));
            }
        }

        // Build adjacency for layer assignment at this level
        let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();
        let mut incoming: HashMap<String, Vec<String>> = HashMap::new();
        
        for state in states {
            outgoing.insert(state.id.clone(), Vec::new());
            incoming.insert(state.id.clone(), Vec::new());
        }
        
        for trans in transitions {
            // Only consider transitions between states at this level
            if states.iter().any(|s| s.id == trans.from) && states.iter().any(|s| s.id == trans.to) {
                if let Some(out) = outgoing.get_mut(&trans.from) {
                    out.push(trans.to.clone());
                }
                if let Some(inc) = incoming.get_mut(&trans.to) {
                    inc.push(trans.from.clone());
                }
            }
        }

        // Assign layers
        let mut layers: Vec<Vec<String>> = Vec::new();
        let mut state_layer: HashMap<String, usize> = HashMap::new();
        let mut remaining: Vec<String> = states.iter().map(|s| s.id.clone()).collect();

        while !remaining.is_empty() {
            let layer_states: Vec<String> = remaining
                .iter()
                .filter(|id| {
                    incoming.get(*id).map_or(true, |inc| {
                        inc.iter().all(|from| state_layer.contains_key(from))
                    })
                })
                .cloned()
                .collect();

            if layer_states.is_empty() {
                for id in remaining.drain(..) {
                    let idx = layers.len();
                    state_layer.insert(id.clone(), idx);
                    if layers.len() <= idx { layers.push(Vec::new()); }
                    layers[idx].push(id);
                }
                break;
            }

            let idx = layers.len();
            layers.push(layer_states.clone());
            for id in &layer_states {
                state_layer.insert(id.clone(), idx);
                remaining.retain(|r| r != id);
            }
        }

        // Calculate positions for this level
        let mut max_x = 0.0_f32;
        let mut max_y = 0.0_f32;

        for (layer_idx, layer) in layers.iter().enumerate() {
            // Calculate max width in this layer
            let layer_max_width = layer.iter()
                .map(|id| {
                    if let Some(comp_size) = composite_sizes.get(id) {
                        comp_size.x
                    } else {
                        state_widths.get(id).copied().unwrap_or(min_state_width)
                    }
                })
                .fold(min_state_width, f32::max);

            let x = start_pos.x + layer_idx as f32 * (layer_max_width + spacing_x) + layer_max_width / 2.0;
            
            // Calculate cumulative Y for states in this layer
            let mut current_y = start_pos.y;
            
            for id in layer {
                let state = states.iter().find(|s| s.id == *id).unwrap();
                let (width, height) = if let Some(comp_size) = composite_sizes.get(id) {
                    (comp_size.x, comp_size.y)
                } else if state.is_start || state.is_end {
                    (24.0, 24.0)
                } else {
                    (state_widths.get(id).copied().unwrap_or(min_state_width), state_height)
                };

                let center_y = current_y + height / 2.0;
                let center = Pos2::new(x, center_y);
                
                layouts.insert(id.clone(), StateLayout {
                    center,
                    size: Vec2::new(width, height),
                    bounds: Rect::from_center_size(center, Vec2::new(width, height)),
                    is_composite: state.is_composite(),
                });

                // Reposition children inside composite
                if state.is_composite() {
                    let child_offset = Vec2::new(
                        center.x - width / 2.0 + composite_padding,
                        center.y - height / 2.0 + title_bar_height + composite_padding,
                    );
                    reposition_children(&state.children, child_offset, layouts);
                }

                current_y += height + spacing_y;
                max_x = max_x.max(x + width / 2.0);
                max_y = max_y.max(current_y - spacing_y);
            }
        }

        Vec2::new(max_x - start_pos.x, max_y - start_pos.y)
    }

    /// Reposition children by an offset
    fn reposition_children(
        children: &[State],
        offset: Vec2,
        layouts: &mut HashMap<String, StateLayout>,
    ) {
        for child in children {
            if let Some(layout) = layouts.get_mut(&child.id) {
                layout.center = layout.center + offset;
                layout.bounds = Rect::from_center_size(layout.center, layout.size);
            }
            reposition_children(&child.children, offset, layouts);
        }
    }

    // Perform layout starting from margin
    let total_size = layout_states(
        &diagram.states,
        &diagram.transitions,
        &state_widths,
        min_state_width,
        state_height,
        spacing_x,
        spacing_y,
        composite_padding,
        title_bar_height,
        Pos2::new(margin, margin),
        &mut state_layouts,
    );

    let total_width = (total_size.x + margin * 2.0).max(300.0);
    let total_height = (total_size.y + margin * 2.0).max(100.0);

    let (response, painter) = ui.allocate_painter(
        Vec2::new(total_width, total_height),
        egui::Sense::hover(),
    );
    let offset = response.rect.min.to_vec2();

    // Draw function for states (recursive, draws composites as containers)
    fn draw_states(
        states: &[State],
        layouts: &HashMap<String, StateLayout>,
        painter: &egui::Painter,
        offset: Vec2,
        colors: &StateDiagramColors,
        font_size: f32,
        title_bar_height: f32,
        dark_mode: bool,
    ) {
        for state in states {
            if let Some(layout) = layouts.get(&state.id) {
                let center = layout.center + offset;
                let bounds = layout.bounds.translate(offset);

                if state.is_start || state.is_end {
                    // Draw start/end circles
                    let radius = if state.is_start { 10.0 } else { 14.0 };
                    painter.circle_filled(center, radius, if state.is_start { colors.start_color } else { colors.state_stroke });
                    if state.is_end {
                        painter.circle_filled(center, 7.0, if dark_mode { Color32::from_rgb(30, 35, 45) } else { Color32::WHITE });
                    }
                } else if state.is_composite() {
                    // Draw composite state container
                    // Outer rounded rectangle
                    painter.rect(
                        bounds,
                        Rounding::same(10.0),
                        colors.composite_fill,
                        Stroke::new(2.0, colors.state_stroke),
                    );
                    
                    // Title bar at top
                    let title_rect = Rect::from_min_size(
                        bounds.min,
                        Vec2::new(bounds.width(), title_bar_height),
                    );
                    painter.rect(
                        title_rect,
                        Rounding { nw: 10.0, ne: 10.0, sw: 0.0, se: 0.0 },
                        colors.composite_title_bg,
                        Stroke::NONE,
                    );
                    
                    // Title text
                    let title_center = Pos2::new(title_rect.center().x, title_rect.center().y);
                    painter.text(
                        title_center,
                        egui::Align2::CENTER_CENTER,
                        &state.label,
                        FontId::proportional(font_size),
                        colors.text_color,
                    );
                    
                    // Separator line under title
                    painter.line_segment(
                        [
                            Pos2::new(bounds.min.x, bounds.min.y + title_bar_height),
                            Pos2::new(bounds.max.x, bounds.min.y + title_bar_height),
                        ],
                        Stroke::new(1.0, colors.state_stroke),
                    );

                    // Recursively draw children
                    draw_states(&state.children, layouts, painter, offset, colors, font_size, title_bar_height, dark_mode);
                } else {
                    // Draw simple state
                    painter.rect(bounds, Rounding::same(8.0), colors.state_fill, Stroke::new(2.0, colors.state_stroke));
                    painter.text(center, egui::Align2::CENTER_CENTER, &state.label, FontId::proportional(font_size), colors.text_color);
                }
            }
        }
    }

    /// Compute the best anchor point on a state boundary for connecting to a target.
    fn compute_anchor_point(
        layout: &StateLayout,
        target: Pos2,
        offset: Vec2,
        is_composite: bool,
        header_height: f32,
    ) -> Pos2 {
        let center = layout.center + offset;
        let bounds = layout.bounds.translate(offset);
        
        // For start/end states (small circles), use simple radial anchor
        if layout.size.x == layout.size.y && layout.size.x < 30.0 {
            let dir = (target - center).normalized();
            return center + dir * 12.0;
        }

        // Determine which side of the bounds the target is on
        let dx = target.x - center.x;
        let dy = target.y - center.y;
        let abs_dx = dx.abs();
        let abs_dy = dy.abs();
        
        // Aspect ratio consideration
        let half_w = layout.size.x / 2.0;
        let half_h = layout.size.y / 2.0;
        
        // Determine dominant direction
        if abs_dx / half_w > abs_dy / half_h {
            // Horizontal - use left or right edge
            if dx > 0.0 {
                // Target is to the right
                let y = center.y.max(bounds.min.y + if is_composite { header_height + 5.0 } else { 0.0 }).min(bounds.max.y - 5.0);
                Pos2::new(bounds.max.x, y.clamp(bounds.min.y + 5.0, bounds.max.y - 5.0))
            } else {
                // Target is to the left
                let y = center.y.max(bounds.min.y + if is_composite { header_height + 5.0 } else { 0.0 }).min(bounds.max.y - 5.0);
                Pos2::new(bounds.min.x, y.clamp(bounds.min.y + 5.0, bounds.max.y - 5.0))
            }
        } else {
            // Vertical - use top or bottom edge
            if dy > 0.0 {
                // Target is below
                Pos2::new(center.x.clamp(bounds.min.x + 5.0, bounds.max.x - 5.0), bounds.max.y)
            } else {
                // Target is above - avoid header for composites
                let min_y = if is_composite { bounds.min.y + header_height } else { bounds.min.y };
                Pos2::new(center.x.clamp(bounds.min.x + 5.0, bounds.max.x - 5.0), min_y)
            }
        }
    }

    // Draw function for transitions with support for different kinds
    fn draw_transitions(
        transitions: &[Transition],
        layouts: &HashMap<String, StateLayout>,
        transition_labels: &HashMap<(String, String), (String, Vec2)>,
        painter: &egui::Painter,
        offset: Vec2,
        colors: &StateDiagramColors,
        font_size: f32,
        config: &StateDiagramConfig,
    ) {
        for trans in transitions {
            if let (Some(from_layout), Some(to_layout)) = (layouts.get(&trans.from), layouts.get(&trans.to)) {
                // Get arrow color based on transition kind
                let arrow_color = colors.arrow_color_for_kind(trans.kind);
                
                // Compute anchor points
                let from_center = from_layout.center + offset;
                let to_center = to_layout.center + offset;
                
                let start = compute_anchor_point(
                    from_layout, to_center, offset, from_layout.is_composite, config.header_height
                );
                let end = compute_anchor_point(
                    to_layout, from_center, offset, to_layout.is_composite, config.header_height
                );
                
                // For cross-hierarchy transitions, optionally use orthogonal routing
                let use_orthogonal = config.orthogonal_cross_routing 
                    && trans.kind == TransitionKind::CrossHierarchy;
                
                if use_orthogonal && (start.x - end.x).abs() > 20.0 && (start.y - end.y).abs() > 20.0 {
                    // Draw orthogonal path with elbow
                    let mid_x = (start.x + end.x) / 2.0;
                    let elbow1 = Pos2::new(mid_x, start.y);
                    let elbow2 = Pos2::new(mid_x, end.y);
                    
                    painter.line_segment([start, elbow1], Stroke::new(1.5, arrow_color));
                    painter.line_segment([elbow1, elbow2], Stroke::new(1.5, arrow_color));
                    painter.line_segment([elbow2, end], Stroke::new(1.5, arrow_color));
                    
                    // Arrow head at end
                    let final_dir = (end - elbow2).normalized();
                    let arrow_size = 8.0;
                    let perp = Vec2::new(-final_dir.y, final_dir.x);
                    let arrow_left = end - final_dir * arrow_size + perp * (arrow_size * 0.4);
                    let arrow_right = end - final_dir * arrow_size - perp * (arrow_size * 0.4);
                    painter.add(egui::Shape::convex_polygon(
                        vec![end, arrow_left, arrow_right],
                        arrow_color,
                        Stroke::NONE,
                    ));
                    
                    // Label at elbow midpoint
                    if let Some((label_text, label_size)) = transition_labels.get(&(trans.from.clone(), trans.to.clone())) {
                        let label_pos = Pos2::new(mid_x, (elbow1.y + elbow2.y) / 2.0);
                        let label_rect = Rect::from_center_size(label_pos, *label_size);
                        painter.rect_filled(label_rect, 3.0, colors.label_bg);
                        painter.text(label_pos, egui::Align2::CENTER_CENTER, label_text, FontId::proportional(font_size - 2.0), colors.text_color);
                    }
                } else {
                    // Draw straight line (default)
                    let dir = (end - start).normalized();
                    
                    painter.line_segment([start, end], Stroke::new(1.5, arrow_color));
                    
                    // Draw arrow head
                    let arrow_size = 8.0;
                    let perp = Vec2::new(-dir.y, dir.x);
                    let arrow_left = end - dir * arrow_size + perp * (arrow_size * 0.4);
                    let arrow_right = end - dir * arrow_size - perp * (arrow_size * 0.4);
                    painter.add(egui::Shape::convex_polygon(
                        vec![end, arrow_left, arrow_right],
                        arrow_color,
                        Stroke::NONE,
                    ));
                    
                    // Draw label with background
                    if let Some((label_text, label_size)) = transition_labels.get(&(trans.from.clone(), trans.to.clone())) {
                        let mid = Pos2::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
                        let label_pos = mid - Vec2::new(0.0, 14.0);
                        let label_rect = Rect::from_center_size(label_pos, *label_size);
                        painter.rect_filled(label_rect, 3.0, colors.label_bg);
                        painter.text(label_pos, egui::Align2::CENTER_CENTER, label_text, FontId::proportional(font_size - 2.0), colors.text_color);
                    }
                }
            }
        }
    }

    // Draw function for internal transitions (recursive)
    fn draw_all_internal_transitions(
        states: &[State],
        layouts: &HashMap<String, StateLayout>,
        transition_labels: &HashMap<(String, String), (String, Vec2)>,
        painter: &egui::Painter,
        offset: Vec2,
        colors: &StateDiagramColors,
        font_size: f32,
        config: &StateDiagramConfig,
    ) {
        for state in states {
            // Draw internal transitions
            draw_transitions(&state.internal_transitions, layouts, transition_labels, painter, offset, colors, font_size, config);
            // Recurse into children
            draw_all_internal_transitions(&state.children, layouts, transition_labels, painter, offset, colors, font_size, config);
        }
    }

    // Draw everything: states first, then transitions on top
    draw_states(&diagram.states, &state_layouts, &painter, offset, &colors, font_size, title_bar_height, dark_mode);
    draw_transitions(&diagram.transitions, &state_layouts, &transition_labels, &painter, offset, &colors, label_font_size, config);
    draw_all_internal_transitions(&diagram.states, &state_layouts, &transition_labels, &painter, offset, &colors, label_font_size, config);
}

// ─────────────────────────────────────────────────────────────────────────────
// Mindmap Types and Renderer
// ─────────────────────────────────────────────────────────────────────────────

/// A node in a mindmap.
#[derive(Debug, Clone)]
pub struct MindmapNode {
    pub text: String,
    pub level: usize,
    pub children: Vec<MindmapNode>,
}

/// A parsed mindmap.
#[derive(Debug, Clone)]
pub struct Mindmap {
    pub root: Option<MindmapNode>,
}

/// Parse mermaid mindmap source.
pub fn parse_mindmap(source: &str) -> Result<Mindmap, String> {
    let mut root: Option<MindmapNode> = None;
    let mut stack: Vec<(usize, MindmapNode)> = Vec::new();

    for line in source.lines().skip(1) {
        if line.trim().is_empty() || line.trim().starts_with("%%") {
            continue;
        }

        // Count indentation (spaces)
        let indent = line.chars().take_while(|c| c.is_whitespace()).count();
        let level = indent / 4; // Assume 4 spaces per level
        let text = line.trim();
        
        // Handle root node with (( )) or just text
        let text = if text.starts_with("root") {
            let inner = text.strip_prefix("root").unwrap_or(text).trim();
            if inner.starts_with("((") && inner.ends_with("))") {
                inner[2..inner.len()-2].to_string()
            } else if inner.starts_with('(') && inner.ends_with(')') {
                inner[1..inner.len()-1].to_string()
            } else {
                inner.to_string()
            }
        } else {
            text.to_string()
        };

        if text.is_empty() {
            continue;
        }

        let node = MindmapNode { text, level, children: Vec::new() };

        // Find parent
        while let Some((parent_level, _)) = stack.last() {
            if *parent_level >= level {
                let (_, finished_node) = stack.pop().unwrap();
                if let Some((_, parent)) = stack.last_mut() {
                    parent.children.push(finished_node);
                } else {
                    root = Some(finished_node);
                }
            } else {
                break;
            }
        }

        stack.push((level, node));
    }

    // Pop remaining nodes
    while let Some((_, finished_node)) = stack.pop() {
        if let Some((_, parent)) = stack.last_mut() {
            parent.children.push(finished_node);
        } else {
            root = Some(finished_node);
        }
    }

    if root.is_none() {
        return Err("No root node found in mindmap".to_string());
    }

    Ok(Mindmap { root })
}

/// Layout info for a mindmap node.
#[derive(Debug, Clone)]
struct MindmapLayout {
    center: Pos2,
    width: f32,
    children: Vec<MindmapLayout>,
}

/// Render a mindmap to the UI.
pub fn render_mindmap(
    ui: &mut Ui,
    mindmap: &Mindmap,
    dark_mode: bool,
    font_size: f32,
) {
    let root = match &mindmap.root {
        Some(r) => r,
        None => return,
    };

    let margin = 30.0_f32;
    let node_height = 28.0_f32;
    let level_width = 160.0_f32; // Increased for wider nodes
    let vertical_spacing = 12.0_f32;
    let node_padding = 24.0_f32;
    let min_node_width = 60.0_f32;
    let max_node_width = 180.0_f32; // Increased max width

    // Pre-measure all node text widths
    fn collect_texts(node: &MindmapNode, texts: &mut Vec<String>) {
        texts.push(node.text.clone());
        for child in &node.children {
            collect_texts(child, texts);
        }
    }
    let mut all_texts = Vec::new();
    collect_texts(root, &mut all_texts);

    let text_widths: HashMap<String, f32> = {
        let text_measurer = EguiTextMeasurer::new(ui);
        all_texts.into_iter()
            .map(|text| {
                let size = text_measurer.measure(&text, font_size);
                let width = (size.width * 1.15 + node_padding).max(min_node_width).min(max_node_width);
                (text, width)
            })
            .collect()
    };

    // First pass: calculate layout WITHOUT drawing
    fn calc_layout(
        node: &MindmapNode,
        x: f32,
        y: &mut f32,
        node_height: f32,
        level_width: f32,
        vertical_spacing: f32,
        text_widths: &HashMap<String, f32>,
        min_node_width: f32,
    ) -> MindmapLayout {
        let node_width = text_widths.get(&node.text).copied().unwrap_or(min_node_width);
        
        // First, layout all children
        let mut children_layouts: Vec<MindmapLayout> = Vec::new();
        for child in &node.children {
            let child_layout = calc_layout(child, x + level_width, y, node_height, level_width, vertical_spacing, text_widths, min_node_width);
            children_layouts.push(child_layout);
        }
        
        // Calculate this node's center Y
        let center_y = if children_layouts.is_empty() {
            let cy = *y + node_height / 2.0;
            *y += node_height + vertical_spacing;
            cy
        } else {
            // Center among children
            let first_y = children_layouts.first().map(|c| c.center.y).unwrap_or(*y);
            let last_y = children_layouts.last().map(|c| c.center.y).unwrap_or(*y);
            (first_y + last_y) / 2.0
        };
        
        MindmapLayout {
            center: Pos2::new(x + node_width / 2.0, center_y),
            width: node_width,
            children: children_layouts,
        }
    }

    // Calculate layout
    let mut y = margin;
    let layout = calc_layout(root, margin, &mut y, node_height, level_width, vertical_spacing, &text_widths, min_node_width);

    // Calculate total size from layout
    fn calc_bounds(layout: &MindmapLayout, node_height: f32) -> (f32, f32) {
        let mut max_x = layout.center.x + layout.width / 2.0;
        let mut max_y = layout.center.y + node_height / 2.0;
        for child in &layout.children {
            let (cx, cy) = calc_bounds(child, node_height);
            max_x = max_x.max(cx);
            max_y = max_y.max(cy);
        }
        (max_x, max_y)
    }
    let (max_x, max_y) = calc_bounds(&layout, node_height);
    let total_width = max_x + margin;
    let total_height = max_y + margin;

    // Colors
    let colors_by_level = if dark_mode {
        vec![
            Color32::from_rgb(100, 160, 220),
            Color32::from_rgb(120, 180, 140),
            Color32::from_rgb(200, 160, 100),
            Color32::from_rgb(180, 120, 160),
            Color32::from_rgb(140, 140, 180),
        ]
    } else {
        vec![
            Color32::from_rgb(50, 120, 180),
            Color32::from_rgb(60, 140, 80),
            Color32::from_rgb(180, 130, 50),
            Color32::from_rgb(150, 80, 130),
            Color32::from_rgb(100, 100, 150),
        ]
    };
    let text_color = if dark_mode { Color32::from_rgb(220, 230, 240) } else { Color32::from_rgb(30, 40, 50) };
    let bg_color = if dark_mode { Color32::from_rgb(40, 45, 55) } else { Color32::from_rgb(245, 248, 252) };

    // Allocate space based on calculated layout
    let (response, painter) = ui.allocate_painter(
        Vec2::new(total_width.max(300.0), total_height.max(100.0)),
        egui::Sense::hover(),
    );
    let offset = response.rect.min.to_vec2();

    // Second pass: draw using pre-calculated layout
    fn draw_node(
        painter: &egui::Painter,
        node: &MindmapNode,
        layout: &MindmapLayout,
        offset: Vec2,
        level: usize,
        font_size: f32,
        node_height: f32,
        colors: &[Color32],
        text_color: Color32,
        bg_color: Color32,
    ) {
        let color = colors[level % colors.len()];
        let center = layout.center + offset;
        let rect = Rect::from_center_size(center, Vec2::new(layout.width, node_height));
        
        // Draw connections to children first (behind nodes)
        for (child_node, child_layout) in node.children.iter().zip(layout.children.iter()) {
            let child_center = child_layout.center + offset;
            let start = Pos2::new(rect.right(), center.y);
            let end = Pos2::new(child_center.x - child_layout.width / 2.0, child_center.y);
            painter.line_segment([start, end], Stroke::new(1.5, colors[(level + 1) % colors.len()].gamma_multiply(0.6)));
            
            // Recursively draw children
            draw_node(painter, child_node, child_layout, offset, level + 1, font_size, node_height, colors, text_color, bg_color);
        }
        
        // Draw this node
        painter.rect(rect, Rounding::same(node_height / 2.0), bg_color, Stroke::new(2.0, color));
        painter.text(center, egui::Align2::CENTER_CENTER, &node.text, FontId::proportional(font_size - 1.0), text_color);
    }

    draw_node(&painter, root, &layout, offset, 0, font_size, node_height, &colors_by_level, text_color, bg_color);
}

// ─────────────────────────────────────────────────────────────────────────────
// Class Diagram Types and Renderer
// ─────────────────────────────────────────────────────────────────────────────

/// Visibility modifier for class members.
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,    // +
    Private,   // -
    Protected, // #
    Package,   // ~
}

/// A member of a class (attribute or method).
#[derive(Debug, Clone)]
pub struct ClassMember {
    pub visibility: Visibility,
    pub name: String,
    pub member_type: String, // return type or attribute type
    pub is_method: bool,
}

/// A class in the diagram.
#[derive(Debug, Clone)]
pub struct Class {
    pub id: String,
    pub name: String,
    pub stereotype: Option<String>, // <<interface>>, <<abstract>>, etc.
    pub attributes: Vec<ClassMember>,
    pub methods: Vec<ClassMember>,
}

/// Relationship type between classes.
#[derive(Debug, Clone, PartialEq)]
pub enum ClassRelationType {
    Inheritance,   // --|>
    Composition,   // *--
    Aggregation,   // o--
    Association,   // --
    Dependency,    // ..>
    Realization,   // ..|>
}

/// A relationship between classes.
#[derive(Debug, Clone)]
pub struct ClassRelation {
    pub from: String,
    pub to: String,
    pub relation_type: ClassRelationType,
    pub label: Option<String>,
    pub from_cardinality: Option<String>,
    pub to_cardinality: Option<String>,
}

/// A class diagram.
#[derive(Debug, Clone)]
pub struct ClassDiagram {
    pub classes: Vec<Class>,
    pub relations: Vec<ClassRelation>,
}

/// Parse a class diagram from source.
pub fn parse_class_diagram(source: &str) -> Result<ClassDiagram, String> {
    let mut classes: Vec<Class> = Vec::new();
    let mut relations: Vec<ClassRelation> = Vec::new();
    let mut current_class: Option<Class> = None;

    for line in source.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

        // Check for class definition start: class ClassName { or class ClassName
        if line.starts_with("class ") {
            // Save previous class if any
            if let Some(c) = current_class.take() {
                classes.push(c);
            }

            let rest = line[6..].trim();
            let (name, stereotype) = if rest.contains("~") {
                // class Animal~T~ for generics
                let parts: Vec<&str> = rest.splitn(2, '~').collect();
                (parts[0].trim().to_string(), None)
            } else if rest.contains("<<") && rest.contains(">>") {
                // class Interface <<interface>>
                let start = rest.find("<<").unwrap();
                let end = rest.find(">>").unwrap();
                let name = rest[..start].trim().trim_end_matches('{').trim().to_string();
                let stereo = rest[start+2..end].trim().to_string();
                (name, Some(stereo))
            } else {
                (rest.trim_end_matches('{').trim().to_string(), None)
            };

            current_class = Some(Class {
                id: name.clone(),
                name,
                stereotype,
                attributes: Vec::new(),
                methods: Vec::new(),
            });
            continue;
        }

        // Check for class definition end
        if line == "}" {
            if let Some(c) = current_class.take() {
                classes.push(c);
            }
            continue;
        }

        // Check for member definition inside class
        if current_class.is_some() && !line.contains("--") && !line.contains("..") {
            if let Some(member) = parse_class_member(line) {
                if let Some(ref mut c) = current_class {
                    if member.is_method {
                        c.methods.push(member);
                    } else {
                        c.attributes.push(member);
                    }
                }
            }
            continue;
        }

        // Check for relationship
        if let Some(relation) = parse_class_relation(line) {
            // Ensure classes exist
            for class_id in [&relation.from, &relation.to] {
                if !classes.iter().any(|c| &c.id == class_id) && 
                   current_class.as_ref().map(|c| &c.id != class_id).unwrap_or(true) {
                    classes.push(Class {
                        id: class_id.clone(),
                        name: class_id.clone(),
                        stereotype: None,
                        attributes: Vec::new(),
                        methods: Vec::new(),
                    });
                }
            }
            relations.push(relation);
        }
    }

    // Save last class
    if let Some(c) = current_class {
        classes.push(c);
    }

    if classes.is_empty() {
        return Err("No classes found in diagram".to_string());
    }

    Ok(ClassDiagram { classes, relations })
}

fn parse_class_member(line: &str) -> Option<ClassMember> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let (visibility, rest) = if line.starts_with('+') {
        (Visibility::Public, &line[1..])
    } else if line.starts_with('-') {
        (Visibility::Private, &line[1..])
    } else if line.starts_with('#') {
        (Visibility::Protected, &line[1..])
    } else if line.starts_with('~') {
        (Visibility::Package, &line[1..])
    } else {
        (Visibility::Public, line)
    };

    let is_method = rest.contains('(');
    
    // Parse type and name
    let (name, member_type) = if is_method {
        // method() ReturnType or method(): ReturnType
        let paren_idx = rest.find('(')?;
        let name_part = rest[..paren_idx].trim().to_string();
        let type_part = if rest.contains(':') {
            rest.rsplit(':').next().unwrap_or("void").trim().to_string()
        } else if rest.contains(')') {
            let after_paren = rest.rfind(')')?;
            rest[after_paren+1..].trim().to_string()
        } else {
            "void".to_string()
        };
        (name_part, type_part)
    } else {
        // attribute: Type or Type attribute
        if rest.contains(':') {
            let parts: Vec<&str> = rest.splitn(2, ':').collect();
            (parts[0].trim().to_string(), parts.get(1).map(|s| s.trim()).unwrap_or("").to_string())
        } else {
            (rest.trim().to_string(), String::new())
        }
    };

    Some(ClassMember {
        visibility,
        name,
        member_type,
        is_method,
    })
}

fn parse_class_relation(line: &str) -> Option<ClassRelation> {
    // Patterns: A --|> B, A *-- B, A o-- B, A -- B, A ..> B, A ..|> B
    // With optional labels: A "label" --|> "label2" B
    
    let relation_patterns = [
        ("--|>", ClassRelationType::Inheritance),
        ("<|--", ClassRelationType::Inheritance),
        ("..|>", ClassRelationType::Realization),
        ("<|..", ClassRelationType::Realization),
        ("*--", ClassRelationType::Composition),
        ("--*", ClassRelationType::Composition),
        ("o--", ClassRelationType::Aggregation),
        ("--o", ClassRelationType::Aggregation),
        ("..>", ClassRelationType::Dependency),
        ("<..", ClassRelationType::Dependency),
        ("--", ClassRelationType::Association),
        ("..", ClassRelationType::Dependency),
    ];

    for (pattern, rel_type) in &relation_patterns {
        if line.contains(pattern) {
            let parts: Vec<&str> = line.split(pattern).collect();
            if parts.len() >= 2 {
                let from = parts[0].trim().trim_matches('"').split_whitespace().last()?.to_string();
                let to = parts[1].trim().trim_matches('"').split_whitespace().next()?.to_string();
                
                // Extract label if present (between quotes after relation)
                let label = if parts[1].contains(':') {
                    Some(parts[1].split(':').last()?.trim().to_string())
                } else {
                    None
                };
                
                return Some(ClassRelation {
                    from,
                    to,
                    relation_type: rel_type.clone(),
                    label,
                    from_cardinality: None,
                    to_cardinality: None,
                });
            }
        }
    }
    None
}

/// Render a class diagram to the UI.
pub fn render_class_diagram(
    ui: &mut Ui,
    diagram: &ClassDiagram,
    dark_mode: bool,
    font_size: f32,
) {
    let margin = 30.0_f32;
    let class_min_width = 120.0_f32;
    let member_height = font_size + 4.0;
    let header_height = font_size + 10.0;
    let spacing = Vec2::new(60.0, 50.0);
    let member_font_size = font_size - 2.0;
    let text_width_factor = 1.15;
    let name_padding = 24.0;
    let member_padding = 24.0;

    // Pre-measure class names and member text
    let class_sizes: HashMap<String, Vec2> = {
        let text_measurer = EguiTextMeasurer::new(ui);
        diagram.classes.iter()
            .map(|class| {
                // Measure class name
                let name_size = text_measurer.measure(&class.name, font_size);
                let name_width = name_size.width * text_width_factor + name_padding;

                // Measure all members (attributes and methods)
                let max_member_width = class.attributes.iter()
                    .chain(class.methods.iter())
                    .map(|m| {
                        let member_text = format!("{}: {}", m.name, m.member_type);
                        let size = text_measurer.measure(&member_text, member_font_size);
                        size.width * text_width_factor + member_padding
                    })
                    .fold(0.0_f32, f32::max);

                let width = name_width.max(max_member_width).max(class_min_width);
                let height = header_height
                    + (class.attributes.len().max(1) as f32 * member_height)
                    + (class.methods.len().max(1) as f32 * member_height)
                    + 10.0;

                (class.id.clone(), Vec2::new(width, height))
            })
            .collect()
    };

    // Layout classes in grid
    let classes_per_row = 3.max((diagram.classes.len() as f32).sqrt().ceil() as usize);
    let mut class_pos: HashMap<String, Pos2> = HashMap::new();
    let mut max_x = 0.0_f32;
    let mut max_y = 0.0_f32;
    let mut row_height = 0.0_f32;
    let mut x = margin;
    let mut y = margin;

    for (i, class) in diagram.classes.iter().enumerate() {
        let size = class_sizes.get(&class.id).copied().unwrap_or(Vec2::new(class_min_width, 80.0));
        
        if i > 0 && i % classes_per_row == 0 {
            x = margin;
            y += row_height + spacing.y;
            row_height = 0.0;
        }
        
        class_pos.insert(class.id.clone(), Pos2::new(x, y));
        max_x = max_x.max(x + size.x);
        max_y = max_y.max(y + size.y);
        row_height = row_height.max(size.y);
        x += size.x + spacing.x;
    }

    let total_width = max_x + margin;
    let total_height = max_y + margin;

    // Colors
    let (class_fill, class_stroke, header_fill, text_color, line_color) = if dark_mode {
        (
            Color32::from_rgb(40, 48, 60),
            Color32::from_rgb(100, 140, 180),
            Color32::from_rgb(55, 70, 90),
            Color32::from_rgb(220, 230, 240),
            Color32::from_rgb(120, 150, 180),
        )
    } else {
        (
            Color32::from_rgb(255, 255, 255),
            Color32::from_rgb(100, 140, 180),
            Color32::from_rgb(230, 240, 250),
            Color32::from_rgb(30, 40, 50),
            Color32::from_rgb(100, 130, 160),
        )
    };

    let (response, painter) = ui.allocate_painter(
        Vec2::new(total_width.max(300.0), total_height.max(100.0)),
        egui::Sense::hover(),
    );
    let offset = response.rect.min.to_vec2();

    // Draw relations first
    for relation in &diagram.relations {
        if let (Some(&from_pos), Some(&to_pos)) = (class_pos.get(&relation.from), class_pos.get(&relation.to)) {
            let from_size = class_sizes.get(&relation.from).copied().unwrap_or(Vec2::new(100.0, 80.0));
            let to_size = class_sizes.get(&relation.to).copied().unwrap_or(Vec2::new(100.0, 80.0));
            
            let from_center = from_pos + from_size / 2.0 + offset;
            let to_center = to_pos + to_size / 2.0 + offset;
            
            let dir = (to_center - from_center).normalized();
            let start = from_center + dir * (from_size.x / 2.0).min(from_size.y / 2.0);
            let end = to_center - dir * (to_size.x / 2.0).min(to_size.y / 2.0);
            
            // Draw line based on type
            let is_dashed = matches!(relation.relation_type, ClassRelationType::Dependency | ClassRelationType::Realization);
            
            if is_dashed {
                draw_dashed_line(&painter, start, end, Stroke::new(1.5, line_color), 6.0, 4.0);
            } else {
                painter.line_segment([start, end], Stroke::new(1.5, line_color));
            }
            
            // Draw arrow/decoration at end
            let arrow_size = 10.0;
            let perp = Vec2::new(-dir.y, dir.x);
            
            match relation.relation_type {
                ClassRelationType::Inheritance | ClassRelationType::Realization => {
                    // Empty triangle
                    let tip = end;
                    let left = end - dir * arrow_size + perp * (arrow_size * 0.5);
                    let right = end - dir * arrow_size - perp * (arrow_size * 0.5);
                    painter.add(egui::Shape::convex_polygon(
                        vec![tip, left, right],
                        if dark_mode { Color32::from_rgb(40, 48, 60) } else { Color32::WHITE },
                        Stroke::new(1.5, line_color),
                    ));
                }
                ClassRelationType::Composition => {
                    // Filled diamond at start
                    let diamond_size = 8.0;
                    let d_center = start + dir * diamond_size;
                    painter.add(egui::Shape::convex_polygon(
                        vec![
                            start,
                            d_center + perp * (diamond_size * 0.5),
                            start + dir * diamond_size * 2.0,
                            d_center - perp * (diamond_size * 0.5),
                        ],
                        line_color,
                        Stroke::NONE,
                    ));
                }
                ClassRelationType::Aggregation => {
                    // Empty diamond at start
                    let diamond_size = 8.0;
                    let d_center = start + dir * diamond_size;
                    painter.add(egui::Shape::convex_polygon(
                        vec![
                            start,
                            d_center + perp * (diamond_size * 0.5),
                            start + dir * diamond_size * 2.0,
                            d_center - perp * (diamond_size * 0.5),
                        ],
                        if dark_mode { Color32::from_rgb(40, 48, 60) } else { Color32::WHITE },
                        Stroke::new(1.5, line_color),
                    ));
                }
                ClassRelationType::Dependency => {
                    // Simple arrow
                    let left = end - dir * arrow_size + perp * (arrow_size * 0.4);
                    let right = end - dir * arrow_size - perp * (arrow_size * 0.4);
                    painter.line_segment([left, end], Stroke::new(1.5, line_color));
                    painter.line_segment([right, end], Stroke::new(1.5, line_color));
                }
                ClassRelationType::Association => {
                    // Simple line, no decoration
                }
            }
            
            // Draw label
            if let Some(label) = &relation.label {
                let mid = Pos2::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
                painter.text(mid, egui::Align2::CENTER_CENTER, label, FontId::proportional(font_size - 2.0), text_color);
            }
        }
    }

    // Draw classes
    for class in &diagram.classes {
        if let Some(&pos) = class_pos.get(&class.id) {
            let size = class_sizes.get(&class.id).copied().unwrap_or(Vec2::new(100.0, 80.0));
            let rect = Rect::from_min_size(pos + offset, size);
            
            // Draw box
            painter.rect(rect, Rounding::same(4.0), class_fill, Stroke::new(2.0, class_stroke));
            
            // Draw header
            let header_rect = Rect::from_min_size(rect.min, Vec2::new(size.x, header_height));
            painter.rect_filled(header_rect, Rounding { nw: 4.0, ne: 4.0, sw: 0.0, se: 0.0 }, header_fill);
            
            // Draw stereotype
            let mut text_y = rect.min.y + 4.0;
            if let Some(stereo) = &class.stereotype {
                painter.text(
                    Pos2::new(rect.center().x, text_y + font_size * 0.35),
                    egui::Align2::CENTER_CENTER,
                    format!("<<{}>>", stereo),
                    FontId::proportional(font_size - 3.0),
                    text_color.gamma_multiply(0.7),
                );
                text_y += font_size * 0.7;
            }
            
            // Draw class name
            painter.text(
                Pos2::new(rect.center().x, text_y + font_size * 0.5),
                egui::Align2::CENTER_CENTER,
                &class.name,
                FontId::proportional(font_size),
                text_color,
            );
            
            // Draw separator after header
            let sep_y = rect.min.y + header_height;
            painter.line_segment(
                [Pos2::new(rect.min.x, sep_y), Pos2::new(rect.max.x, sep_y)],
                Stroke::new(1.0, class_stroke),
            );
            
            // Draw attributes
            let mut y = sep_y + 4.0;
            for attr in &class.attributes {
                let vis_char = match attr.visibility {
                    Visibility::Public => "+",
                    Visibility::Private => "-",
                    Visibility::Protected => "#",
                    Visibility::Package => "~",
                };
                let text = if attr.member_type.is_empty() {
                    format!("{} {}", vis_char, attr.name)
                } else {
                    format!("{} {}: {}", vis_char, attr.name, attr.member_type)
                };
                painter.text(
                    Pos2::new(rect.min.x + 6.0, y + member_height * 0.4),
                    egui::Align2::LEFT_CENTER,
                    text,
                    FontId::proportional(font_size - 2.0),
                    text_color,
                );
                y += member_height;
            }
            if class.attributes.is_empty() {
                y += member_height;
            }
            
            // Draw separator before methods
            painter.line_segment(
                [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                Stroke::new(1.0, class_stroke),
            );
            y += 4.0;
            
            // Draw methods
            for method in &class.methods {
                let vis_char = match method.visibility {
                    Visibility::Public => "+",
                    Visibility::Private => "-",
                    Visibility::Protected => "#",
                    Visibility::Package => "~",
                };
                let text = if method.member_type.is_empty() || method.member_type == "void" {
                    format!("{} {}()", vis_char, method.name)
                } else {
                    format!("{} {}(): {}", vis_char, method.name, method.member_type)
                };
                painter.text(
                    Pos2::new(rect.min.x + 6.0, y + member_height * 0.4),
                    egui::Align2::LEFT_CENTER,
                    text,
                    FontId::proportional(font_size - 2.0),
                    text_color,
                );
                y += member_height;
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Entity-Relationship Diagram Types and Renderer
// ─────────────────────────────────────────────────────────────────────────────

/// An entity in an ER diagram.
#[derive(Debug, Clone)]
pub struct Entity {
    pub name: String,
    pub attributes: Vec<EntityAttribute>,
}

/// An attribute of an entity.
#[derive(Debug, Clone)]
pub struct EntityAttribute {
    pub name: String,
    pub attr_type: String,
    pub is_pk: bool,  // Primary key
    pub is_fk: bool,  // Foreign key
}

/// Cardinality in a relationship.
#[derive(Debug, Clone, PartialEq)]
pub enum Cardinality {
    ZeroOrOne,   // |o or o|
    ExactlyOne,  // ||
    ZeroOrMore,  // }o or o{
    OneOrMore,   // }|  or |{
}

/// A relationship between entities.
#[derive(Debug, Clone)]
pub struct ERRelation {
    pub from: String,
    pub to: String,
    pub from_cardinality: Cardinality,
    pub to_cardinality: Cardinality,
    pub label: Option<String>,
    pub identifying: bool,  // solid vs dashed line
}

/// An ER diagram.
#[derive(Debug, Clone)]
pub struct ERDiagram {
    pub entities: Vec<Entity>,
    pub relations: Vec<ERRelation>,
}

/// Parse an ER diagram from source.
pub fn parse_er_diagram(source: &str) -> Result<ERDiagram, String> {
    let mut entities: Vec<Entity> = Vec::new();
    let mut relations: Vec<ERRelation> = Vec::new();
    let mut current_entity: Option<Entity> = None;

    for line in source.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

        // Check for entity definition
        if line.ends_with('{') {
            if let Some(e) = current_entity.take() {
                entities.push(e);
            }
            let name = line.trim_end_matches('{').trim().to_string();
            current_entity = Some(Entity {
                name,
                attributes: Vec::new(),
            });
            continue;
        }

        // Check for entity end
        if line == "}" {
            if let Some(e) = current_entity.take() {
                entities.push(e);
            }
            continue;
        }

        // Check for attribute inside entity
        if current_entity.is_some() && !line.contains("||") && !line.contains("}") && !line.contains("{") {
            if let Some(attr) = parse_er_attribute(line) {
                if let Some(ref mut e) = current_entity {
                    e.attributes.push(attr);
                }
            }
            continue;
        }

        // Check for relationship
        if let Some(relation) = parse_er_relation(line) {
            // Ensure entities exist
            for entity_name in [&relation.from, &relation.to] {
                if !entities.iter().any(|e| &e.name == entity_name) &&
                   current_entity.as_ref().map(|e| &e.name != entity_name).unwrap_or(true) {
                    entities.push(Entity {
                        name: entity_name.clone(),
                        attributes: Vec::new(),
                    });
                }
            }
            relations.push(relation);
        }
    }

    // Save last entity
    if let Some(e) = current_entity {
        entities.push(e);
    }

    if entities.is_empty() {
        return Err("No entities found in diagram".to_string());
    }

    Ok(ERDiagram { entities, relations })
}

fn parse_er_attribute(line: &str) -> Option<EntityAttribute> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // Format: type name PK/FK or type name "comment"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let attr_type = parts[0].to_string();
    let name = parts[1].to_string();
    let is_pk = parts.get(2).map(|s| s.to_uppercase() == "PK").unwrap_or(false);
    let is_fk = parts.get(2).map(|s| s.to_uppercase() == "FK").unwrap_or(false);

    Some(EntityAttribute {
        name,
        attr_type,
        is_pk,
        is_fk,
    })
}

fn parse_er_relation(line: &str) -> Option<ERRelation> {
    // Patterns: ENTITY1 ||--o{ ENTITY2 : "label"
    // Cardinality markers: || (exactly one), |o (zero or one), }| (one or more), }o (zero or more)
    
    let relation_patterns = [
        ("||--||", Cardinality::ExactlyOne, Cardinality::ExactlyOne, true),
        ("||--|{", Cardinality::ExactlyOne, Cardinality::OneOrMore, true),
        ("||--o{", Cardinality::ExactlyOne, Cardinality::ZeroOrMore, true),
        ("||--o|", Cardinality::ExactlyOne, Cardinality::ZeroOrOne, true),
        ("|o--||", Cardinality::ZeroOrOne, Cardinality::ExactlyOne, true),
        ("|o--|{", Cardinality::ZeroOrOne, Cardinality::OneOrMore, true),
        ("|o--o{", Cardinality::ZeroOrOne, Cardinality::ZeroOrMore, true),
        ("|o--o|", Cardinality::ZeroOrOne, Cardinality::ZeroOrOne, true),
        ("}|--||", Cardinality::OneOrMore, Cardinality::ExactlyOne, true),
        ("}|--|{", Cardinality::OneOrMore, Cardinality::OneOrMore, true),
        ("}|--o{", Cardinality::OneOrMore, Cardinality::ZeroOrMore, true),
        ("}|--o|", Cardinality::OneOrMore, Cardinality::ZeroOrOne, true),
        ("}o--||", Cardinality::ZeroOrMore, Cardinality::ExactlyOne, true),
        ("}o--|{", Cardinality::ZeroOrMore, Cardinality::OneOrMore, true),
        ("}o--o{", Cardinality::ZeroOrMore, Cardinality::ZeroOrMore, true),
        ("}o--o|", Cardinality::ZeroOrMore, Cardinality::ZeroOrOne, true),
        // Dashed variants (non-identifying)
        ("||..||", Cardinality::ExactlyOne, Cardinality::ExactlyOne, false),
        ("||..|{", Cardinality::ExactlyOne, Cardinality::OneOrMore, false),
        ("||..o{", Cardinality::ExactlyOne, Cardinality::ZeroOrMore, false),
        ("||..o|", Cardinality::ExactlyOne, Cardinality::ZeroOrOne, false),
    ];

    for (pattern, from_card, to_card, identifying) in &relation_patterns {
        if line.contains(pattern) {
            let parts: Vec<&str> = line.split(pattern).collect();
            if parts.len() >= 2 {
                let from = parts[0].trim().to_string();
                let rest = parts[1].trim();
                
                // Extract entity name and label
                let (to, label) = if rest.contains(':') {
                    let label_parts: Vec<&str> = rest.splitn(2, ':').collect();
                    (
                        label_parts[0].trim().to_string(),
                        Some(label_parts[1].trim().trim_matches('"').to_string()),
                    )
                } else {
                    (rest.to_string(), None)
                };

                return Some(ERRelation {
                    from,
                    to,
                    from_cardinality: from_card.clone(),
                    to_cardinality: to_card.clone(),
                    label,
                    identifying: *identifying,
                });
            }
        }
    }
    None
}

/// Render an ER diagram to the UI.
pub fn render_er_diagram(
    ui: &mut Ui,
    diagram: &ERDiagram,
    dark_mode: bool,
    font_size: f32,
) {
    let margin = 30.0_f32;
    let entity_min_width = 140.0_f32;
    let attr_height = font_size + 4.0;
    let header_height = font_size + 12.0;
    let spacing = Vec2::new(80.0, 60.0);
    let attr_font_size = font_size - 2.0;
    let text_width_factor = 1.15;
    let name_padding = 30.0;
    let attr_padding = 30.0;
    let label_font_size = font_size - 2.0;

    // Pre-measure entity sizes and relation labels
    let entity_sizes: HashMap<String, Vec2> = {
        let text_measurer = EguiTextMeasurer::new(ui);
        diagram.entities.iter()
            .map(|entity| {
                // Measure entity name
                let name_size = text_measurer.measure(&entity.name, font_size);
                let name_width = name_size.width * text_width_factor + name_padding;

                // Measure all attributes
                let max_attr_width = entity.attributes.iter()
                    .map(|a| {
                        let attr_text = format!("{} {}", a.attr_type, a.name);
                        let size = text_measurer.measure(&attr_text, attr_font_size);
                        size.width * text_width_factor + attr_padding
                    })
                    .fold(0.0_f32, f32::max);

                let width = name_width.max(max_attr_width).max(entity_min_width);
                let height = header_height + entity.attributes.len().max(1) as f32 * attr_height + 10.0;

                (entity.name.clone(), Vec2::new(width, height))
            })
            .collect()
    };

    // Pre-measure relation labels
    let relation_labels: HashMap<usize, (String, Vec2)> = {
        let text_measurer = EguiTextMeasurer::new(ui);
        diagram.relations.iter().enumerate()
            .filter_map(|(idx, rel)| {
                rel.label.as_ref().map(|label| {
                    let size = text_measurer.measure(label, label_font_size);
                    let label_size = Vec2::new(
                        size.width * text_width_factor + 16.0,
                        size.height + 8.0,
                    );
                    (idx, (label.clone(), label_size))
                })
            })
            .collect()
    };

    // Layout entities
    let entities_per_row = 3.max((diagram.entities.len() as f32).sqrt().ceil() as usize);
    let mut entity_pos: HashMap<String, Pos2> = HashMap::new();
    let mut max_x = 0.0_f32;
    let mut max_y = 0.0_f32;
    let mut row_height = 0.0_f32;
    let mut x = margin;
    let mut y = margin;

    for (i, entity) in diagram.entities.iter().enumerate() {
        let size = entity_sizes.get(&entity.name).copied().unwrap_or(Vec2::new(entity_min_width, 80.0));
        
        if i > 0 && i % entities_per_row == 0 {
            x = margin;
            y += row_height + spacing.y;
            row_height = 0.0;
        }
        
        entity_pos.insert(entity.name.clone(), Pos2::new(x, y));
        max_x = max_x.max(x + size.x);
        max_y = max_y.max(y + size.y);
        row_height = row_height.max(size.y);
        x += size.x + spacing.x;
    }

    let total_width = max_x + margin;
    let total_height = max_y + margin;

    // Colors
    let (entity_fill, entity_stroke, header_fill, text_color, line_color, pk_color) = if dark_mode {
        (
            Color32::from_rgb(40, 48, 60),
            Color32::from_rgb(100, 160, 140),
            Color32::from_rgb(50, 70, 65),
            Color32::from_rgb(220, 230, 240),
            Color32::from_rgb(100, 150, 130),
            Color32::from_rgb(220, 180, 80),
        )
    } else {
        (
            Color32::from_rgb(255, 255, 255),
            Color32::from_rgb(60, 140, 100),
            Color32::from_rgb(220, 245, 230),
            Color32::from_rgb(30, 40, 50),
            Color32::from_rgb(60, 120, 90),
            Color32::from_rgb(200, 150, 50),
        )
    };

    let (response, painter) = ui.allocate_painter(
        Vec2::new(total_width.max(300.0), total_height.max(100.0)),
        egui::Sense::hover(),
    );
    let offset = response.rect.min.to_vec2();

    // Draw relations first
    for (rel_idx, relation) in diagram.relations.iter().enumerate() {
        if let (Some(&from_pos), Some(&to_pos)) = (entity_pos.get(&relation.from), entity_pos.get(&relation.to)) {
            let from_size = entity_sizes.get(&relation.from).copied().unwrap_or(Vec2::new(100.0, 80.0));
            let to_size = entity_sizes.get(&relation.to).copied().unwrap_or(Vec2::new(100.0, 80.0));
            
            let from_center = from_pos + from_size / 2.0 + offset;
            let to_center = to_pos + to_size / 2.0 + offset;
            
            let dir = (to_center - from_center).normalized();
            let start = from_center + dir * (from_size.x / 2.0).min(from_size.y / 2.0);
            let end = to_center - dir * (to_size.x / 2.0).min(to_size.y / 2.0);
            
            // Draw line
            if relation.identifying {
                painter.line_segment([start, end], Stroke::new(1.5, line_color));
            } else {
                draw_dashed_line(&painter, start, end, Stroke::new(1.5, line_color), 6.0, 4.0);
            }
            
            // Draw cardinality markers
            let marker_size = 8.0;
            let perp = Vec2::new(-dir.y, dir.x);
            
            // From side marker
            draw_cardinality_marker(&painter, start, dir, perp, &relation.from_cardinality, marker_size, line_color);
            
            // To side marker
            draw_cardinality_marker(&painter, end, -dir, -perp, &relation.to_cardinality, marker_size, line_color);
            
            // Draw label using pre-measured size
            if let Some((label_text, label_size)) = relation_labels.get(&rel_idx) {
                let mid = Pos2::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
                let label_bg = if dark_mode { Color32::from_rgb(35, 40, 50) } else { Color32::from_rgb(255, 255, 255) };
                let label_rect = Rect::from_center_size(mid, *label_size);
                painter.rect_filled(label_rect, 2.0, label_bg);
                painter.text(mid, egui::Align2::CENTER_CENTER, label_text, FontId::proportional(label_font_size), text_color);
            }
        }
    }

    // Draw entities
    for entity in &diagram.entities {
        if let Some(&pos) = entity_pos.get(&entity.name) {
            let size = entity_sizes.get(&entity.name).copied().unwrap_or(Vec2::new(100.0, 80.0));
            let rect = Rect::from_min_size(pos + offset, size);
            
            // Draw box
            painter.rect(rect, Rounding::same(4.0), entity_fill, Stroke::new(2.0, entity_stroke));
            
            // Draw header
            let header_rect = Rect::from_min_size(rect.min, Vec2::new(size.x, header_height));
            painter.rect_filled(header_rect, Rounding { nw: 4.0, ne: 4.0, sw: 0.0, se: 0.0 }, header_fill);
            
            // Draw entity name
            painter.text(
                Pos2::new(rect.center().x, rect.min.y + header_height / 2.0),
                egui::Align2::CENTER_CENTER,
                &entity.name,
                FontId::proportional(font_size),
                text_color,
            );
            
            // Draw separator
            let sep_y = rect.min.y + header_height;
            painter.line_segment(
                [Pos2::new(rect.min.x, sep_y), Pos2::new(rect.max.x, sep_y)],
                Stroke::new(1.0, entity_stroke),
            );
            
            // Draw attributes
            let mut y = sep_y + 4.0;
            for attr in &entity.attributes {
                let color = if attr.is_pk { pk_color } else { text_color };
                let prefix = if attr.is_pk { "🔑 " } else if attr.is_fk { "🔗 " } else { "" };
                let text = format!("{}{} {}", prefix, attr.attr_type, attr.name);
                
                painter.text(
                    Pos2::new(rect.min.x + 8.0, y + attr_height * 0.4),
                    egui::Align2::LEFT_CENTER,
                    text,
                    FontId::proportional(font_size - 2.0),
                    color,
                );
                y += attr_height;
            }
        }
    }
}

fn draw_cardinality_marker(
    painter: &egui::Painter,
    pos: Pos2,
    dir: Vec2,
    perp: Vec2,
    cardinality: &Cardinality,
    size: f32,
    color: Color32,
) {
    match cardinality {
        Cardinality::ExactlyOne => {
            // Two vertical lines ||
            let p1a = pos + perp * size * 0.4;
            let p1b = pos - perp * size * 0.4;
            let p2a = pos + dir * 4.0 + perp * size * 0.4;
            let p2b = pos + dir * 4.0 - perp * size * 0.4;
            painter.line_segment([p1a, p1b], Stroke::new(2.0, color));
            painter.line_segment([p2a, p2b], Stroke::new(2.0, color));
        }
        Cardinality::ZeroOrOne => {
            // Circle and line |o
            let line_p1 = pos + perp * size * 0.4;
            let line_p2 = pos - perp * size * 0.4;
            painter.line_segment([line_p1, line_p2], Stroke::new(2.0, color));
            painter.circle_stroke(pos + dir * 8.0, 4.0, Stroke::new(2.0, color));
        }
        Cardinality::ZeroOrMore => {
            // Circle and crow's foot }o
            painter.circle_stroke(pos + dir * 4.0, 4.0, Stroke::new(2.0, color));
            let foot_start = pos + dir * 12.0;
            painter.line_segment([foot_start, pos + perp * size * 0.5], Stroke::new(1.5, color));
            painter.line_segment([foot_start, pos - perp * size * 0.5], Stroke::new(1.5, color));
            painter.line_segment([foot_start, pos], Stroke::new(1.5, color));
        }
        Cardinality::OneOrMore => {
            // Line and crow's foot }|
            let line_p1 = pos + perp * size * 0.4;
            let line_p2 = pos - perp * size * 0.4;
            painter.line_segment([line_p1, line_p2], Stroke::new(2.0, color));
            let foot_start = pos + dir * 8.0;
            painter.line_segment([foot_start, pos + perp * size * 0.5], Stroke::new(1.5, color));
            painter.line_segment([foot_start, pos - perp * size * 0.5], Stroke::new(1.5, color));
            painter.line_segment([foot_start, pos], Stroke::new(1.5, color));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Git Graph Types and Renderer
// ─────────────────────────────────────────────────────────────────────────────

/// A commit in a git graph.
#[derive(Debug, Clone)]
pub struct GitCommit {
    pub id: String,
    pub branch: String,
    pub message: Option<String>,
    pub is_merge: bool,
    pub merge_from: Option<String>,
}

/// A branch in a git graph.
#[derive(Debug, Clone)]
pub struct GitBranch {
    pub name: String,
    pub color_idx: usize,
}

/// A git graph.
#[derive(Debug, Clone)]
pub struct GitGraph {
    pub commits: Vec<GitCommit>,
    pub branches: Vec<GitBranch>,
}

/// Parse a git graph from source.
pub fn parse_git_graph(source: &str) -> Result<GitGraph, String> {
    let mut commits: Vec<GitCommit> = Vec::new();
    let mut branches: Vec<GitBranch> = vec![GitBranch { name: "main".to_string(), color_idx: 0 }];
    let mut current_branch = "main".to_string();
    let mut commit_counter = 0;

    for line in source.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

        let line_lower = line.to_lowercase();

        // Parse commit
        if line_lower.starts_with("commit") {
            commit_counter += 1;
            let id = if line.contains("id:") {
                // commit id: "abc123"
                line.split("id:")
                    .nth(1)
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    .unwrap_or_else(|| format!("c{}", commit_counter))
            } else {
                format!("c{}", commit_counter)
            };
            
            let message = if line.contains("msg:") {
                line.split("msg:")
                    .nth(1)
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            } else {
                None
            };

            commits.push(GitCommit {
                id,
                branch: current_branch.clone(),
                message,
                is_merge: false,
                merge_from: None,
            });
        }
        // Parse branch creation
        else if line_lower.starts_with("branch") {
            let name = line[6..].trim().to_string();
            if !branches.iter().any(|b| b.name == name) {
                branches.push(GitBranch {
                    name: name.clone(),
                    color_idx: branches.len(),
                });
            }
            current_branch = name;
        }
        // Parse checkout
        else if line_lower.starts_with("checkout") {
            let name = line[8..].trim().to_string();
            if !branches.iter().any(|b| b.name == name) {
                branches.push(GitBranch {
                    name: name.clone(),
                    color_idx: branches.len(),
                });
            }
            current_branch = name;
        }
        // Parse merge
        else if line_lower.starts_with("merge") {
            commit_counter += 1;
            let rest = line[5..].trim();
            let (merge_from, id) = if rest.contains("id:") {
                let parts: Vec<&str> = rest.split("id:").collect();
                let from = parts[0].trim().to_string();
                let id = parts[1].trim().trim_matches('"').trim_matches('\'').to_string();
                (from, id)
            } else {
                (rest.to_string(), format!("m{}", commit_counter))
            };

            commits.push(GitCommit {
                id,
                branch: current_branch.clone(),
                message: Some(format!("Merge {}", merge_from)),
                is_merge: true,
                merge_from: Some(merge_from),
            });
        }
    }

    if commits.is_empty() {
        return Err("No commits found in git graph".to_string());
    }

    Ok(GitGraph { commits, branches })
}

/// Render a git graph to the UI.
pub fn render_git_graph(
    ui: &mut Ui,
    graph: &GitGraph,
    dark_mode: bool,
    font_size: f32,
) {
    let margin = 30.0_f32;
    let commit_radius = 8.0_f32;
    let commit_spacing = 50.0_f32;
    let branch_spacing = 60.0_f32;  // Wider spacing between branches
    let label_width = 140.0_f32;    // Wider label area

    // Branch colors
    let branch_colors = if dark_mode {
        vec![
            Color32::from_rgb(100, 180, 100),  // green (main)
            Color32::from_rgb(100, 150, 220),  // blue
            Color32::from_rgb(220, 160, 100),  // orange
            Color32::from_rgb(180, 100, 180),  // purple
            Color32::from_rgb(220, 100, 100),  // red
            Color32::from_rgb(100, 200, 200),  // cyan
        ]
    } else {
        vec![
            Color32::from_rgb(60, 140, 60),
            Color32::from_rgb(60, 110, 180),
            Color32::from_rgb(180, 120, 60),
            Color32::from_rgb(140, 60, 140),
            Color32::from_rgb(180, 60, 60),
            Color32::from_rgb(60, 160, 160),
        ]
    };
    let text_color = if dark_mode { Color32::from_rgb(220, 230, 240) } else { Color32::from_rgb(30, 40, 50) };
    let line_bg = if dark_mode { Color32::from_rgb(50, 55, 65) } else { Color32::from_rgb(240, 245, 250) };

    // Calculate branch positions
    let mut branch_x: HashMap<String, f32> = HashMap::new();
    for (i, branch) in graph.branches.iter().enumerate() {
        branch_x.insert(branch.name.clone(), margin + label_width + i as f32 * branch_spacing);
    }

    let total_width = margin * 2.0 + label_width + graph.branches.len() as f32 * branch_spacing + 50.0;
    let total_height = margin * 2.0 + graph.commits.len() as f32 * commit_spacing;

    let (response, painter) = ui.allocate_painter(
        Vec2::new(total_width.max(300.0), total_height.max(100.0)),
        egui::Sense::hover(),
    );
    let offset = response.rect.min.to_vec2();

    // Track last commit position per branch for drawing lines
    let mut last_commit_pos: HashMap<String, Pos2> = HashMap::new();

    // Draw commits
    for (i, commit) in graph.commits.iter().enumerate() {
        let x = branch_x.get(&commit.branch).copied().unwrap_or(margin + label_width);
        let y = margin + i as f32 * commit_spacing;
        let pos = Pos2::new(x, y) + offset;

        let branch = graph.branches.iter().find(|b| b.name == commit.branch);
        let color = branch_colors[branch.map(|b| b.color_idx).unwrap_or(0) % branch_colors.len()];

        // Draw line from previous commit on same branch
        if let Some(prev_pos) = last_commit_pos.get(&commit.branch) {
            painter.line_segment([*prev_pos, pos], Stroke::new(3.0, color));
        }

        // Draw merge line
        if let Some(ref merge_from) = commit.merge_from {
            if let Some(merge_pos) = last_commit_pos.get(merge_from) {
                let merge_color = graph.branches.iter()
                    .find(|b| &b.name == merge_from)
                    .map(|b| branch_colors[b.color_idx % branch_colors.len()])
                    .unwrap_or(color);
                
                // Draw curved merge line
                let mid_y = (merge_pos.y + pos.y) / 2.0;
                let ctrl1 = Pos2::new(merge_pos.x, mid_y);
                let ctrl2 = Pos2::new(pos.x, mid_y);
                
                // Approximate bezier with line segments
                painter.line_segment([*merge_pos, ctrl1], Stroke::new(2.0, merge_color));
                painter.line_segment([ctrl1, ctrl2], Stroke::new(2.0, merge_color));
                painter.line_segment([ctrl2, pos], Stroke::new(2.0, merge_color));
            }
        }

        // Draw commit circle
        if commit.is_merge {
            // Merge commit - filled circle with border
            painter.circle_filled(pos, commit_radius, color);
            painter.circle_stroke(pos, commit_radius, Stroke::new(2.0, if dark_mode { Color32::WHITE } else { Color32::BLACK }));
        } else {
            // Regular commit - filled circle
            painter.circle_filled(pos, commit_radius, color);
        }

        // Draw commit label
        let label = commit.message.as_ref().unwrap_or(&commit.id);
        let label_bg_rect = Rect::from_min_size(
            Pos2::new(offset.x + margin - 5.0, pos.y - font_size * 0.5 - 2.0),
            Vec2::new(label_width - 10.0, font_size + 4.0),
        );
        painter.rect_filled(label_bg_rect, 3.0, line_bg);
        painter.text(
            Pos2::new(offset.x + margin, pos.y),
            egui::Align2::LEFT_CENTER,
            label,
            FontId::proportional(font_size - 2.0),
            text_color,
        );

        last_commit_pos.insert(commit.branch.clone(), pos);
    }

    // Draw branch labels at top
    for branch in &graph.branches {
        if let Some(&x) = branch_x.get(&branch.name) {
            let color = branch_colors[branch.color_idx % branch_colors.len()];
            let pos = Pos2::new(x, margin - 15.0) + offset;
            painter.text(
                pos,
                egui::Align2::CENTER_BOTTOM,
                &branch.name,
                FontId::proportional(font_size - 2.0),
                color,
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Gantt Chart Types and Renderer
// ─────────────────────────────────────────────────────────────────────────────

/// A task in a Gantt chart.
#[derive(Debug, Clone)]
pub struct GanttTask {
    pub id: String,
    pub name: String,
    pub section: Option<String>,
    pub start_day: i32,
    pub duration: i32,
    pub is_milestone: bool,
    pub is_done: bool,
    pub is_active: bool,
    pub is_crit: bool,
}

/// A section in a Gantt chart.
#[derive(Debug, Clone)]
pub struct GanttSection {
    pub name: String,
}

/// A Gantt chart.
#[derive(Debug, Clone)]
pub struct GanttChart {
    pub title: Option<String>,
    pub tasks: Vec<GanttTask>,
    pub sections: Vec<GanttSection>,
}

/// Parse a Gantt chart from source.
pub fn parse_gantt_chart(source: &str) -> Result<GanttChart, String> {
    let mut title: Option<String> = None;
    let mut tasks: Vec<GanttTask> = Vec::new();
    let mut sections: Vec<GanttSection> = Vec::new();
    let mut current_section: Option<String> = None;
    let mut task_map: HashMap<String, i32> = HashMap::new(); // task_id -> end_day
    let mut current_day = 0;
    let mut task_counter = 0;

    for line in source.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

        // Parse title
        if line.to_lowercase().starts_with("title") {
            title = Some(line[5..].trim().to_string());
            continue;
        }

        // Skip dateFormat and other directives
        if line.to_lowercase().starts_with("dateformat") 
            || line.to_lowercase().starts_with("excludes")
            || line.to_lowercase().starts_with("todaymarker")
            || line.to_lowercase().starts_with("axisformat") {
            continue;
        }

        // Parse section
        if line.to_lowercase().starts_with("section") {
            let name = line[7..].trim().to_string();
            sections.push(GanttSection { name: name.clone() });
            current_section = Some(name);
            continue;
        }

        // Parse task: name :id, start, duration or name :id, after id2, duration
        if line.contains(':') {
            task_counter += 1;
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() < 2 {
                continue;
            }

            let name = parts[0].trim().to_string();
            let spec = parts[1].trim();
            let spec_parts: Vec<&str> = spec.split(',').map(|s| s.trim()).collect();

            let mut id = format!("t{}", task_counter);
            let mut start_day = current_day;
            let mut duration = 1;
            let mut is_milestone = false;
            let mut is_done = false;
            let mut is_active = false;
            let mut is_crit = false;

            for (i, part) in spec_parts.iter().enumerate() {
                let part_lower = part.to_lowercase();
                
                if part_lower == "done" {
                    is_done = true;
                } else if part_lower == "active" {
                    is_active = true;
                } else if part_lower == "crit" {
                    is_crit = true;
                } else if part_lower == "milestone" {
                    is_milestone = true;
                    duration = 0;
                } else if part_lower.starts_with("after") {
                    // after task_id
                    let after_id = part[5..].trim();
                    if let Some(&end_day) = task_map.get(after_id) {
                        start_day = end_day;
                    }
                } else if part.ends_with('d') {
                    // Duration like "7d"
                    if let Ok(d) = part[..part.len()-1].parse::<i32>() {
                        duration = d;
                    }
                } else if i == 0 && !part.contains(' ') {
                    // First part might be ID
                    id = part.to_string();
                } else if let Ok(d) = part.parse::<i32>() {
                    // Plain number as duration
                    duration = d;
                }
            }

            let task = GanttTask {
                id: id.clone(),
                name,
                section: current_section.clone(),
                start_day,
                duration,
                is_milestone,
                is_done,
                is_active,
                is_crit,
            };

            task_map.insert(id, start_day + duration);
            current_day = start_day + duration;
            tasks.push(task);
        }
    }

    if tasks.is_empty() {
        return Err("No tasks found in Gantt chart".to_string());
    }

    Ok(GanttChart { title, tasks, sections })
}

/// Render a Gantt chart to the UI.
pub fn render_gantt_chart(
    ui: &mut Ui,
    chart: &GanttChart,
    dark_mode: bool,
    font_size: f32,
) {
    let margin = 30.0_f32;
    let row_height = 28.0_f32;
    let row_spacing = 6.0_f32;
    let label_width = 150.0_f32;
    let day_width = 20.0_f32;
    let header_height = 30.0_f32;

    // Find total duration
    let max_day = chart.tasks.iter()
        .map(|t| t.start_day + t.duration)
        .max()
        .unwrap_or(10);

    let total_width = margin * 2.0 + label_width + (max_day as f32 + 2.0) * day_width;
    let total_height = margin * 2.0 + header_height + chart.tasks.len() as f32 * (row_height + row_spacing);

    // Colors
    let (bg_color, grid_color, text_color, task_done, task_active, task_normal, task_crit, milestone_color) = if dark_mode {
        (
            Color32::from_rgb(35, 40, 50),
            Color32::from_rgb(60, 65, 75),
            Color32::from_rgb(220, 230, 240),
            Color32::from_rgb(80, 140, 80),
            Color32::from_rgb(100, 150, 200),
            Color32::from_rgb(80, 100, 140),
            Color32::from_rgb(200, 80, 80),
            Color32::from_rgb(220, 180, 60),
        )
    } else {
        (
            Color32::from_rgb(250, 252, 255),
            Color32::from_rgb(220, 225, 235),
            Color32::from_rgb(30, 40, 50),
            Color32::from_rgb(100, 180, 100),
            Color32::from_rgb(100, 160, 220),
            Color32::from_rgb(140, 160, 200),
            Color32::from_rgb(220, 100, 100),
            Color32::from_rgb(240, 200, 80),
        )
    };

    let (response, painter) = ui.allocate_painter(
        Vec2::new(total_width.max(400.0), total_height.max(100.0)),
        egui::Sense::hover(),
    );
    let offset = response.rect.min.to_vec2();

    // Draw title
    let mut start_y = margin;
    if let Some(ref title) = chart.title {
        painter.text(
            Pos2::new(offset.x + total_width / 2.0, offset.y + margin / 2.0),
            egui::Align2::CENTER_CENTER,
            title,
            FontId::proportional(font_size + 2.0),
            text_color,
        );
        start_y += 10.0;
    }

    // Draw grid background
    let grid_rect = Rect::from_min_size(
        Pos2::new(offset.x + margin + label_width, offset.y + start_y),
        Vec2::new((max_day as f32 + 1.0) * day_width, total_height - margin - start_y),
    );
    painter.rect_filled(grid_rect, 0.0, bg_color);

    // Draw vertical grid lines (days)
    for day in 0..=max_day {
        let x = offset.x + margin + label_width + day as f32 * day_width;
        painter.line_segment(
            [Pos2::new(x, offset.y + start_y), Pos2::new(x, offset.y + total_height - margin)],
            Stroke::new(1.0, grid_color),
        );
        
        // Day labels (every 5 days or if small chart)
        if day % 5 == 0 || max_day <= 10 {
            painter.text(
                Pos2::new(x + day_width / 2.0, offset.y + start_y + header_height / 2.0),
                egui::Align2::CENTER_CENTER,
                format!("{}", day),
                FontId::proportional(font_size - 3.0),
                text_color.gamma_multiply(0.6),
            );
        }
    }

    // Draw tasks
    let mut y = start_y + header_height;
    let mut current_section: Option<String> = None;

    for task in &chart.tasks {
        // Draw section header if changed
        if task.section != current_section {
            if let Some(ref section) = task.section {
                painter.text(
                    Pos2::new(offset.x + margin, offset.y + y + row_height / 2.0),
                    egui::Align2::LEFT_CENTER,
                    section,
                    FontId::proportional(font_size - 1.0),
                    text_color.gamma_multiply(0.7),
                );
            }
            current_section = task.section.clone();
        }

        // Draw horizontal grid line
        painter.line_segment(
            [Pos2::new(offset.x + margin + label_width, offset.y + y + row_height + row_spacing / 2.0),
             Pos2::new(offset.x + margin + label_width + (max_day as f32 + 1.0) * day_width, offset.y + y + row_height + row_spacing / 2.0)],
            Stroke::new(0.5, grid_color),
        );

        // Draw task label
        painter.text(
            Pos2::new(offset.x + margin + label_width - 8.0, offset.y + y + row_height / 2.0),
            egui::Align2::RIGHT_CENTER,
            &task.name,
            FontId::proportional(font_size - 2.0),
            text_color,
        );

        // Draw task bar or milestone
        let task_x = offset.x + margin + label_width + task.start_day as f32 * day_width;
        let task_y = offset.y + y + 4.0;

        if task.is_milestone {
            // Diamond for milestone
            let center = Pos2::new(task_x, task_y + row_height / 2.0 - 4.0);
            let size = 8.0;
            painter.add(egui::Shape::convex_polygon(
                vec![
                    center + Vec2::new(0.0, -size),
                    center + Vec2::new(size, 0.0),
                    center + Vec2::new(0.0, size),
                    center + Vec2::new(-size, 0.0),
                ],
                milestone_color,
                Stroke::NONE,
            ));
        } else {
            // Bar for task
            let bar_width = task.duration as f32 * day_width;
            let bar_height = row_height - 8.0;
            let bar_rect = Rect::from_min_size(
                Pos2::new(task_x, task_y),
                Vec2::new(bar_width.max(4.0), bar_height),
            );
            
            let bar_color = if task.is_crit {
                task_crit
            } else if task.is_done {
                task_done
            } else if task.is_active {
                task_active
            } else {
                task_normal
            };

            painter.rect_filled(bar_rect, 3.0, bar_color);
            
            // Progress indicator for done tasks
            if task.is_done {
                painter.text(
                    bar_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "✓",
                    FontId::proportional(font_size - 3.0),
                    Color32::WHITE,
                );
            }
        }

        y += row_height + row_spacing;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Timeline Types and Renderer
// ─────────────────────────────────────────────────────────────────────────────

/// A period in a timeline.
#[derive(Debug, Clone)]
pub struct TimelinePeriod {
    pub label: String,
    pub events: Vec<String>,
}

/// A timeline diagram.
#[derive(Debug, Clone)]
pub struct Timeline {
    pub title: Option<String>,
    pub periods: Vec<TimelinePeriod>,
}

/// Parse a timeline from source.
pub fn parse_timeline(source: &str) -> Result<Timeline, String> {
    let mut title: Option<String> = None;
    let mut periods: Vec<TimelinePeriod> = Vec::new();
    let mut current_period: Option<TimelinePeriod> = None;

    for line in source.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

        // Parse title
        if line.to_lowercase().starts_with("title") {
            title = Some(line[5..].trim().to_string());
            continue;
        }

        // Check if this is a period label (doesn't start with whitespace in original)
        // or a section marker
        if line.to_lowercase().starts_with("section") {
            // Save previous period
            if let Some(p) = current_period.take() {
                if !p.events.is_empty() || periods.is_empty() {
                    periods.push(p);
                }
            }
            let label = line[7..].trim().to_string();
            current_period = Some(TimelinePeriod {
                label,
                events: Vec::new(),
            });
            continue;
        }

        // Check if line starts with a date/period pattern or is indented event
        if line.contains(':') {
            // Period with events: "2024-Q1 : Event 1 : Event 2"
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                // Save previous period
                if let Some(p) = current_period.take() {
                    periods.push(p);
                }
                
                let label = parts[0].trim().to_string();
                let events: Vec<String> = parts[1..].iter()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                
                periods.push(TimelinePeriod { label, events });
            }
        } else if current_period.is_some() {
            // Event for current period
            if let Some(ref mut p) = current_period {
                p.events.push(line.to_string());
            }
        } else {
            // New period without colon
            if let Some(p) = current_period.take() {
                periods.push(p);
            }
            current_period = Some(TimelinePeriod {
                label: line.to_string(),
                events: Vec::new(),
            });
        }
    }

    // Save last period
    if let Some(p) = current_period {
        periods.push(p);
    }

    if periods.is_empty() {
        return Err("No periods found in timeline".to_string());
    }

    Ok(Timeline { title, periods })
}

/// Render a timeline to the UI.
pub fn render_timeline(
    ui: &mut Ui,
    timeline: &Timeline,
    dark_mode: bool,
    font_size: f32,
) {
    let margin = 30.0_f32;
    let period_width = 160.0_f32;
    let period_spacing = 20.0_f32;
    let event_height = font_size + 6.0;
    let header_height = 40.0_f32;
    let timeline_y = 80.0_f32;

    // Calculate max events per period for height
    let max_events = timeline.periods.iter()
        .map(|p| p.events.len())
        .max()
        .unwrap_or(1)
        .max(1);

    let total_width = margin * 2.0 + timeline.periods.len() as f32 * (period_width + period_spacing);
    let total_height = margin * 2.0 + timeline_y + max_events as f32 * (event_height + 8.0) + 40.0;

    // Colors
    let (bg_color, line_color, text_color, period_colors) = if dark_mode {
        (
            Color32::from_rgb(35, 40, 50),
            Color32::from_rgb(100, 140, 180),
            Color32::from_rgb(220, 230, 240),
            vec![
                Color32::from_rgb(80, 140, 200),
                Color32::from_rgb(100, 180, 140),
                Color32::from_rgb(200, 160, 100),
                Color32::from_rgb(180, 120, 160),
                Color32::from_rgb(140, 160, 200),
            ],
        )
    } else {
        (
            Color32::from_rgb(250, 252, 255),
            Color32::from_rgb(100, 140, 180),
            Color32::from_rgb(30, 40, 50),
            vec![
                Color32::from_rgb(70, 130, 180),
                Color32::from_rgb(80, 160, 120),
                Color32::from_rgb(180, 140, 80),
                Color32::from_rgb(160, 100, 140),
                Color32::from_rgb(120, 140, 180),
            ],
        )
    };

    let (response, painter) = ui.allocate_painter(
        Vec2::new(total_width.max(400.0), total_height.max(150.0)),
        egui::Sense::hover(),
    );
    let offset = response.rect.min.to_vec2();

    // Draw title
    if let Some(ref title) = timeline.title {
        painter.text(
            Pos2::new(offset.x + total_width / 2.0, offset.y + margin),
            egui::Align2::CENTER_CENTER,
            title,
            FontId::proportional(font_size + 2.0),
            text_color,
        );
    }

    // Draw timeline line
    let line_y = offset.y + timeline_y;
    let line_start = offset.x + margin + period_width / 2.0;
    let line_end = offset.x + margin + (timeline.periods.len() - 1) as f32 * (period_width + period_spacing) + period_width / 2.0;
    painter.line_segment(
        [Pos2::new(line_start, line_y), Pos2::new(line_end, line_y)],
        Stroke::new(3.0, line_color),
    );

    // Draw periods and events
    for (i, period) in timeline.periods.iter().enumerate() {
        let x = offset.x + margin + i as f32 * (period_width + period_spacing) + period_width / 2.0;
        let color = period_colors[i % period_colors.len()];

        // Draw period marker (circle on timeline)
        painter.circle_filled(Pos2::new(x, line_y), 8.0, color);
        painter.circle_stroke(Pos2::new(x, line_y), 8.0, Stroke::new(2.0, if dark_mode { Color32::WHITE } else { Color32::BLACK }));

        // Draw period label above
        painter.text(
            Pos2::new(x, line_y - 20.0),
            egui::Align2::CENTER_BOTTOM,
            &period.label,
            FontId::proportional(font_size),
            color,
        );

        // Draw events below
        let mut event_y = line_y + 25.0;
        for event in &period.events {
            let event_rect = Rect::from_center_size(
                Pos2::new(x, event_y + event_height / 2.0),
                Vec2::new(period_width - 10.0, event_height),
            );
            painter.rect_filled(event_rect, 4.0, color.gamma_multiply(0.2));
            painter.rect_stroke(event_rect, 4.0, Stroke::new(1.0, color.gamma_multiply(0.5)));
            painter.text(
                event_rect.center(),
                egui::Align2::CENTER_CENTER,
                event,
                FontId::proportional(font_size - 2.0),
                text_color,
            );
            event_y += event_height + 8.0;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// User Journey Types and Renderer
// ─────────────────────────────────────────────────────────────────────────────

/// A task in a user journey.
#[derive(Debug, Clone)]
pub struct JourneyTask {
    pub name: String,
    pub score: i32,  // 1-5 satisfaction score
    pub actors: Vec<String>,
}

/// A section in a user journey.
#[derive(Debug, Clone)]
pub struct JourneySection {
    pub name: String,
    pub tasks: Vec<JourneyTask>,
}

/// A user journey diagram.
#[derive(Debug, Clone)]
pub struct UserJourney {
    pub title: Option<String>,
    pub sections: Vec<JourneySection>,
}

/// Parse a user journey from source.
pub fn parse_user_journey(source: &str) -> Result<UserJourney, String> {
    let mut title: Option<String> = None;
    let mut sections: Vec<JourneySection> = Vec::new();
    let mut current_section: Option<JourneySection> = None;

    for line in source.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

        // Parse title
        if line.to_lowercase().starts_with("title") {
            title = Some(line[5..].trim().to_string());
            continue;
        }

        // Parse section
        if line.to_lowercase().starts_with("section") {
            // Save previous section
            if let Some(s) = current_section.take() {
                sections.push(s);
            }
            let name = line[7..].trim().to_string();
            current_section = Some(JourneySection {
                name,
                tasks: Vec::new(),
            });
            continue;
        }

        // Parse task: "Task name: score: Actor1, Actor2"
        if line.contains(':') {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 2 {
                let name = parts[0].trim().to_string();
                let score = parts[1].trim().parse::<i32>().unwrap_or(3);
                let actors: Vec<String> = if parts.len() > 2 {
                    parts[2].split(',').map(|s| s.trim().to_string()).collect()
                } else {
                    Vec::new()
                };

                let task = JourneyTask { name, score, actors };

                if let Some(ref mut s) = current_section {
                    s.tasks.push(task);
                } else {
                    // Create default section if none exists
                    current_section = Some(JourneySection {
                        name: "Journey".to_string(),
                        tasks: vec![task],
                    });
                }
            }
        }
    }

    // Save last section
    if let Some(s) = current_section {
        sections.push(s);
    }

    if sections.is_empty() {
        return Err("No sections found in user journey".to_string());
    }

    Ok(UserJourney { title, sections })
}

/// Render a user journey to the UI.
pub fn render_user_journey(
    ui: &mut Ui,
    journey: &UserJourney,
    dark_mode: bool,
    font_size: f32,
) {
    let margin = 30.0_f32;
    let task_width = 140.0_f32;
    let task_spacing = 15.0_f32;
    let section_spacing = 30.0_f32;
    let row_height = 80.0_f32;
    let header_height = 50.0_f32;

    // Count total tasks
    let total_tasks: usize = journey.sections.iter().map(|s| s.tasks.len()).sum();

    let total_width = margin * 2.0 + total_tasks as f32 * (task_width + task_spacing) 
        + journey.sections.len() as f32 * section_spacing;
    let total_height = margin * 2.0 + header_height + row_height + 60.0;

    // Colors - score based (1=red, 5=green)
    let score_colors = if dark_mode {
        vec![
            Color32::from_rgb(200, 80, 80),   // 1 - Bad
            Color32::from_rgb(200, 140, 80),  // 2 - Poor
            Color32::from_rgb(200, 200, 80),  // 3 - Neutral
            Color32::from_rgb(140, 200, 80),  // 4 - Good
            Color32::from_rgb(80, 200, 120),  // 5 - Great
        ]
    } else {
        vec![
            Color32::from_rgb(220, 100, 100),
            Color32::from_rgb(220, 160, 100),
            Color32::from_rgb(220, 220, 100),
            Color32::from_rgb(160, 200, 100),
            Color32::from_rgb(100, 180, 120),
        ]
    };
    let text_color = if dark_mode { Color32::from_rgb(220, 230, 240) } else { Color32::from_rgb(30, 40, 50) };
    let line_color = if dark_mode { Color32::from_rgb(80, 90, 100) } else { Color32::from_rgb(180, 190, 200) };
    let section_color = if dark_mode { Color32::from_rgb(100, 140, 180) } else { Color32::from_rgb(80, 120, 160) };

    let (response, painter) = ui.allocate_painter(
        Vec2::new(total_width.max(400.0), total_height.max(150.0)),
        egui::Sense::hover(),
    );
    let offset = response.rect.min.to_vec2();

    // Draw title
    if let Some(ref title) = journey.title {
        painter.text(
            Pos2::new(offset.x + total_width / 2.0, offset.y + margin),
            egui::Align2::CENTER_CENTER,
            title,
            FontId::proportional(font_size + 2.0),
            text_color,
        );
    }

    // Draw journey path
    let path_y = offset.y + header_height + row_height / 2.0 + 10.0;
    let mut x = offset.x + margin + task_width / 2.0;
    let mut prev_x: Option<f32> = None;
    let mut prev_score_y: Option<f32> = None;

    for (section_idx, section) in journey.sections.iter().enumerate() {
        // Draw section label
        let section_start_x = x;
        
        for (task_idx, task) in section.tasks.iter().enumerate() {
            let score_idx = (task.score.clamp(1, 5) - 1) as usize;
            let color = score_colors[score_idx];
            
            // Score affects Y position (higher score = higher position)
            let score_offset = (3 - task.score) as f32 * 10.0;
            let task_y = path_y + score_offset;

            // Draw connection line from previous task
            if let (Some(px), Some(py)) = (prev_x, prev_score_y) {
                painter.line_segment(
                    [Pos2::new(px, py), Pos2::new(x, task_y)],
                    Stroke::new(2.0, line_color),
                );
            }

            // Draw task card
            let card_rect = Rect::from_center_size(
                Pos2::new(x, task_y),
                Vec2::new(task_width - 10.0, 50.0),
            );
            painter.rect_filled(card_rect, 6.0, color.gamma_multiply(0.3));
            painter.rect_stroke(card_rect, 6.0, Stroke::new(2.0, color));

            // Draw task name
            painter.text(
                Pos2::new(x, task_y - 8.0),
                egui::Align2::CENTER_CENTER,
                &task.name,
                FontId::proportional(font_size - 2.0),
                text_color,
            );

            // Draw score indicator (filled circle - size reflects score)
            let indicator_radius = 4.0 + task.score as f32 * 1.0; // 5-9 radius based on score
            painter.circle_filled(
                Pos2::new(x, task_y + 12.0),
                indicator_radius,
                color,
            );

            // Draw actors below
            if !task.actors.is_empty() {
                let actors_text = task.actors.join(", ");
                painter.text(
                    Pos2::new(x, card_rect.max.y + 10.0),
                    egui::Align2::CENTER_TOP,
                    &actors_text,
                    FontId::proportional(font_size - 3.0),
                    text_color.gamma_multiply(0.6),
                );
            }

            prev_x = Some(x);
            prev_score_y = Some(task_y);
            x += task_width + task_spacing;
        }

        // Draw section label above the section
        let section_end_x = x - task_spacing;
        let section_mid_x = (section_start_x + section_end_x) / 2.0;
        painter.text(
            Pos2::new(section_mid_x, offset.y + header_height - 5.0),
            egui::Align2::CENTER_BOTTOM,
            &section.name,
            FontId::proportional(font_size - 1.0),
            section_color,
        );

        // Add section spacing
        if section_idx < journey.sections.len() - 1 {
            x += section_spacing;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Result of attempting to render a mermaid diagram.
#[derive(Debug, Clone)]
pub enum RenderResult {
    /// Successfully rendered.
    Success,
    /// Parse error with message.
    ParseError(String),
    /// Diagram type not yet supported.
    Unsupported(String),
}

/// Render a mermaid diagram to the UI.
///
/// Returns a RenderResult indicating success or failure.
pub fn render_mermaid_diagram(
    ui: &mut Ui,
    source: &str,
    dark_mode: bool,
    font_size: f32,
) -> RenderResult {
    let source = source.trim();
    if source.is_empty() {
        return RenderResult::ParseError("Empty diagram source".to_string());
    }

    // Detect diagram type from first non-comment line
    let first_line = source.lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty() && !l.starts_with("%%"))
        .unwrap_or("")
        .to_lowercase();
    
    if first_line.starts_with("flowchart") || first_line.starts_with("graph") {
        match parse_flowchart(source) {
            Ok(flowchart) => {
                let colors = if dark_mode {
                    FlowchartColors::dark()
                } else {
                    FlowchartColors::light()
                };
                let text_measurer = EguiTextMeasurer::new(ui);
                let layout = layout_flowchart(&flowchart, ui.available_width(), font_size, &text_measurer);
                render_flowchart(ui, &flowchart, &layout, &colors, font_size);
                RenderResult::Success
            }
            Err(e) => RenderResult::ParseError(e),
        }
    } else if first_line.starts_with("sequencediagram") {
        match parse_sequence_diagram(source) {
            Ok(diagram) => {
                render_sequence_diagram(ui, &diagram, dark_mode, font_size);
                RenderResult::Success
            }
            Err(e) => RenderResult::ParseError(e),
        }
    } else if first_line.starts_with("pie") {
        match parse_pie_chart(source) {
            Ok(chart) => {
                render_pie_chart(ui, &chart, dark_mode, font_size);
                RenderResult::Success
            }
            Err(e) => RenderResult::ParseError(e),
        }
    } else if first_line.starts_with("statediagram") {
        match parse_state_diagram(source) {
            Ok(diagram) => {
                render_state_diagram(ui, &diagram, dark_mode, font_size);
                RenderResult::Success
            }
            Err(e) => RenderResult::ParseError(e),
        }
    } else if first_line.starts_with("mindmap") {
        match parse_mindmap(source) {
            Ok(mindmap) => {
                render_mindmap(ui, &mindmap, dark_mode, font_size);
                RenderResult::Success
            }
            Err(e) => RenderResult::ParseError(e),
        }
    } else if first_line.starts_with("classdiagram") {
        match parse_class_diagram(source) {
            Ok(diagram) => {
                render_class_diagram(ui, &diagram, dark_mode, font_size);
                RenderResult::Success
            }
            Err(e) => RenderResult::ParseError(e),
        }
    } else if first_line.starts_with("erdiagram") {
        match parse_er_diagram(source) {
            Ok(diagram) => {
                render_er_diagram(ui, &diagram, dark_mode, font_size);
                RenderResult::Success
            }
            Err(e) => RenderResult::ParseError(e),
        }
    } else if first_line.starts_with("gantt") {
        match parse_gantt_chart(source) {
            Ok(chart) => {
                render_gantt_chart(ui, &chart, dark_mode, font_size);
                RenderResult::Success
            }
            Err(e) => RenderResult::ParseError(e),
        }
    } else if first_line.starts_with("gitgraph") {
        match parse_git_graph(source) {
            Ok(graph) => {
                render_git_graph(ui, &graph, dark_mode, font_size);
                RenderResult::Success
            }
            Err(e) => RenderResult::ParseError(e),
        }
    } else if first_line.starts_with("timeline") {
        match parse_timeline(source) {
            Ok(timeline) => {
                render_timeline(ui, &timeline, dark_mode, font_size);
                RenderResult::Success
            }
            Err(e) => RenderResult::ParseError(e),
        }
    } else if first_line.starts_with("journey") {
        match parse_user_journey(source) {
            Ok(journey) => {
                render_user_journey(ui, &journey, dark_mode, font_size);
                RenderResult::Success
            }
            Err(e) => RenderResult::ParseError(e),
        }
    } else {
        RenderResult::ParseError(format!("Unknown diagram type: {}", first_line))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_flowchart() {
        let source = "flowchart TD\n  A[Start] --> B[End]";
        let result = parse_flowchart(source);
        assert!(result.is_ok());
        let flowchart = result.unwrap();
        assert_eq!(flowchart.nodes.len(), 2);
        assert_eq!(flowchart.edges.len(), 1);
    }

    #[test]
    fn test_parse_direction() {
        assert_eq!(parse_direction("flowchart TD"), FlowDirection::TopDown);
        assert_eq!(parse_direction("flowchart LR"), FlowDirection::LeftRight);
        assert_eq!(parse_direction("flowchart BT"), FlowDirection::BottomUp);
        assert_eq!(parse_direction("flowchart RL"), FlowDirection::RightLeft);
    }

    #[test]
    fn test_parse_node_shapes() {
        let rect = parse_node_from_text("A[Text]").unwrap();
        assert_eq!(rect.2, NodeShape::Rectangle);
        
        let round = parse_node_from_text("B(Text)").unwrap();
        assert_eq!(round.2, NodeShape::RoundRect);
        
        let diamond = parse_node_from_text("C{Decision}").unwrap();
        assert_eq!(diamond.2, NodeShape::Diamond);
        
        let circle = parse_node_from_text("D((Circle))").unwrap();
        assert_eq!(circle.2, NodeShape::Circle);
    }

    #[test]
    fn test_parse_edge_with_label() {
        let result = parse_edge_line("A -->|Yes| B");
        assert!(result.is_some());
        let (nodes, edge) = result.unwrap();
        assert_eq!(nodes.len(), 2);
        let edge = edge.unwrap();
        assert_eq!(edge.label, Some("Yes".to_string()));
    }

    #[test]
    fn test_parse_multiple_edges() {
        let source = r#"flowchart TD
            A[Start] --> B{Decision}
            B -->|Yes| C[Great!]
            B -->|No| D[Debug]
            D --> E[Fix]
            E --> B
            C --> F[End]"#;
        
        let result = parse_flowchart(source);
        assert!(result.is_ok());
        let flowchart = result.unwrap();
        assert_eq!(flowchart.nodes.len(), 6); // A, B, C, D, E, F
        assert_eq!(flowchart.edges.len(), 6);
    }

    #[test]
    fn test_layout_produces_valid_positions() {
        let source = "flowchart TD\n  A[Start] --> B[End]";
        let flowchart = parse_flowchart(source).unwrap();
        let text_measurer = EstimatedTextMeasurer::new();
        let layout = layout_flowchart(&flowchart, 400.0, 14.0, &text_measurer);

        assert_eq!(layout.nodes.len(), 2);
        assert!(layout.nodes.contains_key("A"));
        assert!(layout.nodes.contains_key("B"));

        // In TD layout, B should be below A
        let a_pos = layout.nodes.get("A").unwrap().pos;
        let b_pos = layout.nodes.get("B").unwrap().pos;
        assert!(b_pos.y > a_pos.y);
    }

    #[test]
    fn test_text_measurer_trait() {
        let measurer = EstimatedTextMeasurer::new();

        // Test basic measurement
        let size = measurer.measure("Hello", 14.0);
        assert!(size.width > 0.0);
        assert!(size.height > 0.0);

        // Longer text should have greater width
        let size_longer = measurer.measure("Hello World", 14.0);
        assert!(size_longer.width > size.width);

        // Test row height
        let row_height = measurer.row_height(14.0);
        assert!(row_height > 0.0);
    }

    #[test]
    fn test_truncate_with_ellipsis() {
        let measurer = EstimatedTextMeasurer::new();

        // Text that fits should not be truncated
        let short_text = "Hi";
        let result = measurer.truncate_with_ellipsis(short_text, 14.0, 100.0);
        assert_eq!(result, short_text);

        // Long text should be truncated
        let long_text = "This is a very long label that should be truncated";
        let result = measurer.truncate_with_ellipsis(long_text, 14.0, 50.0);
        assert!(result.len() < long_text.len());
        assert!(result.ends_with('…'));
    }
}
