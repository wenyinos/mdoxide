//! Minimal SVG export for Mermaid flowcharts (HTML embedding).

use eframe::egui::Color32;

use crate::markdown::mermaid::{
    layout_flowchart, parse_flowchart, ArrowHead, EdgeStyle, EstimatedTextMeasurer, Flowchart,
    FlowchartColors, FlowchartLayout, NodeLayout, NodeShape, NodeStyle,
};

const FONT_SIZE: f32 = 14.0;
const PAD: f32 = 10.0;

fn svg_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn rgba(c: Color32) -> String {
    format!(
        "rgba({},{},{},{})",
        c.r(),
        c.g(),
        c.b(),
        c.a() as f32 / 255.0
    )
}

fn node_center(nl: &NodeLayout) -> (f32, f32) {
    (nl.pos.x + nl.size.x * 0.5, nl.pos.y + nl.size.y * 0.5)
}

fn diamond_path(x: f32, y: f32, w: f32, h: f32) -> String {
    let mx = x + w * 0.5;
    let my = y + h * 0.5;
    let x2 = x + w;
    let y2 = y + h;
    format!("M {mx},{y} L {x2},{my} L {mx},{y2} L {x},{my} Z")
}

fn draw_arrow_head(x1: f32, y1: f32, x2: f32, y2: f32, stroke: &str) -> String {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt().max(0.001);
    let ux = dx / len;
    let uy = dy / len;
    let bx = x2 - ux * 10.0;
    let by = y2 - uy * 10.0;
    let px = -uy * 5.0;
    let py = ux * 5.0;
    let x3 = bx + px;
    let y3 = by + py;
    let x4 = bx - px;
    let y4 = by - py;
    format!(r#"<polygon points="{x2},{y2} {x3},{y3} {x4},{y4}" fill="{stroke}"/>"#,)
}

/// Try to render Mermaid flowchart source as an `<svg>...</svg>` element.
pub fn try_flowchart_svg_snippet(
    source: &str,
    colors: &FlowchartColors,
    width: f32,
) -> Option<String> {
    let source = source.trim();
    if source.is_empty() {
        return None;
    }
    let diagram = crate::markdown::mermaid::strip_frontmatter(source);
    let fc = parse_flowchart(diagram).ok()?;
    if fc.nodes.is_empty() {
        return None;
    }
    let measurer = EstimatedTextMeasurer::new();
    let layout = layout_flowchart(&fc, width, FONT_SIZE, &measurer);
    if layout.total_size.x <= 0.0 || layout.total_size.y <= 0.0 {
        return None;
    }
    Some(flowchart_layout_to_svg(&fc, &layout, colors))
}

fn merged_node_style(fc: &Flowchart, node_id: &str) -> Option<NodeStyle> {
    let class = fc
        .node_classes
        .get(node_id)
        .and_then(|c| fc.class_defs.get(c));
    let inline = fc.node_styles.get(node_id);
    NodeStyle::merge_class_and_inline(class, inline)
}

fn flowchart_layout_to_svg(
    fc: &Flowchart,
    layout: &FlowchartLayout,
    colors: &FlowchartColors,
) -> String {
    let w = layout.total_size.x + PAD * 2.0;
    let h = layout.total_size.y + PAD * 2.0;
    let mut out = String::new();
    out.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" role="img" aria-label="Flowchart">"#
    ));
    out.push_str(&format!(
        r#"<defs><style type="text/css"><![CDATA[.fc-text{{font:{}px system-ui,-apple-system,"Segoe UI",sans-serif;dominant-baseline:middle;}}]]></style></defs>"#,
        FONT_SIZE as i32
    ));
    let ox = PAD;
    let oy = PAD;

    for sg in &fc.subgraphs {
        if let Some(sl) = layout.subgraphs.get(&sg.id) {
            out.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}" stroke-width="1" rx="4"/>"#,
                ox + sl.pos.x,
                oy + sl.pos.y,
                sl.size.x,
                sl.size.y,
                rgba(colors.subgraph_fill),
                rgba(colors.subgraph_stroke),
            ));
            if let Some(ref title) = sl.title {
                out.push_str(&format!(
                    r#"<text class="fc-text" x="{}" y="{}" fill="{}">{}</text>"#,
                    ox + sl.pos.x + 8.0,
                    oy + sl.pos.y + FONT_SIZE + 2.0,
                    rgba(colors.subgraph_title),
                    svg_escape(title),
                ));
            }
        }
    }

    for edge in &fc.edges {
        let (Some(from_l), Some(to_l)) = (layout.nodes.get(&edge.from), layout.nodes.get(&edge.to))
        else {
            continue;
        };
        let (x1, y1) = node_center(from_l);
        let (x2, y2) = node_center(to_l);
        let x1 = ox + x1;
        let y1 = oy + y1;
        let x2 = ox + x2;
        let y2 = oy + y2;
        let stroke = rgba(colors.edge_stroke);
        let dash = match edge.style {
            EdgeStyle::Dotted => " stroke-dasharray=\"4 4\"",
            _ => "",
        };
        let sw = match edge.style {
            EdgeStyle::Thick => 3.0,
            _ => 1.8,
        };
        out.push_str(&format!(
            r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{stroke}" stroke-width="{sw}"{dash}"/>"#,
        ));
        if !matches!(edge.arrow_end, ArrowHead::None) {
            out.push_str(&draw_arrow_head(x1, y1, x2, y2, &stroke));
        }
        if let Some(label) = &edge.label {
            let mx = (x1 + x2) * 0.5;
            let my = (y1 + y2) * 0.5;
            out.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" rx="3"/>"#,
                mx - 40.0,
                my - 10.0,
                80.0,
                20.0,
                rgba(colors.edge_label_bg),
            ));
            out.push_str(&format!(
                r#"<text class="fc-text" text-anchor="middle" x="{mx}" y="{my}" fill="{}">{}</text>"#,
                rgba(colors.edge_label_text),
                svg_escape(label),
            ));
        }
    }

    for node in &fc.nodes {
        let Some(nl) = layout.nodes.get(&node.id) else {
            continue;
        };
        let x = ox + nl.pos.x;
        let y = oy + nl.pos.y;
        let nw = nl.size.x;
        let nh = nl.size.y;
        let merged = merged_node_style(fc, &node.id);
        let base_fill = match node.shape {
            NodeShape::Diamond => colors.diamond_fill,
            NodeShape::Circle | NodeShape::Cylinder | NodeShape::DoubleCircle => colors.circle_fill,
            _ => colors.node_fill,
        };
        let fill_c = merged.as_ref().and_then(|s| s.fill).unwrap_or(base_fill);
        let stroke_c = merged
            .as_ref()
            .and_then(|s| s.stroke)
            .unwrap_or(colors.node_stroke);
        let sw = merged.as_ref().and_then(|s| s.stroke_width).unwrap_or(1.4);
        let text_c = merged
            .as_ref()
            .and_then(|s| s.color)
            .unwrap_or(colors.node_text);
        match node.shape {
            NodeShape::Diamond => {
                let d = diamond_path(x, y, nw, nh);
                out.push_str(&format!(
                    r#"<path d="{}" fill="{}" stroke="{}" stroke-width="{sw}"/>"#,
                    d,
                    rgba(fill_c),
                    rgba(stroke_c),
                ));
            }
            NodeShape::Circle | NodeShape::Cylinder => {
                let rx = nw * 0.5;
                let ry = nh * 0.5;
                let cx = x + rx;
                let cy = y + ry;
                out.push_str(&format!(
                    r#"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{}" stroke="{}" stroke-width="{sw}"/>"#,
                    rgba(fill_c),
                    rgba(stroke_c),
                ));
            }
            NodeShape::DoubleCircle => {
                let rx = nw * 0.5;
                let ry = nh * 0.5;
                let cx = x + rx;
                let cy = y + ry;
                out.push_str(&format!(
                    r#"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" fill="{}" stroke="{}" stroke-width="{sw}"/>"#,
                    rgba(fill_c),
                    rgba(stroke_c),
                ));
                let rx2 = (rx - 4.0).max(rx * 0.55);
                let ry2 = (ry - 4.0).max(ry * 0.55);
                out.push_str(&format!(
                    r#"<ellipse cx="{cx}" cy="{cy}" rx="{rx2}" ry="{ry2}" fill="none" stroke="{}" stroke-width="{sw}"/>"#,
                    rgba(stroke_c),
                ));
            }
            NodeShape::Stadium => {
                let r = (nh * 0.5).min(nw * 0.5);
                out.push_str(&format!(
                    r#"<rect x="{x}" y="{y}" width="{nw}" height="{nh}" rx="{r}" ry="{r}" fill="{}" stroke="{}" stroke-width="{sw}"/>"#,
                    rgba(fill_c),
                    rgba(stroke_c),
                ));
            }
            NodeShape::RoundRect => {
                let r = (nh * 0.22).min(12.0).max(4.0);
                out.push_str(&format!(
                    r#"<rect x="{x}" y="{y}" width="{nw}" height="{nh}" rx="{r}" ry="{r}" fill="{}" stroke="{}" stroke-width="{sw}"/>"#,
                    rgba(fill_c),
                    rgba(stroke_c),
                ));
            }
            NodeShape::Parallelogram => {
                let skew = nw * 0.15;
                let d = format!(
                    "M {},{} L {},{} L {},{} L {},{} Z",
                    x + skew,
                    y,
                    x + nw,
                    y,
                    x + nw - skew,
                    y + nh,
                    x,
                    y + nh
                );
                out.push_str(&format!(
                    r#"<path d="{}" fill="{}" stroke="{}" stroke-width="{sw}"/>"#,
                    d,
                    rgba(fill_c),
                    rgba(stroke_c),
                ));
            }
            NodeShape::Trapezoid => {
                let skew = nw * 0.15;
                let d = format!(
                    "M {},{} L {},{} L {},{} L {},{} Z",
                    x + skew,
                    y,
                    x + nw - skew,
                    y,
                    x + nw,
                    y + nh,
                    x,
                    y + nh
                );
                out.push_str(&format!(
                    r#"<path d="{}" fill="{}" stroke="{}" stroke-width="{sw}"/>"#,
                    d,
                    rgba(fill_c),
                    rgba(stroke_c),
                ));
            }
            NodeShape::TrapezoidInv => {
                let skew = nw * 0.15;
                let d = format!(
                    "M {},{} L {},{} L {},{} L {},{} Z",
                    x,
                    y,
                    x + nw,
                    y,
                    x + nw - skew,
                    y + nh,
                    x + skew,
                    y + nh
                );
                out.push_str(&format!(
                    r#"<path d="{}" fill="{}" stroke="{}" stroke-width="{sw}"/>"#,
                    d,
                    rgba(fill_c),
                    rgba(stroke_c),
                ));
            }
            _ => {
                out.push_str(&format!(
                    r#"<rect x="{x}" y="{y}" width="{nw}" height="{nh}" rx="4" ry="4" fill="{}" stroke="{}" stroke-width="{sw}"/>"#,
                    rgba(fill_c),
                    rgba(stroke_c),
                ));
            }
        }
        let cx = x + nw * 0.5;
        let cy = y + nh * 0.5;
        out.push_str(&format!(
            r#"<text class="fc-text" text-anchor="middle" x="{cx}" y="{cy}" fill="{}">{}</text>"#,
            rgba(text_c),
            svg_escape(&node.label),
        ));
    }
    out.push_str("</svg>");
    out
}
