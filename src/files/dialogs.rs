//! Native file dialog integration using the rfd crate
//!
//! This module provides functions to open native file picker dialogs
//! for opening and saving files, and for opening workspace folders.
//!
//! On Linux, rfd uses xdg-desktop-portal by default (via its built-in ashpd
//! backend). In Flatpak sandboxes, portal-based dialogs grant the app access
//! to user-selected paths without requiring broad filesystem permissions.

use log::debug;
use rfd::FileDialog;
use rust_i18n::t;
use std::path::PathBuf;

/// File extension filters for supported file types.
const MARKDOWN_EXTENSIONS: &[&str] = &["md", "markdown", "mdown", "mkd", "mkdn"];
const JSON_EXTENSIONS: &[&str] = &["json", "jsonc"];
const YAML_EXTENSIONS: &[&str] = &["yaml", "yml"];
const TOML_EXTENSIONS: &[&str] = &["toml"];
const TEXT_EXTENSIONS: &[&str] = &["txt", "text"];
const CSV_EXTENSIONS: &[&str] = &["csv", "tsv"];

/// Combined filter for all commonly edited file types (default filter).
/// Includes markdown, text, and data files that Ferrite supports.
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "md", "markdown", "mdown", "mkd", "mkdn", // Markdown
    "txt", "text",                            // Plain text
    "json", "jsonc",                          // JSON
    "yaml", "yml",                            // YAML
    "toml",                                   // TOML
    "csv", "tsv",                             // Tabular data
];

/// Returns true when running inside a Flatpak sandbox.
pub fn is_flatpak() -> bool {
    std::env::var("FLATPAK_ID").is_ok()
}

/// Resolve initial directory for a file dialog, with Flatpak-aware fallback.
///
/// In Flatpak, the xdg-desktop-portal file chooser needs a navigable starting
/// directory. Without one, the portal may fail silently or start in an
/// inaccessible sandbox-internal path. We fall back to `$HOME` (which the
/// portal can translate) to ensure the dialog always opens at a usable location.
fn resolve_initial_dir(initial_dir: Option<&PathBuf>) -> Option<PathBuf> {
    if let Some(dir) = initial_dir {
        if dir.is_dir() {
            return Some(dir.clone());
        }
        debug!("Provided initial_dir does not exist: {}", dir.display());
    }

    // Fallback: use $HOME so the portal dialog has a navigable starting point.
    // This is especially important in Flatpak where the default may be
    // a sandbox-internal path the user can't browse from.
    if let Some(home) = dirs::home_dir() {
        if home.is_dir() {
            return Some(home);
        }
    }

    None
}

/// Opens a native folder picker dialog for selecting a workspace folder.
///
/// Uses xdg-desktop-portal automatically on Linux (rfd's default backend).
/// In Flatpak, the portal grants sandbox access to the selected directory.
/// Returns `Some(PathBuf)` if a folder was selected, `None` if cancelled.
pub fn open_folder_dialog(initial_dir: Option<&PathBuf>) -> Option<PathBuf> {
    let effective_dir = resolve_initial_dir(initial_dir);

    let mut dialog = FileDialog::new().set_title(&t!("file_dialog.open_workspace").to_string());

    if let Some(dir) = effective_dir.as_ref() {
        dialog = dialog.set_directory(dir);
    }

    let result = dialog.pick_folder();

    if result.is_none() && is_flatpak() {
        debug!(
            "Folder dialog returned None in Flatpak (initial_dir: {:?}). \
             This may be a portal/sandbox issue or the user cancelled.",
            initial_dir
        );
    }

    result
}

/// Opens a native file dialog for selecting multiple files.
///
/// Supports Markdown, JSON, YAML, TOML, CSV/TSV, and plain text files.
/// The default filter shows all supported file types.
/// Returns a vector of selected file paths. Empty if the dialog was cancelled.
pub fn open_multiple_files_dialog(initial_dir: Option<&PathBuf>) -> Vec<PathBuf> {
    let effective_dir = resolve_initial_dir(initial_dir);

    let mut dialog = FileDialog::new()
        .set_title(&t!("file_dialog.open_files").to_string())
        .add_filter(&t!("file_dialog.filter.supported").to_string(), SUPPORTED_EXTENSIONS)
        .add_filter(&t!("file_dialog.filter.markdown").to_string(), MARKDOWN_EXTENSIONS)
        .add_filter(&t!("file_dialog.filter.text").to_string(), TEXT_EXTENSIONS)
        .add_filter(&t!("file_dialog.filter.json").to_string(), JSON_EXTENSIONS)
        .add_filter(&t!("file_dialog.filter.yaml").to_string(), YAML_EXTENSIONS)
        .add_filter(&t!("file_dialog.filter.toml").to_string(), TOML_EXTENSIONS)
        .add_filter(&t!("file_dialog.filter.csv_tsv").to_string(), CSV_EXTENSIONS)
        .add_filter(&t!("file_dialog.filter.all").to_string(), &["*"]);

    if let Some(dir) = effective_dir.as_ref() {
        dialog = dialog.set_directory(dir);
    }

    dialog.pick_files().unwrap_or_default()
}

/// Opens a native save dialog for saving a file.
///
/// Returns `Some(PathBuf)` if a location was selected, `None` if cancelled.
pub fn save_file_dialog(
    initial_dir: Option<&PathBuf>,
    default_name: Option<&str>,
) -> Option<PathBuf> {
    let effective_dir = resolve_initial_dir(initial_dir);

    let mut dialog = FileDialog::new()
        .set_title(&t!("file_dialog.save_file").to_string())
        .add_filter(&t!("file_dialog.filter.supported").to_string(), SUPPORTED_EXTENSIONS)
        .add_filter(&t!("file_dialog.filter.markdown").to_string(), MARKDOWN_EXTENSIONS)
        .add_filter(&t!("file_dialog.filter.text").to_string(), TEXT_EXTENSIONS)
        .add_filter(&t!("file_dialog.filter.json").to_string(), JSON_EXTENSIONS)
        .add_filter(&t!("file_dialog.filter.yaml").to_string(), YAML_EXTENSIONS)
        .add_filter(&t!("file_dialog.filter.toml").to_string(), TOML_EXTENSIONS)
        .add_filter(&t!("file_dialog.filter.csv_tsv").to_string(), CSV_EXTENSIONS)
        .add_filter(&t!("file_dialog.filter.all").to_string(), &["*"]);

    if let Some(dir) = effective_dir.as_ref() {
        dialog = dialog.set_directory(dir);
    }

    if let Some(name) = default_name {
        dialog = dialog.set_file_name(name);
    }

    dialog.save_file()
}
