//! Starter snippets for **Insert → Mermaid** in the raw-editor format toolbar.
//!
//! Each variant matches a diagram type handled in `render_mermaid_diagram`.
//! Edit bodies here to adjust defaults app-wide.

use rust_i18n::t;

/// Supported Mermaid diagram kinds for template insertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MermaidTemplateKind {
    Flowchart,
    Sequence,
    State,
    Class,
    Er,
    Pie,
    Gantt,
    Journey,
    Mindmap,
    Timeline,
    GitGraph,
}

impl MermaidTemplateKind {
    /// Stable order for toolbar / menus.
    pub const ALL: &[Self] = &[
        Self::Flowchart,
        Self::Sequence,
        Self::State,
        Self::Class,
        Self::Er,
        Self::Pie,
        Self::Gantt,
        Self::Journey,
        Self::Mindmap,
        Self::Timeline,
        Self::GitGraph,
    ];

    pub fn snippet_body(self) -> &'static str {
        match self {
            Self::Flowchart => concat!(
                "flowchart TD\n",
                "    A[Start] --> T[/Trapezoid\\]\n",
                "    T --> I[\\Inverted/]\n",
                "    I --> D(((Double circle))\n",
                "    style A fill:#eef,stroke:#333,color:#222"
            ),
            Self::Sequence => "sequenceDiagram\n    Alice->>Bob: Hello",
            Self::State => "stateDiagram-v2\n    [*] --> Still\n    Still --> [*]",
            Self::Class => "classDiagram\n    class Animal\n    class Dog\n    Dog --|> Animal",
            Self::Er => "erDiagram\n    CUSTOMER ||--o{ ORDER : places",
            Self::Pie => "pie title Distribution\n    \"A\" : 40\n    \"B\" : 35\n    \"C\" : 25",
            Self::Gantt => "gantt\n    title Sample\n    First :f1, 1d",
            Self::Journey => "journey\n    section Browse\n      Look around: 4: User",
            Self::Mindmap => "mindmap\n  root((Topic))\n    Idea A\n    Idea B",
            Self::Timeline => "timeline\n    2024 : Kickoff\n    2025 : Launch",
            Self::GitGraph => "gitGraph\n    commit id: \"initial\"",
        }
    }

    /// `rust_i18n` key for the About / Help paragraph (under `about.mermaid.types.*`).
    pub fn help_description_key(self) -> &'static str {
        match self {
            Self::Flowchart => "about.mermaid.types.flowchart",
            Self::Sequence => "about.mermaid.types.sequence",
            Self::State => "about.mermaid.types.state",
            Self::Class => "about.mermaid.types.class",
            Self::Er => "about.mermaid.types.er",
            Self::Pie => "about.mermaid.types.pie",
            Self::Gantt => "about.mermaid.types.gantt",
            Self::Journey => "about.mermaid.types.journey",
            Self::Mindmap => "about.mermaid.types.mindmap",
            Self::Timeline => "about.mermaid.types.timeline",
            Self::GitGraph => "about.mermaid.types.gitgraph",
        }
    }
}

/// Localized label matching **Insert → Mermaid…** menu entries.
pub fn mermaid_kind_menu_label(kind: MermaidTemplateKind) -> String {
    match kind {
        MermaidTemplateKind::Flowchart => t!("format_toolbar.mermaid_flowchart").to_string(),
        MermaidTemplateKind::Sequence => t!("format_toolbar.mermaid_sequence").to_string(),
        MermaidTemplateKind::State => t!("format_toolbar.mermaid_state").to_string(),
        MermaidTemplateKind::Class => t!("format_toolbar.mermaid_class").to_string(),
        MermaidTemplateKind::Er => t!("format_toolbar.mermaid_er").to_string(),
        MermaidTemplateKind::Pie => t!("format_toolbar.mermaid_pie").to_string(),
        MermaidTemplateKind::Gantt => t!("format_toolbar.mermaid_gantt").to_string(),
        MermaidTemplateKind::Journey => t!("format_toolbar.mermaid_journey").to_string(),
        MermaidTemplateKind::Mindmap => t!("format_toolbar.mermaid_mindmap").to_string(),
        MermaidTemplateKind::Timeline => t!("format_toolbar.mermaid_timeline").to_string(),
        MermaidTemplateKind::GitGraph => t!("format_toolbar.mermaid_gitgraph").to_string(),
    }
}

/// Fenced block body as inserted at the cursor (core `` ```mermaid ... ``` `` only; surrounding newlines depend on context).
pub fn snippet_fenced_block(kind: MermaidTemplateKind) -> String {
    format!("```mermaid\n{}\n```", kind.snippet_body())
}
