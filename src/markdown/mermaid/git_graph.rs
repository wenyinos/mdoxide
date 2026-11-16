//! Git graph diagram parsing and rendering.

use egui::{Color32, FontId, Pos2, Rect, Stroke, Ui, Vec2};
use std::collections::HashMap;

/// Layout direction for a git graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GitDirection {
    #[default]
    TopToBottom,
    LeftToRight,
}

/// Mermaid commit type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GitCommitType {
    #[default]
    Normal,
    Reverse,
    Highlight,
}

/// A commit in a git graph.
#[derive(Debug, Clone)]
pub struct GitCommit {
    pub id: String,
    pub branch: String,
    pub message: Option<String>,
    pub is_merge: bool,
    pub merge_from: Option<String>,
    pub tags: Vec<String>,
    pub commit_type: GitCommitType,
    /// Original commit id for `cherry-pick` commits.
    pub cherry_pick_from: Option<String>,
}

/// A branch in a git graph.
#[derive(Debug, Clone)]
pub struct GitBranch {
    pub name: String,
    pub color_idx: usize,
    /// Optional `order:` attribute controlling lane order.
    pub order: Option<i32>,
}

/// A git graph.
#[derive(Debug, Clone)]
pub struct GitGraph {
    pub commits: Vec<GitCommit>,
    pub branches: Vec<GitBranch>,
    pub direction: GitDirection,
}

/// Parse a git graph from source.
pub fn parse_git_graph(source: &str) -> Result<GitGraph, String> {
    let mut commits: Vec<GitCommit> = Vec::new();
    let mut branches: Vec<GitBranch> = vec![GitBranch {
        name: "main".to_string(),
        color_idx: 0,
        order: None,
    }];
    let mut current_branch = "main".to_string();
    let mut commit_counter = 0;
    let mut direction = GitDirection::TopToBottom;

    for (line_index, line) in source.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

        let line_lower = line.to_lowercase();

        if line_index == 0 || line_lower.starts_with("gitgraph") {
            if line_lower.contains("lr") {
                direction = GitDirection::LeftToRight;
            } else if line_lower.contains("tb") || line_lower.contains("bt") {
                direction = GitDirection::TopToBottom;
            }
            continue;
        }

        // Parse commit
        if line_lower.starts_with("commit") {
            commit_counter += 1;
            let attrs = parse_attributes(&line[6..]);
            let id = attrs
                .iter()
                .rev()
                .find(|(key, _)| key == "id")
                .map(|(_, value)| value.clone())
                .unwrap_or_else(|| format!("c{}", commit_counter));
            let message = attrs
                .iter()
                .rev()
                .find(|(key, _)| key == "msg")
                .map(|(_, value)| value.clone());
            let tags = attrs
                .iter()
                .filter(|(key, _)| key == "tag")
                .map(|(_, value)| value.clone())
                .collect();
            let commit_type = attrs
                .iter()
                .rev()
                .find(|(key, _)| key == "type")
                .map(|(_, value)| parse_commit_type(value))
                .unwrap_or_default();

            commits.push(GitCommit {
                id,
                branch: current_branch.clone(),
                message,
                is_merge: false,
                merge_from: None,
                tags,
                commit_type,
                cherry_pick_from: None,
            });
        }
        // Parse branch creation
        else if line_lower.starts_with("branch") {
            let rest = line[6..].trim();
            let name = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches(':')
                .to_string();
            if name.is_empty() {
                continue;
            }
            let attrs = parse_attributes(rest);
            let order = attrs
                .iter()
                .rev()
                .find(|(key, _)| key == "order")
                .and_then(|(_, value)| value.parse::<i32>().ok());
            if let Some(branch) = branches.iter_mut().find(|branch| branch.name == name) {
                branch.order = order.or(branch.order);
            } else {
                branches.push(GitBranch {
                    name: name.clone(),
                    color_idx: branches.len(),
                    order,
                });
            }
            current_branch = name;
        }
        // Parse checkout
        else if line_lower.starts_with("checkout") {
            let name = line[8..]
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
            if name.is_empty() {
                continue;
            }
            if !branches.iter().any(|branch| branch.name == name) {
                branches.push(GitBranch {
                    name: name.clone(),
                    color_idx: branches.len(),
                    order: None,
                });
            }
            current_branch = name;
        }
        // Parse merge
        else if line_lower.starts_with("merge") {
            commit_counter += 1;
            let rest = line[5..].trim();
            let merge_from = rest.split_whitespace().next().unwrap_or("").to_string();
            if merge_from.is_empty() {
                continue;
            }
            let attrs = parse_attributes(rest);
            let id = attrs
                .iter()
                .rev()
                .find(|(key, _)| key == "id")
                .map(|(_, value)| value.clone())
                .unwrap_or_else(|| format!("m{}", commit_counter));
            let message = attrs
                .iter()
                .rev()
                .find(|(key, _)| key == "msg")
                .map(|(_, value)| value.clone())
                .or_else(|| Some(format!("Merge {}", merge_from)));
            let tags = attrs
                .iter()
                .filter(|(key, _)| key == "tag")
                .map(|(_, value)| value.clone())
                .collect();
            let commit_type = attrs
                .iter()
                .rev()
                .find(|(key, _)| key == "type")
                .map(|(_, value)| parse_commit_type(value))
                .unwrap_or_default();

            commits.push(GitCommit {
                id,
                branch: current_branch.clone(),
                message,
                is_merge: true,
                merge_from: Some(merge_from),
                tags,
                commit_type,
                cherry_pick_from: None,
            });
        }
        // Parse cherry-pick
        else if line_lower.starts_with("cherry-pick") {
            let attrs = parse_attributes(&line[11..]);
            let source_id = attrs
                .iter()
                .rev()
                .find(|(key, _)| key == "id")
                .map(|(_, value)| value.clone());
            let Some(source_id) = source_id else {
                continue;
            };
            let Some(source_commit) = commits
                .iter()
                .find(|commit| commit.id == source_id)
                .cloned()
            else {
                continue;
            };
            commit_counter += 1;
            let id = attrs
                .iter()
                .rev()
                .find(|(key, _)| key == "id")
                .map(|(_, value)| value.clone())
                .unwrap_or_else(|| format!("cp{}", commit_counter));
            // Avoid duplicate ids: suffix the generated id when the source id is reused.
            let id = if commits.iter().any(|commit| commit.id == id) {
                format!("{}-cherry", id)
            } else {
                id
            };
            let message = attrs
                .iter()
                .rev()
                .find(|(key, _)| key == "msg")
                .map(|(_, value)| value.clone())
                .or_else(|| source_commit.message.clone());
            let tags = attrs
                .iter()
                .filter(|(key, _)| key == "tag")
                .map(|(_, value)| value.clone())
                .collect::<Vec<_>>();
            let tags = if tags.is_empty() {
                source_commit.tags.clone()
            } else {
                tags
            };
            let commit_type = attrs
                .iter()
                .rev()
                .find(|(key, _)| key == "type")
                .map(|(_, value)| parse_commit_type(value))
                .unwrap_or(source_commit.commit_type);

            commits.push(GitCommit {
                id,
                branch: current_branch.clone(),
                message,
                is_merge: false,
                merge_from: None,
                tags,
                commit_type,
                cherry_pick_from: Some(source_commit.id),
            });
        }
    }

    if commits.is_empty() {
        return Err("No commits found in git graph".to_string());
    }

    Ok(GitGraph {
        commits,
        branches,
        direction,
    })
}

/// Parse `key: value` and repeated `tag: "..."` attributes.
fn parse_attributes(text: &str) -> Vec<(String, String)> {
    let chars: Vec<char> = text.chars().collect();
    let mut attributes = Vec::new();
    let mut index = 0;

    while index < chars.len() {
        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }
        let key_start = index;
        while index < chars.len()
            && (chars[index].is_ascii_alphanumeric() || matches!(chars[index], '_' | '-'))
        {
            index += 1;
        }
        if key_start == index {
            index += 1;
            continue;
        }
        let key: String = chars[key_start..index].iter().collect::<String>().to_lowercase();

        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }
        if index >= chars.len() || chars[index] != ':' {
            continue;
        }
        index += 1;
        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }
        if index >= chars.len() {
            break;
        }

        let value = if chars[index] == '"' || chars[index] == '\'' {
            let quote = chars[index];
            index += 1;
            let value_start = index;
            while index < chars.len() && chars[index] != quote {
                index += 1;
            }
            let value: String = chars[value_start..index].iter().collect();
            if index < chars.len() {
                index += 1;
            }
            value
        } else {
            let value_start = index;
            while index < chars.len() && !chars[index].is_whitespace() {
                index += 1;
            }
            chars[value_start..index].iter().collect()
        };
        attributes.push((key, value));
    }

    attributes
}

fn parse_commit_type(value: &str) -> GitCommitType {
    match value.trim().to_ascii_lowercase().as_str() {
        "reverse" => GitCommitType::Reverse,
        "highlight" => GitCommitType::Highlight,
        _ => GitCommitType::Normal,
    }
}

/// Render a git graph to the UI.
pub fn render_git_graph(ui: &mut Ui, graph: &GitGraph, dark_mode: bool, font_size: f32) {
    let margin = 30.0_f32;
    let commit_radius = 8.0_f32;
    let commit_spacing = 50.0_f32;
    let branch_spacing = 60.0_f32;
    let label_width = 150.0_f32;
    let direction = graph.direction;

    let branch_colors = if dark_mode {
        vec![
            Color32::from_rgb(100, 180, 100),
            Color32::from_rgb(100, 150, 220),
            Color32::from_rgb(220, 160, 100),
            Color32::from_rgb(180, 100, 180),
            Color32::from_rgb(220, 100, 100),
            Color32::from_rgb(100, 200, 200),
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
    let text_color = if dark_mode {
        Color32::from_rgb(220, 230, 240)
    } else {
        Color32::from_rgb(30, 40, 50)
    };
    let line_bg = if dark_mode {
        Color32::from_rgb(50, 55, 65)
    } else {
        Color32::from_rgb(240, 245, 250)
    };
    let border_color = if dark_mode {
        Color32::WHITE
    } else {
        Color32::BLACK
    };

    // `order:` controls lane order; ties keep declaration order.
    let mut ordered: Vec<(usize, &GitBranch)> = graph.branches.iter().enumerate().collect();
    ordered.sort_by_key(|(index, branch)| (branch.order.unwrap_or(i32::MAX), *index));
    let branch_lane: HashMap<String, usize> = ordered
        .iter()
        .enumerate()
        .map(|(lane, (_, branch))| (branch.name.clone(), lane))
        .collect();

    let commit_count = graph.commits.len().max(1);
    let branch_count = graph.branches.len().max(1);
    let (total_width, total_height) = match direction {
        GitDirection::TopToBottom => (
            margin * 2.0 + label_width + branch_count as f32 * branch_spacing + 120.0,
            margin * 2.0 + commit_count as f32 * commit_spacing + 40.0,
        ),
        GitDirection::LeftToRight => (
            margin * 2.0 + commit_count as f32 * (commit_spacing + 30.0) + 60.0,
            margin * 2.0 + branch_count as f32 * branch_spacing + label_width,
        ),
    };

    let (response, painter) = ui.allocate_painter(
        Vec2::new(total_width.max(300.0), total_height.max(100.0)),
        egui::Sense::hover(),
    );
    let offset = response.rect.min.to_vec2();

    let commit_position = |commit_index: usize, branch: &str| -> Pos2 {
        let lane = branch_lane.get(branch).copied().unwrap_or(0);
        match direction {
            GitDirection::TopToBottom => Pos2::new(
                offset.x + margin + label_width + lane as f32 * branch_spacing,
                offset.y + margin + 20.0 + commit_index as f32 * commit_spacing,
            ),
            GitDirection::LeftToRight => Pos2::new(
                offset.x + margin + 40.0 + commit_index as f32 * (commit_spacing + 30.0),
                offset.y + margin + 20.0 + lane as f32 * branch_spacing,
            ),
        }
    };

    let mut last_commit_pos: HashMap<String, Pos2> = HashMap::new();
    let mut commit_positions: HashMap<String, Pos2> = HashMap::new();

    for (index, commit) in graph.commits.iter().enumerate() {
        let pos = commit_position(index, &commit.branch);
        commit_positions.insert(commit.id.clone(), pos);

        let color = graph
            .branches
            .iter()
            .find(|branch| branch.name == commit.branch)
            .map(|branch| branch_colors[branch.color_idx % branch_colors.len()])
            .unwrap_or(branch_colors[0]);

        if let Some(previous) = last_commit_pos.get(&commit.branch) {
            painter.line_segment([*previous, pos], Stroke::new(3.0, color));
        }

        if let Some(ref merge_from) = commit.merge_from {
            if let Some(merge_pos) = last_commit_pos.get(merge_from) {
                let merge_color = graph
                    .branches
                    .iter()
                    .find(|branch| &branch.name == merge_from)
                    .map(|branch| branch_colors[branch.color_idx % branch_colors.len()])
                    .unwrap_or(color);
                draw_soft_link(&painter, *merge_pos, pos, Stroke::new(2.0, merge_color));
            }
        }

        if let Some(ref source_id) = commit.cherry_pick_from {
            if let Some(source_pos) = commit_positions.get(source_id).copied() {
                draw_dotted_link(&painter, source_pos, pos, Stroke::new(1.5, color));
            }
        }

        match commit.commit_type {
            GitCommitType::Reverse => {
                painter.circle_filled(pos, commit_radius, line_bg);
                painter.circle_stroke(pos, commit_radius, Stroke::new(3.0, color));
            }
            GitCommitType::Highlight => {
                painter.circle_filled(pos, commit_radius + 2.0, color);
                painter.circle_stroke(
                    pos,
                    commit_radius + 2.0,
                    Stroke::new(2.0, border_color),
                );
            }
            GitCommitType::Normal => {
                painter.circle_filled(pos, commit_radius, color);
            }
        }

        if commit.is_merge {
            painter.circle_stroke(pos, commit_radius, Stroke::new(2.0, border_color));
        }

        // Commit label
        let label = commit.message.as_ref().unwrap_or(&commit.id);
        match direction {
            GitDirection::TopToBottom => {
                let label_rect = Rect::from_min_size(
                    Pos2::new(offset.x + margin - 5.0, pos.y - font_size * 0.5 - 2.0),
                    Vec2::new(label_width - 10.0, font_size + 4.0),
                );
                painter.rect_filled(label_rect, 3.0, line_bg);
                painter.text(
                    Pos2::new(offset.x + margin, pos.y),
                    egui::Align2::LEFT_CENTER,
                    label,
                    FontId::proportional(font_size - 2.0),
                    text_color,
                );
            }
            GitDirection::LeftToRight => {
                painter.text(
                    Pos2::new(pos.x, pos.y - commit_radius - 16.0),
                    egui::Align2::CENTER_BOTTOM,
                    label,
                    FontId::proportional(font_size - 3.0),
                    text_color,
                );
            }
        }

        // Tag badges
        for (tag_index, tag) in commit.tags.iter().enumerate() {
            let tag_pos = match direction {
                GitDirection::TopToBottom => Pos2::new(
                    pos.x + commit_radius + 6.0,
                    pos.y - 6.0 - tag_index as f32 * 16.0,
                ),
                GitDirection::LeftToRight => Pos2::new(
                    pos.x,
                    pos.y + commit_radius + 6.0 + tag_index as f32 * 16.0,
                ),
            };
            let tag_rect = Rect::from_center_size(
                tag_pos + Vec2::new(24.0, 0.0),
                Vec2::new(48.0, 14.0),
            );
            painter.rect_filled(tag_rect, 7.0, line_bg);
            painter.rect_stroke(
                tag_rect,
                7.0,
                Stroke::new(1.0, color),
                egui::StrokeKind::Inside,
            );
            painter.text(
                tag_rect.center(),
                egui::Align2::CENTER_CENTER,
                tag,
                FontId::proportional(font_size - 4.0),
                color,
            );
        }

        last_commit_pos.insert(commit.branch.clone(), pos);
    }

    // Branch labels
    for branch in &graph.branches {
        let lane = branch_lane.get(&branch.name).copied().unwrap_or(0);
        let color = branch_colors[branch.color_idx % branch_colors.len()];
        let pos = match direction {
            GitDirection::TopToBottom => Pos2::new(
                offset.x + margin + label_width + lane as f32 * branch_spacing,
                offset.y + margin,
            ),
            GitDirection::LeftToRight => Pos2::new(
                offset.x + margin,
                offset.y + margin + 20.0 + lane as f32 * branch_spacing,
            ),
        };
        painter.text(
            pos,
            egui::Align2::CENTER_BOTTOM,
            &branch.name,
            FontId::proportional(font_size - 2.0),
            color,
        );
    }
}

fn draw_soft_link(painter: &egui::Painter, from: Pos2, to: Pos2, stroke: Stroke) {
    let mid_y = (from.y + to.y) / 2.0;
    let ctrl1 = Pos2::new(from.x, mid_y);
    let ctrl2 = Pos2::new(to.x, mid_y);
    painter.line_segment([from, ctrl1], stroke);
    painter.line_segment([ctrl1, ctrl2], stroke);
    painter.line_segment([ctrl2, to], stroke);
}

fn draw_dotted_link(painter: &egui::Painter, from: Pos2, to: Pos2, stroke: Stroke) {
    let distance = (to - from).length();
    let steps = (distance / 8.0).max(2.0) as usize;
    for step in (0..steps).step_by(2) {
        let start = step as f32 / steps as f32;
        let end = ((step + 1) as f32 / steps as f32).min(1.0);
        painter.line_segment(
            [
                from + (to - from) * start,
                from + (to - from) * end,
            ],
            stroke,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_graph_tags_types_and_order() {
        let source = "gitGraph\ncommit id: \"a\" tag: \"v1\" tag: \"stable\" type: HIGHLIGHT\nbranch dev order: 1\ncommit id: \"b\" type: REVERSE";
        let graph = parse_git_graph(source).unwrap();
        assert_eq!(graph.direction, GitDirection::TopToBottom);
        assert_eq!(graph.commits[0].tags, vec!["v1", "stable"]);
        assert_eq!(graph.commits[0].commit_type, GitCommitType::Highlight);
        assert_eq!(graph.commits[1].commit_type, GitCommitType::Reverse);
        let dev = graph
            .branches
            .iter()
            .find(|branch| branch.name == "dev")
            .unwrap();
        assert_eq!(dev.order, Some(1));
    }

    #[test]
    fn test_git_graph_lr_and_cherry_pick() {
        let source = "gitGraph LR:\ncommit id: \"a\" msg: \"base\"\nbranch dev\ncheckout dev\ncommit id: \"b\" msg: \"feature\"\ncheckout main\ncherry-pick id: \"b\" msg: \"pick\" tag: \"picked\"";
        let graph = parse_git_graph(source).unwrap();
        assert_eq!(graph.direction, GitDirection::LeftToRight);
        let pick = graph
            .commits
            .iter()
            .find(|commit| commit.message.as_deref() == Some("pick"))
            .unwrap();
        assert_eq!(pick.cherry_pick_from.as_deref(), Some("b"));
        assert_eq!(pick.tags, vec!["picked"]);
    }

    #[test]
    fn test_git_graph_attribute_parser_handles_quotes() {
        let attrs = parse_attributes("id: \"a b\" msg: 'hello world' tag: v1");
        assert_eq!(
            attrs,
            vec![
                ("id".to_string(), "a b".to_string()),
                ("msg".to_string(), "hello world".to_string()),
                ("tag".to_string(), "v1".to_string()),
            ]
        );
    }
}
