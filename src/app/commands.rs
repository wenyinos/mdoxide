//! Command registry for the command palette.
//!
//! Provides a unified list of all executable commands with metadata
//! (display name, category, shortcut hint, icon) for the command palette UI.

use crate::config::ShortcutCommand;

/// A command entry for the palette, combining a shortcut command with its
/// display metadata and an optional icon.
#[derive(Debug, Clone)]
pub struct PaletteCommand {
    pub command: ShortcutCommand,
    pub icon: &'static str,
}

impl PaletteCommand {
    pub fn label(&self) -> &'static str {
        self.command.display_name()
    }

    pub fn category(&self) -> &'static str {
        self.command.category()
    }
}

/// Build the full list of commands available in the palette.
/// Excludes CommandPalette itself (no recursion).
pub fn all_palette_commands() -> Vec<PaletteCommand> {
    ShortcutCommand::all()
        .iter()
        .filter(|cmd| !matches!(cmd, ShortcutCommand::CommandPalette))
        .map(|&command| PaletteCommand {
            command,
            icon: icon_for_command(command),
        })
        .collect()
}

fn icon_for_command(cmd: ShortcutCommand) -> &'static str {
    match cmd {
        // File
        ShortcutCommand::Save => "💾",
        ShortcutCommand::SaveAs => "💾",
        ShortcutCommand::Open => "📂",
        ShortcutCommand::New => "📄",
        ShortcutCommand::NewTab => "➕",
        ShortcutCommand::CloseTab => "✖",
        ShortcutCommand::OpenWorkspace => "📁",
        ShortcutCommand::CloseWorkspace => "📁",
        // Navigation
        ShortcutCommand::NextTab => "→",
        ShortcutCommand::PrevTab => "←",
        ShortcutCommand::GoToLine => "🔢",
        ShortcutCommand::QuickOpen => "🔍",
        // View
        ShortcutCommand::ToggleViewMode => "👁",
        ShortcutCommand::CycleTheme => "🎨",
        ShortcutCommand::ToggleZenMode => "🧘",
        ShortcutCommand::ToggleFullscreen => "⛶",
        ShortcutCommand::ToggleOutline => "📑",
        ShortcutCommand::ToggleFileTree => "🌳",
        ShortcutCommand::TogglePipeline => "⚡",
        ShortcutCommand::ToggleTerminal => "💻",
        ShortcutCommand::ToggleProductivityHub => "📋",
        ShortcutCommand::ZoomIn => "🔎",
        ShortcutCommand::ZoomOut => "🔎",
        ShortcutCommand::ResetZoom => "🔎",
        // Edit
        ShortcutCommand::Undo => "↩",
        ShortcutCommand::Redo => "↪",
        ShortcutCommand::DeleteLine => "🗑",
        ShortcutCommand::DuplicateLine => "📋",
        ShortcutCommand::MoveLineUp => "⬆",
        ShortcutCommand::MoveLineDown => "⬇",
        ShortcutCommand::SelectNextOccurrence => "🔤",
        // Search
        ShortcutCommand::Find => "🔍",
        ShortcutCommand::FindReplace => "🔄",
        ShortcutCommand::FindNext => "▼",
        ShortcutCommand::FindPrev => "▲",
        ShortcutCommand::SearchInFiles => "🔍",
        // Format
        ShortcutCommand::FormatBold => "𝐁",
        ShortcutCommand::FormatItalic => "𝐼",
        ShortcutCommand::FormatInlineCode => "<>",
        ShortcutCommand::FormatCodeBlock => "{ }",
        ShortcutCommand::FormatLink => "🔗",
        ShortcutCommand::FormatImage => "🖼",
        ShortcutCommand::FormatBlockquote => "❝",
        ShortcutCommand::FormatBulletList => "•",
        ShortcutCommand::FormatNumberedList => "1.",
        ShortcutCommand::FormatHeading1
        | ShortcutCommand::FormatHeading2
        | ShortcutCommand::FormatHeading3
        | ShortcutCommand::FormatHeading4
        | ShortcutCommand::FormatHeading5
        | ShortcutCommand::FormatHeading6 => "H",
        // Folding
        ShortcutCommand::FoldAll => "▶",
        ShortcutCommand::UnfoldAll => "▽",
        ShortcutCommand::ToggleFoldAtCursor => "⊞",
        // Other
        ShortcutCommand::CommandPalette => "⌘",
        ShortcutCommand::OpenSettings => "⚙",
        ShortcutCommand::OpenAbout => "ℹ",
        ShortcutCommand::ExportHtml => "🌐",
        ShortcutCommand::InsertToc => "📑",
        ShortcutCommand::ToggleFrontmatter => "📝",
    }
}
