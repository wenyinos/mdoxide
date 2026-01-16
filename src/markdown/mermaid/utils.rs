//! Shared drawing utilities for Mermaid diagram rendering.

use egui::{Pos2, Stroke};

/// Draw a dashed line from start to end.
pub fn draw_dashed_line(
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

/// Calculate a point on a cubic bezier curve at parameter t (0..1).
#[allow(dead_code)]
pub fn bezier_point(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
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

/// Draw a cubic bezier curve by sampling points.
#[allow(dead_code)]
pub fn draw_bezier_curve(
    painter: &egui::Painter,
    p0: Pos2,
    p1: Pos2,
    p2: Pos2,
    p3: Pos2,
    stroke: Stroke,
) {
    // Sample curve at multiple points
    let segments = 20;
    let mut prev = p0;
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let next = bezier_point(p0, p1, p2, p3, t);
        painter.line_segment([prev, next], stroke);
        prev = next;
    }
}
