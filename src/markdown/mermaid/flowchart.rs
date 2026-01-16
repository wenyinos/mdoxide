//! Flowchart diagram parsing, layout, and rendering.
//!
//! This module implements Mermaid flowchart/graph diagrams with support for:
//! - Multiple flow directions (TD, TB, LR, RL, BT)
//! - Various node shapes (rectangle, diamond, circle, etc.)
//! - Edge styles (solid, dotted, thick)
//! - Arrow types (arrow, circle, cross, bidirectional)
//! - Subgraphs with nesting
//! - Chained edges (A --> B --> C)
//! - Cycle detection and back-edge rendering

use egui::{Color32, FontId, Pos2, Rect, Rounding, Stroke, Ui, Vec2};
use std::collections::HashMap;

use super::text::{EguiTextMeasurer, TextMeasurer};

// ─────────────────────────────────────────────────────────────────────────────
// AST Types
// ─────────────────────────────────────────────────────────────────────────────

/// Direction of the flowchart layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlowDirection {
    #[default]
    TopDown,   // TD or TB
    BottomUp,  // BT
    LeftRight, // LR
    RightLeft, // RL
}

/// Shape of a flowchart node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeShape {
    #[default]
    Rectangle,     // [text]
    RoundRect,     // (text)
    Stadium,       // ([text])
    Diamond,       // {text}
    Hexagon,       // {{text}}
    Parallelogram, // [/text/]
    Circle,        // ((text))
    Cylinder,      // [(text)]
    Subroutine,    // [[text]]
    Asymmetric,    // >text]
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
    Solid,  // ---
    Dotted, // -.-
    Thick,  // ===
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

/// Custom style for a node defined via classDef.
#[derive(Debug, Clone)]
pub struct NodeStyle {
    /// Fill color (background)
    pub fill: Option<Color32>,
    /// Stroke color (border)
    pub stroke: Option<Color32>,
    /// Stroke width
    pub stroke_width: Option<f32>,
}

impl Default for NodeStyle {
    fn default() -> Self {
        Self {
            fill: None,
            stroke: None,
            stroke_width: None,
        }
    }
}

/// Custom style for an edge defined via linkStyle directive.
#[derive(Debug, Clone, Default)]
pub struct LinkStyle {
    /// Stroke color
    pub stroke: Option<Color32>,
    /// Stroke width
    pub stroke_width: Option<f32>,
}

/// A parsed flowchart.
#[derive(Debug, Clone, Default)]
pub struct Flowchart {
    pub direction: FlowDirection,
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
    /// Subgraphs in the flowchart (order matters for rendering - parents before children)
    pub subgraphs: Vec<FlowSubgraph>,
    /// Class definitions: class_name -> NodeStyle
    pub class_defs: HashMap<String, NodeStyle>,
    /// Node class assignments: node_id -> class_name
    pub node_classes: HashMap<String, String>,
    /// Link styles: edge_index -> LinkStyle
    pub link_styles: HashMap<usize, LinkStyle>,
    /// Default style applied to all edges without explicit style
    pub default_link_style: Option<LinkStyle>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser
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

    while line_idx < lines.len() {
        let line = lines[line_idx].trim();
        line_idx += 1;

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

        let line_lower = line.to_lowercase();

        // Parse classDef directive: classDef className fill:#fff,stroke:#000,stroke-width:2px
        if line_lower.starts_with("classdef ") {
            if let Some((class_name, style)) = parse_class_def(line) {
                flowchart.class_defs.insert(class_name, style);
            }
            continue;
        }

        // Parse class directive: class nodeId1,nodeId2 className
        if line_lower.starts_with("class ") {
            parse_class_assignment(line, &mut flowchart.node_classes);
            continue;
        }

        // Parse linkStyle directive: linkStyle <index|default> <css-properties>
        if line_lower.starts_with("linkstyle ") {
            parse_link_style(line, &mut flowchart);
            continue;
        }

        // Skip other styling directives (not yet implemented, but shouldn't create nodes)
        if line_lower.starts_with("style ") || line_lower.starts_with("click ") {
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

        // Try to parse as edge (contains arrow) - use the full parser for chained edges
        if let Some((nodes, edges)) = parse_edge_line_full(line) {
            for (id, label, shape) in nodes {
                if let Some(&idx) = node_map.get(&id) {
                    // Node exists - update if new definition has more info
                    let existing = &mut flowchart.nodes[idx];
                    // Only update and associate with subgraph if this is a NEW definition
                    // (has label content beyond just the ID). Plain references like "C --> E"
                    // where C was already defined elsewhere should NOT add C to this subgraph.
                    if label != id && existing.label == existing.id {
                        existing.label = label;
                        existing.shape = shape;
                        
                        // Only associate with current subgraph when actually defining the node
                        if let Some(current) = subgraph_stack.last_mut() {
                            if !current.node_ids.contains(&id) {
                                current.node_ids.push(id);
                            }
                        }
                    }
                    // Note: Plain references to existing nodes don't add them to the current subgraph
                } else {
                    node_map.insert(id.clone(), flowchart.nodes.len());
                    flowchart.nodes.push(FlowNode {
                        id: id.clone(),
                        label,
                        shape,
                    });

                    // Associate with current subgraph if any
                    if let Some(current) = subgraph_stack.last_mut() {
                        current.node_ids.push(id);
                    }
                }
            }
            // Add all edges from the chain
            for e in edges {
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
                
                // Associate with current subgraph if node appears inside it
                if let Some(current) = subgraph_stack.last_mut() {
                    if !current.node_ids.contains(&node.id) {
                        current.node_ids.push(node.id);
                    }
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
    let rest = line
        .trim_start_matches(|c: char| c.is_ascii_alphabetic())
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

pub(crate) fn parse_direction(header: &str) -> FlowDirection {
    // Strip trailing semicolon from header (e.g., "graph TD;")
    let header = strip_trailing_semicolon(header);
    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts.len() > 1 {
        // Strip any trailing semicolon from the direction part too
        let direction = strip_trailing_semicolon(parts[1]);
        match direction.to_uppercase().as_str() {
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

/// Parse a classDef directive: `classDef className fill:#fff,stroke:#000,stroke-width:2px`
/// Returns (class_name, NodeStyle) on success.
fn parse_class_def(line: &str) -> Option<(String, NodeStyle)> {
    // Remove "classDef " prefix (case-insensitive)
    let rest = if line.to_lowercase().starts_with("classdef ") {
        &line[9..] // len("classdef ") = 9
    } else {
        return None;
    };

    let rest = rest.trim();
    if rest.is_empty() {
        return None;
    }

    // Split into class name and style properties
    // Format: className property1:value1,property2:value2
    let mut parts = rest.splitn(2, char::is_whitespace);
    let class_name = parts.next()?.trim().to_string();
    let properties_str = parts.next().unwrap_or("").trim();

    if class_name.is_empty() {
        return None;
    }

    let mut style = NodeStyle::default();

    // Parse comma-separated properties
    for prop in properties_str.split(',') {
        let prop = prop.trim();
        if let Some(colon_pos) = prop.find(':') {
            let key = prop[..colon_pos].trim().to_lowercase();
            let value = prop[colon_pos + 1..].trim();

            match key.as_str() {
                "fill" => {
                    style.fill = parse_css_color(value);
                }
                "stroke" => {
                    style.stroke = parse_css_color(value);
                }
                "stroke-width" => {
                    style.stroke_width = parse_stroke_width(value);
                }
                _ => {
                    // Ignore unknown properties (color, font-size, etc.)
                }
            }
        }
    }

    Some((class_name, style))
}

/// Parse a class assignment directive: `class nodeId1,nodeId2 className`
/// or inline syntax: `class nodeId className`
fn parse_class_assignment(line: &str, node_classes: &mut HashMap<String, String>) {
    // Remove "class " prefix (case-insensitive)
    let rest = if line.to_lowercase().starts_with("class ") {
        &line[6..] // len("class ") = 6
    } else {
        return;
    };

    let rest = rest.trim();
    if rest.is_empty() {
        return;
    }

    // Split into node IDs and class name
    // The class name is the last whitespace-separated token
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    if tokens.len() < 2 {
        return;
    }

    let class_name = tokens.last().unwrap().trim().to_string();
    
    // Everything before the class name is node IDs (comma-separated)
    let node_ids_str = tokens[..tokens.len() - 1].join(" ");
    
    // Parse node IDs (can be comma-separated: "A,B,C" or "A, B, C")
    for node_id in node_ids_str.split(',') {
        let node_id = node_id.trim();
        if !node_id.is_empty() {
            node_classes.insert(node_id.to_string(), class_name.clone());
        }
    }
}

/// Parse a CSS color value (hex format).
/// Supports: #RGB, #RRGGBB, #RRGGBBAA
fn parse_css_color(value: &str) -> Option<Color32> {
    let value = value.trim();
    
    if !value.starts_with('#') {
        // Could add named color support here later (red, blue, etc.)
        return None;
    }

    let hex = &value[1..];
    
    match hex.len() {
        // #RGB -> #RRGGBB
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some(Color32::from_rgb(r, g, b))
        }
        // #RRGGBB
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color32::from_rgb(r, g, b))
        }
        // #RRGGBBAA
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Color32::from_rgba_unmultiplied(r, g, b, a))
        }
        _ => None,
    }
}

/// Parse stroke-width value, e.g., "2px", "1.5px", "2"
fn parse_stroke_width(value: &str) -> Option<f32> {
    let value = value.trim();
    // Remove "px" suffix if present
    let num_str = value.strip_suffix("px").unwrap_or(value);
    num_str.parse::<f32>().ok()
}

/// Parse a linkStyle directive and update the flowchart.
/// Syntax: linkStyle <index|default> <css-properties>
/// Examples:
///   linkStyle 0 stroke:#f00,stroke-width:2px
///   linkStyle default stroke:#333
fn parse_link_style(line: &str, flowchart: &mut Flowchart) {
    // Remove "linkStyle " prefix (case-insensitive)
    let content = if line.len() > 10 { &line[10..] } else { return };
    let content = content.trim();

    // Split into index/keyword and CSS properties
    // Find the first space that separates the index from the style properties
    let (index_part, css_part) = match content.find(char::is_whitespace) {
        Some(pos) => {
            let (idx, css) = content.split_at(pos);
            (idx.trim(), css.trim())
        }
        None => return, // No CSS properties provided
    };

    // Parse CSS properties into a LinkStyle
    let mut style = LinkStyle::default();
    for property in css_part.split(',') {
        let property = property.trim();
        if let Some((key, value)) = property.split_once(':') {
            let key = key.trim().to_lowercase();
            let value = value.trim();

            match key.as_str() {
                "stroke" => {
                    style.stroke = parse_css_color(value);
                }
                "stroke-width" => {
                    style.stroke_width = parse_stroke_width(value);
                }
                _ => {} // Ignore unknown properties
            }
        }
    }

    // Check if it's the default keyword or a numeric index
    if index_part.eq_ignore_ascii_case("default") {
        flowchart.default_link_style = Some(style);
    } else if let Ok(index) = index_part.parse::<usize>() {
        flowchart.link_styles.insert(index, style);
    }
    // Invalid index is silently ignored
}

/// Arrow pattern definition for parsing edges.
/// Ordered by length (longest first) to ensure correct matching.
const ARROW_PATTERNS: &[(&str, EdgeStyle, ArrowHead, ArrowHead)] = &[
    // 4+ char patterns first
    ("<-->", EdgeStyle::Solid, ArrowHead::Arrow, ArrowHead::Arrow),
    ("o--o", EdgeStyle::Solid, ArrowHead::Circle, ArrowHead::Circle),
    ("x--x", EdgeStyle::Solid, ArrowHead::Cross, ArrowHead::Cross),
    ("--->", EdgeStyle::Solid, ArrowHead::None, ArrowHead::Arrow),
    ("-.->", EdgeStyle::Dotted, ArrowHead::None, ArrowHead::Arrow),
    // 3 char patterns
    ("-->", EdgeStyle::Solid, ArrowHead::None, ArrowHead::Arrow),
    ("---", EdgeStyle::Solid, ArrowHead::None, ArrowHead::None),
    ("-.-", EdgeStyle::Dotted, ArrowHead::None, ArrowHead::None),
    ("==>", EdgeStyle::Thick, ArrowHead::None, ArrowHead::Arrow),
    ("===", EdgeStyle::Thick, ArrowHead::None, ArrowHead::None),
    ("--o", EdgeStyle::Solid, ArrowHead::None, ArrowHead::Circle),
    ("--x", EdgeStyle::Solid, ArrowHead::None, ArrowHead::Cross),
];

/// Find the first arrow pattern in the given text, returning its position, length, and style info.
/// Returns None if no arrow pattern is found.
fn find_arrow_pattern(
    text: &str,
) -> Option<(usize, &'static str, EdgeStyle, ArrowHead, ArrowHead)> {
    let mut best_match: Option<(usize, &'static str, EdgeStyle, ArrowHead, ArrowHead)> = None;

    for &(pattern, style, arrow_start, arrow_end) in ARROW_PATTERNS {
        if let Some(pos) = text.find(pattern) {
            // Take the match with the smallest position, or if same position, longest pattern
            let dominated = best_match.map_or(false, |(best_pos, best_pat, _, _, _)| {
                pos > best_pos || (pos == best_pos && pattern.len() <= best_pat.len())
            });
            if !dominated {
                best_match = Some((pos, pattern, style, arrow_start, arrow_end));
            }
        }
    }

    best_match
}

/// Parse an edge segment: extracts the label (if any) after the arrow and returns the remaining text.
/// For input like "|Yes| B[Node]", returns (Some("Yes"), "B[Node]").
/// For input like "B[Node]", returns (None, "B[Node]").
fn parse_edge_label(text: &str) -> (Option<String>, &str) {
    let text = text.trim();

    // Check for label syntax: |label|
    if text.starts_with('|') {
        if let Some(end_pos) = text[1..].find('|') {
            let label = text[1..=end_pos].trim();
            let rest = text[end_pos + 2..].trim();
            return (Some(clean_label(label)), rest);
        }
    }

    (None, text)
}

/// Extract dash-style edge label from node text.
/// Handles Mermaid syntax like "A[Node]-- label" where the label is between
/// shape closer and arrow pattern.
///
/// Returns (cleaned_node_text, Option<label>)
///
/// Examples:
/// - "od>Odd shape]-- Two line" -> ("od>Odd shape]", Some("Two line"))
/// - "A[Node]-- label" -> ("A[Node]", Some("label"))
/// - "A[Node]-. label" -> ("A[Node]", Some("label"))
/// - "A[Node]== label" -> ("A[Node]", Some("label"))
/// - "A[Node]" -> ("A[Node]", None)
fn extract_dash_label(node_text: &str) -> (&str, Option<String>) {
    let text = node_text.trim();

    // Look for dash-style label patterns after shape closers
    // The patterns are: "-- ", "-. ", "== " (note the space after)
    // These appear after shape closing characters: ], ), }, |
    let label_start_patterns = ["-- ", "-. ", "== "];

    // Find the last shape-closing character position
    let shape_closers = [']', ')', '}', '|'];
    let last_closer_pos = shape_closers
        .iter()
        .filter_map(|&c| text.rfind(c))
        .max();

    if let Some(closer_pos) = last_closer_pos {
        // Check if there's a dash-style label pattern after the closer
        let after_closer = &text[closer_pos + 1..];

        for pattern in &label_start_patterns {
            if after_closer.starts_with(pattern) {
                // Found a dash-style label!
                let label = after_closer[pattern.len()..].trim();
                let node_part = &text[..=closer_pos];
                log::trace!(
                    "extract_dash_label: found dash label, node='{}', label='{}'",
                    node_part,
                    label
                );
                return (node_part, Some(clean_label(label)));
            }
        }

        // Also check for pattern with newline or end of string (label might be empty or whitespace-only)
        // e.g., "A[Node]--" without space before arrow
        for pattern_start in ["--", "-.", "=="] {
            if after_closer.starts_with(pattern_start) {
                // Check it's not part of a longer pattern (like "---" being a valid node ID suffix)
                let rest = &after_closer[pattern_start.len()..];
                if rest.is_empty() || rest.starts_with(char::is_whitespace) {
                    let label = rest.trim();
                    let node_part = &text[..=closer_pos];
                    log::trace!(
                        "extract_dash_label: found dash label (variant), node='{}', label='{}'",
                        node_part,
                        label
                    );
                    return (node_part, Some(clean_label(label)));
                }
            }
        }
    }

    // No dash-style label found
    (text, None)
}

/// Strip trailing semicolon from a string.
fn strip_trailing_semicolon(s: &str) -> &str {
    s.strip_suffix(';').unwrap_or(s).trim_end()
}

/// Split node text by ampersand, handling the `A & B` syntax.
/// Returns a vec of individual node texts.
fn split_by_ampersand(text: &str) -> Vec<&str> {
    // Don't split if the text contains shape markers (brackets, parens, braces)
    // because ampersand inside labels should not be split
    let has_shape_marker = text.contains('[')
        || text.contains('(')
        || text.contains('{')
        || text.contains('>');

    if has_shape_marker {
        // Check if ampersand appears before any shape marker
        if let Some(amp_pos) = text.find('&') {
            let first_marker = [
                text.find('['),
                text.find('('),
                text.find('{'),
                text.find('>'),
            ]
            .into_iter()
            .flatten()
            .min();

            if let Some(marker_pos) = first_marker {
                if amp_pos < marker_pos {
                    // Ampersand is before shape markers - split the IDs part only
                    let ids_part = &text[..marker_pos];
                    let _shape_part = &text[marker_pos..]; // May be used for future shape handling

                    // Check if shape_part is for a single node definition
                    // e.g., "A & B[Label]" - B gets the shape, A is just ID
                    let ids: Vec<&str> = ids_part.split('&').map(|s| s.trim()).collect();
                    if ids.len() > 1 {
                        return ids;
                    }
                }
            }
        }
        return vec![text];
    }

    // No shape markers - safe to split by ampersand
    if text.contains('&') {
        text.split('&').map(|s| s.trim()).filter(|s| !s.is_empty()).collect()
    } else {
        vec![text]
    }
}

/// Parse a line that may contain chained edges, returning all nodes and all edges.
/// This is the implementation that properly handles chains like "A --> B --> C".
/// Also supports ampersand syntax like "A & B --> C & D" which creates multiple edges.
pub(crate) fn parse_edge_line_full(
    line: &str,
) -> Option<(Vec<(String, String, NodeShape)>, Vec<FlowEdge>)> {
    // Strip trailing semicolon from line
    let line = strip_trailing_semicolon(line.trim());

    // Check if line contains any arrow pattern
    if find_arrow_pattern(line).is_none() {
        return None;
    }

    let mut all_nodes: Vec<(String, String, NodeShape)> = Vec::new();
    let mut all_edges: Vec<FlowEdge> = Vec::new();
    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    // For chained edges, we need to track the previous nodes to create edges
    let mut prev_node_ids: Vec<String> = Vec::new();
    let mut remaining = line;

    while !remaining.is_empty() {
        if let Some((arrow_pos, pattern, style, arrow_start, arrow_end)) =
            find_arrow_pattern(remaining)
        {
            let raw_node_text = remaining[..arrow_pos].trim();
            let after_arrow = &remaining[arrow_pos + pattern.len()..];
            let (pipe_label, rest_after_label) = parse_edge_label(after_arrow);

            // Extract any dash-style label from node text (e.g., "A[Node]-- label" -> "A[Node]", "label")
            let (node_text, dash_label) = extract_dash_label(raw_node_text);

            // Prefer pipe-style label, fall back to dash-style label
            let label = pipe_label.or(dash_label);

            // Parse the source node(s) (before the arrow) - may contain ampersands
            let from_ids: Vec<String> = if !node_text.is_empty() {
                let node_parts = split_by_ampersand(node_text);
                let mut ids = Vec::new();
                for part in node_parts {
                    if let Some((id, node_label, shape)) = parse_node_from_text(part) {
                        if !seen_ids.contains(&id) {
                            seen_ids.insert(id.clone());
                            all_nodes.push((id.clone(), node_label, shape));
                        }
                        ids.push(id);
                    }
                }
                ids
            } else {
                // Empty node text - use previous nodes as source (for chained edges)
                prev_node_ids.clone()
            };

            // Find the target node(s)
            let next_segment = rest_after_label.trim();

            // Determine where the target node ends (at the next arrow or end of line)
            let target_end = find_arrow_pattern(next_segment)
                .map(|(pos, _, _, _, _)| pos)
                .unwrap_or(next_segment.len());

            let target_text = strip_trailing_semicolon(next_segment[..target_end].trim());

            // Parse target node(s) - may contain ampersands
            let target_parts = split_by_ampersand(target_text);
            let mut to_ids: Vec<String> = Vec::new();

            for part in target_parts {
                if let Some((to_id, to_label, to_shape)) = parse_node_from_text(part) {
                    if !seen_ids.contains(&to_id) {
                        seen_ids.insert(to_id.clone());
                        all_nodes.push((to_id.clone(), to_label, to_shape));
                    }
                    to_ids.push(to_id);
                }
            }

            // Create edges from all source nodes to all target nodes
            for from in &from_ids {
                for to in &to_ids {
                    all_edges.push(FlowEdge {
                        from: from.clone(),
                        to: to.clone(),
                        label: label.clone(),
                        style,
                        arrow_start,
                        arrow_end,
                    });
                }
            }

            // The targets become the potential sources for the next edge
            prev_node_ids = to_ids;

            // Continue from after the target node (where next arrow might start)
            remaining = &next_segment[target_end..];
        } else {
            break;
        }
    }

    if all_nodes.is_empty() {
        return None;
    }

    Some((all_nodes, all_edges))
}

pub(crate) fn parse_node_from_text(text: &str) -> Option<(String, String, NodeShape)> {
    // Strip trailing semicolon from the text
    let text = strip_trailing_semicolon(text.trim());
    if text.is_empty() {
        return None;
    }

    log::trace!("parse_node_from_text: input='{}'", text);

    // Try various shape patterns
    // Stadium: ([text])
    if text.contains("([") && text.contains("])") {
        if let Some(start) = text.find("([") {
            let id = text[..start].trim();
            let id = if id.is_empty() {
                &text[..start.max(1)]
            } else {
                id
            };
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
                return Some((
                    extract_id(id, text),
                    clean_label(label),
                    NodeShape::Subroutine,
                ));
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
    if text.contains('(')
        && text.contains(')')
        && !text.contains("((")
        && !text.contains("([")
        && !text.contains("[(")
    {
        if let Some(start) = text.find('(') {
            let id = text[..start].trim();
            if let Some(end) = text.rfind(')') {
                let label = text[start + 1..end].trim();
                return Some((
                    extract_id(id, text),
                    clean_label(label),
                    NodeShape::RoundRect,
                ));
            }
        }
    }

    // Rectangle: [text]
    if text.contains('[')
        && text.contains(']')
        && !text.contains("[[")
        && !text.contains("[(")
        && !text.contains("([")
    {
        if let Some(start) = text.find('[') {
            let id = text[..start].trim();
            if let Some(end) = text.rfind(']') {
                let label = text[start + 1..end].trim();
                return Some((
                    extract_id(id, text),
                    clean_label(label),
                    NodeShape::Rectangle,
                ));
            }
        }
    }

    // Asymmetric: >text]
    // Must have > before ] for valid asymmetric syntax
    if text.contains('>') && text.contains(']') {
        if let Some(start) = text.find('>') {
            if let Some(end) = text.rfind(']') {
                // Only valid if > comes before ]
                if start < end {
                    let id = text[..start].trim();
                    let label = text[start + 1..end].trim();
                    log::debug!(
                        "Asymmetric shape detected: id='{}', label='{}', text='{}'",
                        id,
                        label,
                        text
                    );
                    return Some((
                        extract_id(id, text),
                        clean_label(label),
                        NodeShape::Asymmetric,
                    ));
                }
            }
        }
    }

    // Just an ID (no shape specified) - also strip any trailing semicolon from ID
    let id = strip_trailing_semicolon(text.split_whitespace().next().unwrap_or(text));
    log::trace!(
        "parse_node_from_text: no shape matched, defaulting to Rectangle for id='{}', text='{}'",
        id,
        text
    );
    Some((id.to_string(), id.to_string(), NodeShape::Rectangle))
}

fn extract_id(id: &str, full_text: &str) -> String {
    if id.is_empty() {
        // Generate ID from first part of text
        full_text
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect()
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
        nested_subgraph_margin: 10.0,
    };

    // Build internal graph representation
    let graph = FlowGraph::from_flowchart(flowchart, font_size, text_measurer, &config);

    // Run the Sugiyama layout algorithm
    let sugiyama = SugiyamaLayout::new(graph, flowchart.direction, config.clone(), available_width);
    let mut layout = sugiyama.compute();

    // Compute subgraph bounding boxes
    compute_subgraph_layouts(&mut layout, flowchart, &config, font_size, text_measurer);

    layout
}

/// Compute bounding boxes for all subgraphs based on positioned nodes.
/// If a subgraph already has a pre-computed layout, updates only the title.
/// Otherwise, computes bounds from the positioned node locations.
/// 
/// The text_measurer is used to ensure subgraph width accommodates the title text.
fn compute_subgraph_layouts(
    layout: &mut FlowchartLayout,
    flowchart: &Flowchart,
    config: &FlowLayoutConfig,
    font_size: f32,
    text_measurer: &impl TextMeasurer,
) {
    // Process subgraphs in reverse order (children before parents for nested bounds)
    // But store results keyed by ID so we can look up child bounds
    let mut subgraph_bounds: HashMap<String, (Pos2, Pos2)> = HashMap::new();

    for subgraph in flowchart.subgraphs.iter().rev() {
        // Check if we already have a pre-computed layout from SubgraphLayoutEngine
        if let Some(existing) = layout.subgraphs.get_mut(&subgraph.id) {
            // Update title from flowchart (it wasn't available during layout)
            existing.title = subgraph.title.clone();
            
            // Ensure subgraph width accommodates the title text
            if let Some(title) = &subgraph.title {
                let title_text_size = text_measurer.measure(title, font_size);
                // Title padding: 12px left margin + 12px right margin
                let min_width_for_title = title_text_size.width + 24.0;
                if existing.size.x < min_width_for_title {
                    // Expand width to fit title (keeping left edge, expanding right)
                    existing.size.x = min_width_for_title;
                }
            }
            
            // Store bounds for parent subgraph calculations
            subgraph_bounds.insert(
                subgraph.id.clone(),
                (existing.pos, Pos2::new(existing.pos.x + existing.size.x, existing.pos.y + existing.size.y)),
            );
            continue;
        }

        // No pre-computed layout, compute from node positions (fallback)
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut has_content = false;
        let mut has_nested_children = false;

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

        // Include nested subgraph bounds with extra margin for visual separation
        for child_id in &subgraph.child_subgraph_ids {
            if let Some(&(child_min, child_max)) = subgraph_bounds.get(child_id) {
                // For nested subgraphs, we need extra margin to create visual separation
                // between parent and child borders
                let nested_margin = config.nested_subgraph_margin;
                min_x = min_x.min(child_min.x - nested_margin);
                min_y = min_y.min(child_min.y - nested_margin);
                max_x = max_x.max(child_max.x + nested_margin);
                max_y = max_y.max(child_max.y + nested_margin);
                has_content = true;
                has_nested_children = true;
            }
        }

        if has_content {
            // Use larger padding when we have nested children to ensure proper spacing
            let effective_padding = if has_nested_children {
                config.subgraph_padding + config.nested_subgraph_margin
            } else {
                config.subgraph_padding
            };

            // Add padding around content
            let padded_min = Pos2::new(
                min_x - effective_padding,
                min_y - effective_padding - config.subgraph_title_height,
            );
            let mut padded_max = Pos2::new(
                max_x + effective_padding,
                max_y + effective_padding,
            );

            // Ensure subgraph width accommodates the title text
            if let Some(title) = &subgraph.title {
                let title_text_size = text_measurer.measure(title, font_size);
                // Title padding: 12px left margin + 12px right margin
                let min_width_for_title = title_text_size.width + 24.0;
                let current_width = padded_max.x - padded_min.x;
                if current_width < min_width_for_title {
                    // Expand width to fit title (keeping left edge, expanding right)
                    padded_max.x = padded_min.x + min_width_for_title;
                }
            }

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

    // Calculate true bounds including all nodes and subgraphs
    // This handles cases where subgraphs extend into negative coordinates
    // (e.g., due to title height or padding extending above content)
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    // Include node bounds
    for node_layout in layout.nodes.values() {
        min_x = min_x.min(node_layout.pos.x);
        min_y = min_y.min(node_layout.pos.y);
        max_x = max_x.max(node_layout.pos.x + node_layout.size.x);
        max_y = max_y.max(node_layout.pos.y + node_layout.size.y);
    }

    // Include subgraph bounds
    for sg_layout in layout.subgraphs.values() {
        min_x = min_x.min(sg_layout.pos.x);
        min_y = min_y.min(sg_layout.pos.y);
        max_x = max_x.max(sg_layout.pos.x + sg_layout.size.x);
        max_y = max_y.max(sg_layout.pos.y + sg_layout.size.y);
    }

    // If any content extends into negative coordinates, shift everything
    // to ensure all positions are >= 0 (prevents viewport clipping)
    let shift_x = if min_x < 0.0 { -min_x + config.margin } else { 0.0 };
    let shift_y = if min_y < 0.0 { -min_y + config.margin } else { 0.0 };

    if shift_x > 0.0 || shift_y > 0.0 {
        // Shift all node positions
        for node_layout in layout.nodes.values_mut() {
            node_layout.pos.x += shift_x;
            node_layout.pos.y += shift_y;
        }

        // Shift all subgraph positions
        for sg_layout in layout.subgraphs.values_mut() {
            sg_layout.pos.x += shift_x;
            sg_layout.pos.y += shift_y;
        }

        // Update bounds after shift
        max_x += shift_x;
        max_y += shift_y;
    }

    // Update total_size to encompass all content with margin
    layout.total_size.x = layout.total_size.x.max(max_x + config.margin);
    layout.total_size.y = layout.total_size.y.max(max_y + config.margin);
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
    /// Extra margin between nested subgraph boundaries
    nested_subgraph_margin: f32,
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
    /// Subgraph membership: node_index -> Option<subgraph_id>
    /// None means the node is not in any subgraph
    node_subgraph: Vec<Option<String>>,
    /// Subgraph info: subgraph_id -> (node_indices, child_subgraph_ids)
    subgraph_info: HashMap<String, (Vec<usize>, Vec<String>)>,
    /// Subgraph direction overrides: subgraph_id -> direction
    subgraph_directions: HashMap<String, FlowDirection>,
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

            let (text_width, text_height) =
                if adjusted_width + config.node_padding.x * 2.0 > config.max_node_width {
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
            if let (Some(&from_idx), Some(&to_idx)) =
                (id_to_index.get(&edge.from), id_to_index.get(&edge.to))
            {
                outgoing[from_idx].push(to_idx);
                incoming[to_idx].push(from_idx);
            }
        }

        // Build subgraph membership mapping
        // Each node can only be in one subgraph (the innermost one it belongs to)
        let mut node_subgraph: Vec<Option<String>> = vec![None; n];
        let mut subgraph_info: HashMap<String, (Vec<usize>, Vec<String>)> = HashMap::new();
        let mut subgraph_directions: HashMap<String, FlowDirection> = HashMap::new();

        for subgraph in &flowchart.subgraphs {
            let mut node_indices = Vec::new();
            for node_id in &subgraph.node_ids {
                if let Some(&idx) = id_to_index.get(node_id) {
                    node_indices.push(idx);
                    // Only assign to this subgraph if not already in a more nested one
                    // Since subgraphs are parsed children-before-parents, we assign the first one
                    if node_subgraph[idx].is_none() {
                        node_subgraph[idx] = Some(subgraph.id.clone());
                    }
                }
            }
            subgraph_info.insert(
                subgraph.id.clone(),
                (node_indices, subgraph.child_subgraph_ids.clone()),
            );
            // Store direction override if present
            if let Some(direction) = subgraph.direction {
                subgraph_directions.insert(subgraph.id.clone(), direction);
            }
        }

        FlowGraph {
            node_ids,
            id_to_index,
            node_sizes,
            outgoing,
            incoming,
            back_edges: Vec::new(),
            node_subgraph,
            subgraph_info,
            subgraph_directions,
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

/// Result of laying out a subgraph's internal contents.
#[derive(Debug, Clone)]
struct SubgraphInternalLayout {
    /// Positions of nodes relative to subgraph origin (0,0)
    node_positions: HashMap<usize, Pos2>,
    /// Bounding box size (including padding and title)
    bounding_size: Vec2,
    /// Content size (without padding)
    content_size: Vec2,
}

/// Engine for laying out subgraph contents independently.
struct SubgraphLayoutEngine<'a> {
    /// Reference to the full graph
    graph: &'a FlowGraph,
    /// Layout configuration
    config: &'a FlowLayoutConfig,
    /// Flow direction
    direction: FlowDirection,
    /// Available width for layout
    available_width: f32,
}

impl<'a> SubgraphLayoutEngine<'a> {
    fn new(
        graph: &'a FlowGraph,
        config: &'a FlowLayoutConfig,
        direction: FlowDirection,
        available_width: f32,
    ) -> Self {
        Self {
            graph,
            config,
            direction,
            available_width,
        }
    }

    /// Layout a single subgraph's internal contents.
    /// Returns positions relative to (0, 0) origin.
    fn layout_subgraph(
        &self,
        subgraph_id: &str,
        child_subgraph_layouts: &HashMap<String, SubgraphInternalLayout>,
    ) -> Option<SubgraphInternalLayout> {
        let (node_indices, child_ids) = self.graph.subgraph_info.get(subgraph_id)?;

        if node_indices.is_empty() && child_ids.is_empty() {
            return None;
        }

        // Get the effective direction for this subgraph (use override if present)
        let effective_direction = self.graph.subgraph_directions
            .get(subgraph_id)
            .copied()
            .unwrap_or(self.direction);

        // Collect nodes that directly belong to this subgraph (not nested children)
        let direct_nodes: Vec<usize> = node_indices
            .iter()
            .filter(|&&idx| {
                self.graph.node_subgraph.get(idx)
                    .and_then(|s| s.as_ref())
                    .map(|s| s == subgraph_id)
                    .unwrap_or(false)
            })
            .copied()
            .collect();

        // Build set of all nodes in this subgraph (for edge filtering)
        let all_subgraph_nodes: std::collections::HashSet<usize> = node_indices.iter().copied().collect();

        // Find internal edges (both endpoints in subgraph)
        let back_edge_set: std::collections::HashSet<(usize, usize)> =
            self.graph.back_edges.iter().cloned().collect();

        let mut internal_edges: Vec<(usize, usize)> = Vec::new();
        for &from in &direct_nodes {
            if let Some(targets) = self.graph.outgoing.get(from) {
                for &to in targets {
                    if all_subgraph_nodes.contains(&to) && !back_edge_set.contains(&(from, to)) {
                        internal_edges.push((from, to));
                    }
                }
            }
        }

        // If we only have direct nodes (no nested subgraphs), do simple layout
        if child_ids.is_empty() {
            return self.layout_simple_subgraph(&direct_nodes, &internal_edges, effective_direction);
        }

        // Complex case: we have child subgraphs that act as "super-nodes"
        self.layout_hierarchical_subgraph(
            subgraph_id,
            &direct_nodes,
            child_ids,
            child_subgraph_layouts,
            effective_direction,
        )
    }

    /// Layout a subgraph that contains only direct nodes (no nested subgraphs).
    fn layout_simple_subgraph(
        &self,
        nodes: &[usize],
        edges: &[(usize, usize)],
        direction: FlowDirection,
    ) -> Option<SubgraphInternalLayout> {
        if nodes.is_empty() {
            return None;
        }

        let is_horizontal = matches!(
            direction,
            FlowDirection::LeftRight | FlowDirection::RightLeft
        );

        // Assign layers within subgraph
        let layers = self.assign_internal_layers(nodes, edges);

        // Compute positions within subgraph
        let (node_positions, content_size) = self.compute_internal_positions(
            nodes,
            &layers,
            is_horizontal,
        );

        // Add padding and title height for bounding box
        let padding = self.config.subgraph_padding;
        let title_height = self.config.subgraph_title_height;

        let bounding_size = Vec2::new(
            content_size.x + padding * 2.0,
            content_size.y + padding * 2.0 + title_height,
        );

        Some(SubgraphInternalLayout {
            node_positions,
            bounding_size,
            content_size,
        })
    }

    /// Assign layers to nodes within a subgraph using longest-path.
    fn assign_internal_layers(
        &self,
        nodes: &[usize],
        edges: &[(usize, usize)],
    ) -> Vec<Vec<usize>> {
        if nodes.is_empty() {
            return Vec::new();
        }

        let node_set: std::collections::HashSet<usize> = nodes.iter().copied().collect();
        let local_idx: HashMap<usize, usize> = nodes
            .iter()
            .enumerate()
            .map(|(i, &node)| (node, i))
            .collect();

        let n = nodes.len();
        
        // Build in-degree for internal edges
        let mut in_degree = vec![0usize; n];
        for &(from, to) in edges {
            if let (Some(&_from_local), Some(&to_local)) = (local_idx.get(&from), local_idx.get(&to)) {
                in_degree[to_local] += 1;
            }
        }

        // Longest-path layer assignment
        let mut node_layers = vec![0usize; n];
        let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();

        // Start with nodes that have no internal predecessors
        for (local_i, &deg) in in_degree.iter().enumerate() {
            if deg == 0 {
                queue.push_back(local_i);
            }
        }

        // If all nodes have predecessors (cycle), start from first
        if queue.is_empty() && !nodes.is_empty() {
            queue.push_back(0);
        }

        // Safety limit to prevent infinite loops on malformed input
        let max_iterations = n * n + 100;
        let mut iteration = 0;
        
        while let Some(local_i) = queue.pop_front() {
            iteration += 1;
            if iteration > max_iterations {
                // Malformed graph causing infinite loop - return simple single-layer layout
                return vec![nodes.to_vec()];
            }
            
            let node = nodes[local_i];
            let current_layer = node_layers[local_i];

            for &(from, to) in edges {
                if from == node {
                    if let Some(&to_local) = local_idx.get(&to) {
                        if node_set.contains(&to) {
                            node_layers[to_local] = node_layers[to_local].max(current_layer + 1);
                            
                            in_degree[to_local] = in_degree[to_local].saturating_sub(1);
                            if in_degree[to_local] == 0 {
                                queue.push_back(to_local);
                            }
                        }
                    }
                }
            }
        }

        // Build layers structure
        let max_layer = node_layers.iter().copied().max().unwrap_or(0);
        let mut layers: Vec<Vec<usize>> = vec![Vec::new(); max_layer + 1];
        
        for (local_i, &layer) in node_layers.iter().enumerate() {
            layers[layer].push(nodes[local_i]);
        }

        layers
    }

    /// Compute positions for nodes within a subgraph.
    /// Returns (node_positions, content_size).
    fn compute_internal_positions(
        &self,
        _nodes: &[usize],
        layers: &[Vec<usize>],
        is_horizontal: bool,
    ) -> (HashMap<usize, Pos2>, Vec2) {
        let mut positions: HashMap<usize, Pos2> = HashMap::new();
        
        if layers.is_empty() {
            return (positions, Vec2::ZERO);
        }

        let spacing = &self.config.node_spacing;
        let padding = self.config.subgraph_padding;
        let title_height = self.config.subgraph_title_height;
        
        // Calculate layer sizes
        let mut layer_main_sizes: Vec<f32> = Vec::new();
        let mut layer_cross_sizes: Vec<f32> = Vec::new();

        for layer in layers {
            let main_size: f32 = layer
                .iter()
                .map(|&idx| {
                    let size = self.graph.node_sizes[idx];
                    if is_horizontal { size.x } else { size.y }
                })
                .fold(0.0_f32, f32::max);
            
            let cross_size: f32 = layer
                .iter()
                .map(|&idx| {
                    let size = self.graph.node_sizes[idx];
                    if is_horizontal { size.y } else { size.x }
                })
                .sum::<f32>()
                + (layer.len().saturating_sub(1)) as f32 
                    * if is_horizontal { spacing.y } else { spacing.x };
            
            layer_main_sizes.push(main_size);
            layer_cross_sizes.push(cross_size);
        }

        let max_cross_size = layer_cross_sizes.iter().copied().fold(0.0_f32, f32::max);

        // Position nodes layer by layer
        // Start position accounts for padding and title
        let mut current_main = padding + title_height;
        let mut max_extent = Vec2::ZERO;

        for (layer_idx, layer) in layers.iter().enumerate() {
            let layer_cross = layer_cross_sizes[layer_idx];
            let start_cross = padding + (max_cross_size - layer_cross) / 2.0;
            let mut current_cross = start_cross;

            for &node_idx in layer {
                let size = self.graph.node_sizes[node_idx];
                
                let pos = if is_horizontal {
                    Pos2::new(current_main, current_cross)
                } else {
                    Pos2::new(current_cross, current_main)
                };
                
                positions.insert(node_idx, pos);
                
                max_extent.x = max_extent.x.max(pos.x + size.x);
                max_extent.y = max_extent.y.max(pos.y + size.y);

                current_cross += if is_horizontal {
                    size.y + spacing.y
                } else {
                    size.x + spacing.x
                };
            }

            current_main += layer_main_sizes[layer_idx]
                + if is_horizontal { spacing.x } else { spacing.y };
        }

        // Content size is the extent minus the padding/title we added
        let content_size = Vec2::new(
            max_extent.x - padding,
            max_extent.y - padding - title_height,
        );

        (positions, content_size.max(Vec2::ZERO))
    }

    /// Layout a subgraph that contains nested child subgraphs.
    fn layout_hierarchical_subgraph(
        &self,
        _subgraph_id: &str,
        direct_nodes: &[usize],
        child_ids: &[String],
        child_layouts: &HashMap<String, SubgraphInternalLayout>,
        direction: FlowDirection,
    ) -> Option<SubgraphInternalLayout> {
        // For hierarchical layouts, we treat child subgraphs as large "virtual nodes"
        // and layout them alongside direct nodes.

        // Collect sizes: direct nodes + child subgraph bounding boxes
        let mut all_sizes: Vec<(usize, Vec2, bool)> = Vec::new(); // (id, size, is_child_subgraph)
        
        for &node_idx in direct_nodes {
            all_sizes.push((node_idx, self.graph.node_sizes[node_idx], false));
        }
        
        // For now, simplified approach: layout all items in a single column/row
        // A full hierarchical layout would need to consider edges between items
        let is_horizontal = matches!(
            direction,
            FlowDirection::LeftRight | FlowDirection::RightLeft
        );
        
        let spacing = &self.config.node_spacing;
        let padding = self.config.subgraph_padding;
        let title_height = self.config.subgraph_title_height;
        
        let mut positions: HashMap<usize, Pos2> = HashMap::new();
        let mut current_main = padding + title_height;
        let mut max_cross: f32 = 0.0;
        
        // First pass: compute max cross size
        for &node_idx in direct_nodes {
            let size = self.graph.node_sizes[node_idx];
            let cross = if is_horizontal { size.y } else { size.x };
            max_cross = max_cross.max(cross);
        }
        for child_id in child_ids {
            if let Some(child_layout) = child_layouts.get(child_id) {
                let cross = if is_horizontal { 
                    child_layout.bounding_size.y 
                } else { 
                    child_layout.bounding_size.x 
                };
                max_cross = max_cross.max(cross);
            }
        }

        // Second pass: position items
        for &node_idx in direct_nodes {
            let size = self.graph.node_sizes[node_idx];
            let cross = if is_horizontal { size.y } else { size.x };
            let offset_cross = padding + (max_cross - cross) / 2.0;
            
            let pos = if is_horizontal {
                Pos2::new(current_main, offset_cross)
            } else {
                Pos2::new(offset_cross, current_main)
            };
            
            positions.insert(node_idx, pos);
            
            current_main += if is_horizontal { size.x } else { size.y };
            current_main += if is_horizontal { spacing.x } else { spacing.y };
        }
        
        // Position child subgraphs (their internal positions will be offset later)
        // For now, we store them but the actual positioning happens during global layout
        
        // Calculate content size
        let content_size = if is_horizontal {
            Vec2::new(current_main - padding - title_height, max_cross)
        } else {
            Vec2::new(max_cross, current_main - padding - title_height)
        };
        
        let bounding_size = Vec2::new(
            content_size.x + padding * 2.0,
            content_size.y + padding * 2.0 + title_height,
        );
        
        Some(SubgraphInternalLayout {
            node_positions: positions,
            bounding_size,
            content_size,
        })
    }
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

    /// Run the complete layout algorithm with subgraph-aware positioning.
    fn compute(mut self) -> FlowchartLayout {
        if self.graph.node_count() == 0 {
            return FlowchartLayout::default();
        }

        // Step 0: Layout subgraphs inside-out and compute their bounding boxes
        let subgraph_layouts = self.layout_subgraphs_inside_out();

        // Store original node sizes before replacing with super-node sizes
        let original_sizes = self.graph.node_sizes.clone();

        // Step 1: Detect cycles and mark back-edges
        self.detect_cycles_and_mark_back_edges();

        // Step 2: Assign layers using longest-path algorithm
        self.assign_layers();

        // Step 3: Build initial layer structure
        self.build_layers();

        // Step 4: Reduce edge crossings
        self.reduce_crossings();

        // Step 5: Assign coordinates (using original sizes for actual placement)
        self.graph.node_sizes = original_sizes;
        self.assign_coordinates_with_subgraphs(&subgraph_layouts)
    }

    /// Layout all subgraphs from innermost to outermost.
    /// Returns a map of subgraph_id -> SubgraphInternalLayout.
    fn layout_subgraphs_inside_out(&mut self) -> HashMap<String, SubgraphInternalLayout> {
        let mut layouts: HashMap<String, SubgraphInternalLayout> = HashMap::new();

        if self.graph.subgraph_info.is_empty() {
            return layouts;
        }

        // Build subgraph hierarchy: determine processing order (innermost first)
        // Subgraphs without children should be processed first
        let subgraph_order = self.get_subgraph_processing_order();

        let engine = SubgraphLayoutEngine::new(
            &self.graph,
            &self.config,
            self.direction,
            self.available_width,
        );

        for subgraph_id in &subgraph_order {
            if let Some(layout) = engine.layout_subgraph(subgraph_id, &layouts) {
                layouts.insert(subgraph_id.clone(), layout);
            }
        }

        layouts
    }

    /// Get subgraph IDs in processing order (children before parents).
    fn get_subgraph_processing_order(&self) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        let mut processed: std::collections::HashSet<String> = std::collections::HashSet::new();
        let subgraph_ids: Vec<String> = self.graph.subgraph_info.keys().cloned().collect();
        
        fn process_subgraph(
            id: &str,
            info: &HashMap<String, (Vec<usize>, Vec<String>)>,
            result: &mut Vec<String>,
            processed: &mut std::collections::HashSet<String>,
        ) {
            if processed.contains(id) {
                return;
            }
            
            // First process all children
            if let Some((_, child_ids)) = info.get(id) {
                for child_id in child_ids {
                    process_subgraph(child_id, info, result, processed);
                }
            }
            
            // Then add this subgraph
            result.push(id.to_string());
            processed.insert(id.to_string());
        }

        for id in &subgraph_ids {
            process_subgraph(id, &self.graph.subgraph_info, &mut result, &mut processed);
        }

        result
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

    /// Assign layers using longest-path algorithm with subgraph awareness.
    /// Nodes with no incoming edges (ignoring back-edges) go to layer 0,
    /// others are placed at max(predecessor_layer) + 1.
    /// 
    /// Subgraph-aware clustering ensures nodes in the same subgraph are
    /// assigned to consecutive layers, keeping them grouped together.
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

        // Phase 1: Standard longest-path layer assignment
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

        // Phase 2: Cluster subgraph nodes to consecutive layers
        self.cluster_subgraph_layers(&back_edge_set);
    }

    /// Adjust layer assignments to keep subgraph nodes in consecutive layers.
    /// 
    /// For each subgraph:
    /// 1. Find the layer range [min_layer, max_layer] of its nodes
    /// 2. Compute relative layers within the subgraph based on internal edges
    /// 3. Reassign layers so subgraph content spans consecutive layers starting at min_layer
    fn cluster_subgraph_layers(&mut self, back_edge_set: &std::collections::HashSet<(usize, usize)>) {
        // Process subgraphs from innermost to outermost (children first)
        // The subgraphs vector in the parser is built with children before parents
        let subgraph_ids: Vec<String> = self.graph.subgraph_info.keys().cloned().collect();
        
        for subgraph_id in &subgraph_ids {
            if let Some((node_indices, _)) = self.graph.subgraph_info.get(subgraph_id) {
                if node_indices.is_empty() {
                    continue;
                }

                // Find all nodes that belong to this subgraph (directly, not nested)
                let subgraph_nodes: Vec<usize> = node_indices
                    .iter()
                    .filter(|&&idx| {
                        self.graph.node_subgraph.get(idx)
                            .and_then(|s| s.as_ref())
                            .map(|s| s == subgraph_id)
                            .unwrap_or(false)
                    })
                    .copied()
                    .collect();

                if subgraph_nodes.is_empty() {
                    continue;
                }

                // Find the minimum layer among subgraph nodes (anchor point)
                let min_layer = subgraph_nodes
                    .iter()
                    .map(|&idx| self.node_layers[idx])
                    .min()
                    .unwrap_or(0);

                // Compute relative layers within the subgraph
                // using internal edges only
                let relative_layers = self.compute_subgraph_relative_layers(
                    &subgraph_nodes,
                    back_edge_set,
                );

                // Reassign layers: min_layer + relative_layer
                for (&node_idx, &rel_layer) in subgraph_nodes.iter().zip(relative_layers.iter()) {
                    self.node_layers[node_idx] = min_layer + rel_layer;
                }
            }
        }

        // After clustering subgraphs, we may need to push down nodes that 
        // come after subgraphs to avoid overlap
        self.ensure_layer_constraints();
    }

    /// Compute relative layers for nodes within a subgraph based on internal edges.
    /// Returns a vector of relative layer assignments (0-indexed).
    fn compute_subgraph_relative_layers(
        &self,
        nodes: &[usize],
        back_edge_set: &std::collections::HashSet<(usize, usize)>,
    ) -> Vec<usize> {
        let n = nodes.len();
        if n == 0 {
            return Vec::new();
        }

        // Build a set of nodes in this subgraph for quick lookup
        let node_set: std::collections::HashSet<usize> = nodes.iter().copied().collect();
        
        // Build local index mapping: node_idx -> local_idx (0..n)
        let local_idx: HashMap<usize, usize> = nodes
            .iter()
            .enumerate()
            .map(|(i, &node)| (node, i))
            .collect();

        // Compute in-degree for internal edges only
        let mut in_degree = vec![0usize; n];
        for &node in nodes {
            for &pred in &self.graph.incoming[node] {
                if node_set.contains(&pred) && !back_edge_set.contains(&(pred, node)) {
                    in_degree[local_idx[&node]] += 1;
                }
            }
        }

        // Longest-path layer assignment within subgraph
        let mut relative_layers = vec![0usize; n];
        let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();

        // Start with nodes that have no internal predecessors
        for (local_i, &deg) in in_degree.iter().enumerate() {
            if deg == 0 {
                queue.push_back(local_i);
            }
        }

        // If all nodes have internal predecessors (cycle), start from first
        if queue.is_empty() && !nodes.is_empty() {
            queue.push_back(0);
        }

        while let Some(local_i) = queue.pop_front() {
            let node = nodes[local_i];
            let current_layer = relative_layers[local_i];

            for &succ in &self.graph.outgoing[node] {
                if let Some(&succ_local) = local_idx.get(&succ) {
                    if !back_edge_set.contains(&(node, succ)) {
                        relative_layers[succ_local] = relative_layers[succ_local].max(current_layer + 1);
                        
                        in_degree[succ_local] = in_degree[succ_local].saturating_sub(1);
                        if in_degree[succ_local] == 0 {
                            queue.push_back(succ_local);
                        }
                    }
                }
            }
        }

        relative_layers
    }

    /// Ensure layer constraints are satisfied after subgraph clustering.
    /// If a node's predecessor is in a later or equal layer, push the node down.
    fn ensure_layer_constraints(&mut self) {
        let n = self.graph.node_count();
        let back_edge_set: std::collections::HashSet<(usize, usize)> =
            self.graph.back_edges.iter().cloned().collect();

        // Multiple passes to propagate constraints
        for _ in 0..n {
            let mut changed = false;
            for node in 0..n {
                for &pred in &self.graph.incoming[node] {
                    if !back_edge_set.contains(&(pred, node)) {
                        let min_layer = self.node_layers[pred] + 1;
                        if self.node_layers[node] < min_layer {
                            self.node_layers[node] = min_layer;
                            changed = true;
                        }
                    }
                }
            }
            if !changed {
                break;
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
        // Mermaid convention: FIRST-declared edge target goes LEFT
        for layer in &mut self.layers {
            // Sort by position in outgoing edge list of predecessor
            // LOWER position = earlier in edge declarations = goes LEFT
            layer.sort_by(|&a, &b| {
                let pos_a = all_edge_positions.get(&a).copied().unwrap_or(a);
                let pos_b = all_edge_positions.get(&b).copied().unwrap_or(b);
                pos_a.cmp(&pos_b) // Natural order: lower position first
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
        if min_pos == usize::MAX {
            node
        } else {
            min_pos
        }
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

        // Build current position map to preserve order for unconnected nodes
        let current_positions: HashMap<usize, usize> = self.layers[layer_idx]
            .iter()
            .enumerate()
            .map(|(pos, &node)| (node, pos))
            .collect();

        // Calculate barycenter for each node in current layer
        // Store: (node_index, barycenter) - we'll use edge position as tiebreaker
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
                // No connections in this direction - preserve current layer position
                // This maintains the edge declaration order established earlier
                current_positions.get(&node).copied().unwrap_or(node) as f32
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
                    // Neighbors exist but not in adjacent layer - preserve current position
                    current_positions.get(&node).copied().unwrap_or(node) as f32
                }
            };

            barycenters.push((node, barycenter));
        }

        // Sort by barycenter, with edge position as tiebreaker
        // Mermaid convention: FIRST-declared edge target goes LEFT
        let edge_positions: HashMap<usize, usize> = barycenters
            .iter()
            .map(|&(node, _)| (node, self.get_min_incoming_edge_position(node)))
            .collect();

        barycenters.sort_by(|a, b| match a.1.partial_cmp(&b.1) {
            Some(std::cmp::Ordering::Equal) | None => {
                // LOWER position = earlier declared = goes left
                let pos_a = edge_positions.get(&a.0).copied().unwrap_or(a.0);
                let pos_b = edge_positions.get(&b.0).copied().unwrap_or(b.0);
                pos_a.cmp(&pos_b) // Natural order: lower position first
            }
            Some(ord) => ord,
        });

        // Update layer order
        self.layers[layer_idx] = barycenters.into_iter().map(|(node, _)| node).collect();
    }

    /// Assign final coordinates to all nodes.
    /// 
    /// Note: Subgraph layouts are computed separately in `compute_subgraph_layouts`
    /// after this method runs. The internal layouts computed earlier are available
    /// for future enhancements like edge routing within subgraphs.
    fn assign_coordinates_with_subgraphs(
        self,
        _subgraph_layouts: &HashMap<String, SubgraphInternalLayout>,
    ) -> FlowchartLayout {
        let is_horizontal =
            matches!(self.direction, FlowDirection::LeftRight | FlowDirection::RightLeft);
        let is_reversed =
            matches!(self.direction, FlowDirection::BottomUp | FlowDirection::RightLeft);

        let mut layout = FlowchartLayout::default();
        let margin = self.config.margin;

        // Calculate the maximum cross-axis size for centering
        let mut layer_cross_sizes: Vec<f32> = Vec::new();
        for layer in &self.layers {
            let mut size: f32 = 0.0;
            for &node_idx in layer {
                let node_size = self.graph.node_sizes[node_idx];
                size += if is_horizontal {
                    node_size.y
                } else {
                    node_size.x
                };
            }
            size += (layer.len().saturating_sub(1)) as f32
                * if is_horizontal {
                    self.config.node_spacing.y
                } else {
                    self.config.node_spacing.x
                };
            layer_cross_sizes.push(size);
        }
        let max_cross_size = layer_cross_sizes.iter().copied().fold(0.0_f32, f32::max);

        // Calculate layer main-axis sizes (for positioning)
        let layer_main_sizes: Vec<f32> = self
            .layers
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
                + if is_horizontal {
                    self.config.node_spacing.x
                } else {
                    self.config.node_spacing.y
                };
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
        flowchart
            .edges
            .iter()
            .enumerate()
            .filter_map(|(idx, edge)| {
                edge.label.as_ref().map(|label| {
                    // Calculate max label width based on edge geometry
                    let (from_layout, to_layout) =
                        match (layout.nodes.get(&edge.from), layout.nodes.get(&edge.to)) {
                            (Some(f), Some(t)) => (f, t),
                            _ => return None,
                        };
                    let from_center = from_layout.pos + from_layout.size / 2.0;
                    let to_center = to_layout.pos + to_layout.size / 2.0;
                    let edge_length = ((to_center.x - from_center.x).powi(2)
                        + (to_center.y - from_center.y).powi(2))
                    .sqrt();
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
    let (response, painter) = ui.allocate_painter(layout.total_size, egui::Sense::hover());
    let offset = response.rect.min.to_vec2();

    // Compute actual nesting depth for each subgraph
    // Depth 0 = top-level, depth 1 = child of top-level, etc.
    let subgraph_depths: HashMap<String, usize> = compute_subgraph_depths(flowchart);

    // Draw subgraphs first (behind everything else)
    // Draw in reverse order so parent subgraphs are behind children
    for subgraph in flowchart.subgraphs.iter().rev() {
        if let Some(sg_layout) = layout.subgraphs.get(&subgraph.id) {
            let depth = subgraph_depths.get(&subgraph.id).copied().unwrap_or(0);
            draw_subgraph(&painter, sg_layout, offset, colors, font_size, depth);
        }
    }

    // Draw edges (behind nodes but above subgraphs)
    for (idx, edge) in flowchart.edges.iter().enumerate() {
        if let (Some(from_layout), Some(to_layout)) =
            (layout.nodes.get(&edge.from), layout.nodes.get(&edge.to))
        {
            let label_info = edge_labels.get(&idx);
            let is_back_edge = layout
                .back_edges
                .contains(&(edge.from.clone(), edge.to.clone()));
            draw_edge(
                &painter,
                edge,
                idx,
                from_layout,
                to_layout,
                offset,
                colors,
                label_font_size,
                flowchart.direction,
                label_info,
                is_back_edge,
                flowchart,
                &layout.subgraphs,
            );
        }
    }

    // Draw nodes (on top)
    for node in &flowchart.nodes {
        if let Some(node_layout) = layout.nodes.get(&node.id) {
            // Look up custom style for this node
            let custom_style = flowchart
                .node_classes
                .get(&node.id)
                .and_then(|class_name| flowchart.class_defs.get(class_name));
            draw_node(&painter, node, node_layout, offset, colors, font_size, custom_style);
        }
    }
}

/// Compute nesting depth for each subgraph.
/// Depth 0 = top-level subgraph, depth 1 = child of top-level, etc.
fn compute_subgraph_depths(flowchart: &Flowchart) -> HashMap<String, usize> {
    let mut depths: HashMap<String, usize> = HashMap::new();
    
    // Build parent mapping: child_id -> parent_id
    let mut parent_map: HashMap<String, String> = HashMap::new();
    for subgraph in &flowchart.subgraphs {
        for child_id in &subgraph.child_subgraph_ids {
            parent_map.insert(child_id.clone(), subgraph.id.clone());
        }
    }
    
    // Compute depth for each subgraph by counting ancestors
    for subgraph in &flowchart.subgraphs {
        let mut depth = 0;
        let mut current_id = subgraph.id.clone();
        
        // Walk up the parent chain
        while let Some(parent_id) = parent_map.get(&current_id) {
            depth += 1;
            current_id = parent_id.clone();
        }
        
        depths.insert(subgraph.id.clone(), depth);
    }
    
    depths
}

fn draw_subgraph(
    painter: &egui::Painter,
    layout: &SubgraphLayout,
    offset: Vec2,
    colors: &FlowchartColors,
    font_size: f32,
    depth: usize,
) {
    let rect = Rect::from_min_size(layout.pos + offset, layout.size);

    // Use alternating fill colors for nested subgraphs
    let fill_color = if depth % 2 == 0 {
        colors.subgraph_fill
    } else {
        colors.subgraph_fill_alt
    };

    // Draw visible background with rounded corners and thicker stroke
    painter.rect(
        rect,
        Rounding::same(8.0),
        fill_color,
        Stroke::new(2.0, colors.subgraph_stroke),
    );

    // Draw prominent title if present
    if let Some(title) = &layout.title {
        let title_pos = Pos2::new(rect.left() + 12.0, rect.top() + 8.0);
        painter.text(
            title_pos,
            egui::Align2::LEFT_TOP,
            title,
            FontId::proportional(font_size),
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
    custom_style: Option<&NodeStyle>,
) {
    let rect = Rect::from_min_size(layout.pos + offset, layout.size);
    let center = rect.center();
    
    // Determine colors and stroke, using custom style if available
    let fill_color = custom_style
        .and_then(|s| s.fill)
        .unwrap_or(colors.node_fill);
    let stroke_color = custom_style
        .and_then(|s| s.stroke)
        .unwrap_or(colors.node_stroke);
    let stroke_width = custom_style
        .and_then(|s| s.stroke_width)
        .unwrap_or(2.0);
    let stroke = Stroke::new(stroke_width, stroke_color);

    // For diamond and circle, also check custom fill
    let diamond_fill = custom_style
        .and_then(|s| s.fill)
        .unwrap_or(colors.diamond_fill);
    let circle_fill = custom_style
        .and_then(|s| s.fill)
        .unwrap_or(colors.circle_fill);

    match node.shape {
        NodeShape::Rectangle | NodeShape::Subroutine => {
            painter.rect(rect, Rounding::same(4.0), fill_color, stroke);
            if matches!(node.shape, NodeShape::Subroutine) {
                // Draw double vertical lines
                let inset = 8.0;
                painter.line_segment(
                    [
                        Pos2::new(rect.left() + inset, rect.top()),
                        Pos2::new(rect.left() + inset, rect.bottom()),
                    ],
                    stroke,
                );
                painter.line_segment(
                    [
                        Pos2::new(rect.right() - inset, rect.top()),
                        Pos2::new(rect.right() - inset, rect.bottom()),
                    ],
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
            painter.rect(rect, rounding, fill_color, stroke);
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
                diamond_fill,
                stroke,
            ));
        }
        NodeShape::Circle => {
            let radius = layout.size.x.min(layout.size.y) / 2.0;
            painter.circle(center, radius, circle_fill, stroke);
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
                fill_color,
                stroke,
            ));
        }
        NodeShape::Cylinder => {
            // Simplified cylinder as rounded rect with ellipse hints
            painter.rect(rect, Rounding::same(4.0), fill_color, stroke);
            let ellipse_height = 8.0;
            painter.line_segment(
                [
                    Pos2::new(rect.left(), rect.top() + ellipse_height),
                    Pos2::new(rect.right(), rect.top() + ellipse_height),
                ],
                Stroke::new(1.0, stroke_color.gamma_multiply(0.5)),
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
                fill_color,
                stroke,
            ));
        }
        NodeShape::Asymmetric => {
            // Asymmetric shape: flag/banner pointing left
            // Notch depth is proportional to height for consistent appearance
            let indent = layout.size.y * 0.25;
            let points = [
                Pos2::new(rect.left() + indent, rect.top()),
                Pos2::new(rect.right(), rect.top()),
                Pos2::new(rect.right(), rect.bottom()),
                Pos2::new(rect.left() + indent, rect.bottom()),
                Pos2::new(rect.left(), center.y),
            ];
            painter.add(egui::Shape::convex_polygon(
                points.to_vec(),
                fill_color,
                stroke,
            ));
        }
    }

    // Draw text - offset for asymmetric shape to center in visible area
    let text_center = if matches!(node.shape, NodeShape::Asymmetric) {
        // Offset text to the right by half the indent to center within visible portion
        let indent = layout.size.y * 0.25;
        Pos2::new(center.x + indent / 2.0, center.y)
    } else {
        center
    };
    
    painter.text(
        text_center,
        egui::Align2::CENTER_CENTER,
        &node.label,
        FontId::proportional(font_size),
        colors.node_text,
    );
}

fn draw_edge(
    painter: &egui::Painter,
    edge: &FlowEdge,
    edge_index: usize,
    from_layout: &NodeLayout,
    to_layout: &NodeLayout,
    offset: Vec2,
    colors: &FlowchartColors,
    label_font_size: f32,
    direction: FlowDirection,
    label_info: Option<&EdgeLabelInfo>,
    is_back_edge: bool,
    flowchart: &Flowchart,
    subgraph_layouts: &HashMap<String, SubgraphLayout>,
) {
    let from_rect = Rect::from_min_size(from_layout.pos + offset, from_layout.size);
    let to_rect = Rect::from_min_size(to_layout.pos + offset, to_layout.size);

    // Get custom link style (specific index takes precedence over default)
    let link_style = flowchart
        .link_styles
        .get(&edge_index)
        .or(flowchart.default_link_style.as_ref());

    // Edge style - base width from edge type
    let base_stroke_width = match edge.style {
        EdgeStyle::Solid => 2.0,
        EdgeStyle::Dotted => 1.5,
        EdgeStyle::Thick => 3.0,
    };

    // Apply custom stroke width if specified
    let stroke_width = link_style
        .and_then(|s| s.stroke_width)
        .unwrap_or(base_stroke_width);

    // Apply custom stroke color if specified
    let stroke_color = link_style
        .and_then(|s| s.stroke)
        .unwrap_or(colors.edge_stroke);

    let stroke = Stroke::new(stroke_width, stroke_color);

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
            draw_arrow_head(
                painter,
                ctrl2,
                end,
                &edge.arrow_end,
                stroke_color,
                stroke_width,
            );
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

        // Check for subgraph boundary crossing
        // Note: start/end already have offset applied, so pass the same offset for subgraph rects
        let crossing_info = get_subgraph_crossing_info(
            &edge.from,
            &edge.to,
            start,
            end,
            flowchart,
            subgraph_layouts,
            offset,
        );

        // Determine the path segments to draw
        let (path_segments, label_mid) = if let Some(info) = &crossing_info {
            // Route through subgraph boundaries using orthogonal routing
            let mut segments: Vec<(Pos2, Pos2)> = Vec::new();
            let mut waypoints: Vec<Pos2> = vec![start];
            
            // Add exit point from source subgraph
            if let Some(exit) = info.exit_point {
                // Use orthogonal routing: go perpendicular first, then toward exit
                match direction {
                    FlowDirection::TopDown | FlowDirection::BottomUp => {
                        // Add intermediate point for cleaner routing
                        let mid_y = (start.y + exit.y) / 2.0;
                        if (start.x - exit.x).abs() > 5.0 {
                            waypoints.push(Pos2::new(start.x, mid_y));
                            waypoints.push(Pos2::new(exit.x, mid_y));
                        }
                    }
                    FlowDirection::LeftRight | FlowDirection::RightLeft => {
                        let mid_x = (start.x + exit.x) / 2.0;
                        if (start.y - exit.y).abs() > 5.0 {
                            waypoints.push(Pos2::new(mid_x, start.y));
                            waypoints.push(Pos2::new(mid_x, exit.y));
                        }
                    }
                }
                waypoints.push(exit);
            }
            
            // Add entry point to target subgraph
            if let Some(entry) = info.entry_point {
                let last = *waypoints.last().unwrap_or(&start);
                match direction {
                    FlowDirection::TopDown | FlowDirection::BottomUp => {
                        if (last.x - entry.x).abs() > 5.0 {
                            let mid_y = (last.y + entry.y) / 2.0;
                            waypoints.push(Pos2::new(last.x, mid_y));
                            waypoints.push(Pos2::new(entry.x, mid_y));
                        }
                    }
                    FlowDirection::LeftRight | FlowDirection::RightLeft => {
                        if (last.y - entry.y).abs() > 5.0 {
                            let mid_x = (last.x + entry.x) / 2.0;
                            waypoints.push(Pos2::new(mid_x, last.y));
                            waypoints.push(Pos2::new(mid_x, entry.y));
                        }
                    }
                }
                waypoints.push(entry);
            }
            
            waypoints.push(end);
            
            // Build segments from waypoints
            for i in 0..waypoints.len() - 1 {
                segments.push((waypoints[i], waypoints[i + 1]));
            }
            
            // Calculate label position (midpoint of the path)
            let total_len: f32 = segments.iter().map(|(a, b)| (*b - *a).length()).sum();
            let mut accumulated = 0.0;
            let target_len = total_len / 2.0;
            let mut mid = Pos2::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0);
            
            for (a, b) in &segments {
                let seg_len = (*b - *a).length();
                if accumulated + seg_len >= target_len {
                    let t = (target_len - accumulated) / seg_len;
                    mid = *a + (*b - *a) * t;
                    break;
                }
                accumulated += seg_len;
            }
            
            (segments, mid)
        } else {
            // Simple direct line
            (vec![(start, end)], Pos2::new((start.x + end.x) / 2.0, (start.y + end.y) / 2.0))
        };

        // Draw all path segments
        for (seg_start, seg_end) in &path_segments {
            if matches!(edge.style, EdgeStyle::Dotted) {
                draw_dashed_line(painter, *seg_start, *seg_end, stroke, 5.0, 3.0);
            } else {
                painter.line_segment([*seg_start, *seg_end], stroke);
            }
        }

        // Draw arrow head at end (use last segment for direction)
        if !matches!(edge.arrow_end, ArrowHead::None) {
            let default_seg = (start, end);
            let last_seg = path_segments.last().unwrap_or(&default_seg);
            draw_arrow_head(
                painter,
                last_seg.0,
                last_seg.1,
                &edge.arrow_end,
                stroke_color,
                stroke_width,
            );
        }

        // Draw arrow head at start (for bidirectional)
        if !matches!(edge.arrow_start, ArrowHead::None) {
            let default_seg = (start, end);
            let first_seg = path_segments.first().unwrap_or(&default_seg);
            draw_arrow_head(
                painter,
                first_seg.1,
                first_seg.0,
                &edge.arrow_start,
                stroke_color,
                stroke_width,
            );
        }

        // Draw edge label using pre-computed sizes
        if let Some(info) = label_info {
            let label_rect = Rect::from_center_size(label_mid, info.size);

            painter.rect_filled(label_rect, Rounding::same(3.0), colors.edge_label_bg);
            painter.text(
                label_mid,
                egui::Align2::CENTER_CENTER,
                &info.display_text,
                FontId::proportional(label_font_size),
                colors.edge_label_text,
            );
        }
    }
}

fn draw_dashed_line(
    painter: &egui::Painter,
    start: Pos2,
    end: Pos2,
    stroke: Stroke,
    dash_len: f32,
    gap_len: f32,
) {
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
fn draw_bezier_curve(
    painter: &egui::Painter,
    p0: Pos2,
    p1: Pos2,
    p2: Pos2,
    p3: Pos2,
    stroke: Stroke,
) {
    let segments = 20; // Number of line segments to approximate the curve
    let mut prev = p0;

    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let curr = bezier_point(p0, p1, p2, p3, t);
        painter.line_segment([prev, curr], stroke);
        prev = curr;
    }
}

fn draw_arrow_head(
    painter: &egui::Painter,
    from: Pos2,
    to: Pos2,
    head_type: &ArrowHead,
    color: Color32,
    stroke_width: f32,
) {
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
                [
                    center - perp * size - dir * size,
                    center + perp * size + dir * size,
                ],
                Stroke::new(stroke_width, color),
            );
            painter.line_segment(
                [
                    center + perp * size - dir * size,
                    center - perp * size + dir * size,
                ],
                Stroke::new(stroke_width, color),
            );
        }
        ArrowHead::None => {}
    }
}

/// Find the innermost subgraph that contains a given node.
/// Returns None if the node is not in any subgraph.
fn find_node_subgraph<'a>(
    node_id: &str,
    flowchart: &'a Flowchart,
) -> Option<&'a FlowSubgraph> {
    // Subgraphs are ordered children-before-parents, so iterate in order
    // to find the innermost (most specific) subgraph first
    for subgraph in &flowchart.subgraphs {
        if subgraph.node_ids.contains(&node_id.to_string()) {
            return Some(subgraph);
        }
    }
    None
}

/// Calculate intersection point of a line segment with a rectangle's border.
/// Returns the intersection point closest to `from` on the way to `to`.
fn line_rect_intersection(from: Pos2, to: Pos2, rect: Rect) -> Option<Pos2> {
    let dir = to - from;
    
    if dir.length_sq() < 0.001 {
        return None;
    }
    
    // Check intersection with all four sides
    let mut best_t: Option<f32> = None;
    
    // Left edge (x = rect.left())
    if dir.x.abs() > 0.001 {
        let t = (rect.left() - from.x) / dir.x;
        if t > 0.0 && t < 1.0 {
            let y = from.y + t * dir.y;
            if y >= rect.top() && y <= rect.bottom() {
                if best_t.is_none() || t < best_t.unwrap() {
                    best_t = Some(t);
                }
            }
        }
    }
    
    // Right edge (x = rect.right())
    if dir.x.abs() > 0.001 {
        let t = (rect.right() - from.x) / dir.x;
        if t > 0.0 && t < 1.0 {
            let y = from.y + t * dir.y;
            if y >= rect.top() && y <= rect.bottom() {
                if best_t.is_none() || t < best_t.unwrap() {
                    best_t = Some(t);
                }
            }
        }
    }
    
    // Top edge (y = rect.top())
    if dir.y.abs() > 0.001 {
        let t = (rect.top() - from.y) / dir.y;
        if t > 0.0 && t < 1.0 {
            let x = from.x + t * dir.x;
            if x >= rect.left() && x <= rect.right() {
                if best_t.is_none() || t < best_t.unwrap() {
                    best_t = Some(t);
                }
            }
        }
    }
    
    // Bottom edge (y = rect.bottom())
    if dir.y.abs() > 0.001 {
        let t = (rect.bottom() - from.y) / dir.y;
        if t > 0.0 && t < 1.0 {
            let x = from.x + t * dir.x;
            if x >= rect.left() && x <= rect.right() {
                if best_t.is_none() || t < best_t.unwrap() {
                    best_t = Some(t);
                }
            }
        }
    }
    
    best_t.map(|t| from + dir * t)
}

/// Information about how an edge crosses subgraph boundaries.
#[derive(Debug, Clone)]
struct SubgraphCrossingInfo {
    /// Entry point into a subgraph (from outside to inside)
    entry_point: Option<Pos2>,
    /// Exit point from a subgraph (from inside to outside)
    exit_point: Option<Pos2>,
}

/// Calculate subgraph boundary crossing information for an edge.
/// Returns crossing info if the edge crosses a subgraph boundary.
fn get_subgraph_crossing_info(
    from_id: &str,
    to_id: &str,
    from_pos: Pos2,
    to_pos: Pos2,
    flowchart: &Flowchart,
    subgraph_layouts: &HashMap<String, SubgraphLayout>,
    offset: Vec2,
) -> Option<SubgraphCrossingInfo> {
    let from_sg = find_node_subgraph(from_id, flowchart);
    let to_sg = find_node_subgraph(to_id, flowchart);
    
    // Check if nodes are in different subgraphs
    let from_sg_id = from_sg.map(|sg| sg.id.as_str());
    let to_sg_id = to_sg.map(|sg| sg.id.as_str());
    
    if from_sg_id == to_sg_id {
        // Same subgraph (or both not in any) - no crossing needed
        return None;
    }
    
    // Case 1: From outside to inside a subgraph
    if from_sg_id.is_none() && to_sg_id.is_some() {
        if let Some(sg_layout) = to_sg_id.and_then(|id| subgraph_layouts.get(id)) {
            let sg_rect = Rect::from_min_size(sg_layout.pos + offset, sg_layout.size);
            if let Some(entry) = line_rect_intersection(from_pos, to_pos, sg_rect) {
                return Some(SubgraphCrossingInfo {
                    entry_point: Some(entry),
                    exit_point: None,
                });
            }
        }
    }
    
    // Case 2: From inside to outside a subgraph
    if from_sg_id.is_some() && to_sg_id.is_none() {
        if let Some(sg_layout) = from_sg_id.and_then(|id| subgraph_layouts.get(id)) {
            let sg_rect = Rect::from_min_size(sg_layout.pos + offset, sg_layout.size);
            if let Some(exit) = line_rect_intersection(from_pos, to_pos, sg_rect) {
                return Some(SubgraphCrossingInfo {
                    entry_point: None,
                    exit_point: Some(exit),
                });
            }
        }
    }
    
    // Case 3: From one subgraph to a different subgraph
    if from_sg_id.is_some() && to_sg_id.is_some() && from_sg_id != to_sg_id {
        let mut exit_point = None;
        let mut entry_point = None;
        
        // Find exit from source subgraph
        if let Some(sg_layout) = from_sg_id.and_then(|id| subgraph_layouts.get(id)) {
            let sg_rect = Rect::from_min_size(sg_layout.pos + offset, sg_layout.size);
            exit_point = line_rect_intersection(from_pos, to_pos, sg_rect);
        }
        
        // Find entry to target subgraph (using exit point as starting position if available)
        if let Some(sg_layout) = to_sg_id.and_then(|id| subgraph_layouts.get(id)) {
            let sg_rect = Rect::from_min_size(sg_layout.pos + offset, sg_layout.size);
            let start = exit_point.unwrap_or(from_pos);
            entry_point = line_rect_intersection(start, to_pos, sg_rect);
        }
        
        if exit_point.is_some() || entry_point.is_some() {
            return Some(SubgraphCrossingInfo {
                entry_point,
                exit_point,
            });
        }
    }
    
    None
}
