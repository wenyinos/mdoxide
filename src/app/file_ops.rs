//! File operations for the Ferrite application.
//!
//! This module contains handlers for file open, save, save-as, workspace
//! management, drag-and-drop, file tree context actions, file watcher events,
//! and git auto-refresh.

use super::FerriteApp;
use crate::config::ViewMode;
use crate::files::dialogs::{open_multiple_files_dialog, save_file_dialog};
use crate::ui::{FileOperationDialog, FileTreeContextAction, SearchNavigationTarget};
use eframe::egui;
use log::{debug, info, trace, warn};
use std::path::{Path, PathBuf};

impl FerriteApp {

    /// Handle the "File > Open" action.
    ///
    /// Opens a native file dialog allowing multiple file selection and loads
    /// each selected file into a new tab.
    pub(crate) fn handle_open_file(&mut self) {
        // Get the last open directory from recent files, if available
        let initial_dir = self
            .state
            .settings
            .recent_files
            .first()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf());

        // Open the native file dialog (supports multiple selection)
        let paths = open_multiple_files_dialog(initial_dir.as_ref());

        if paths.is_empty() {
            debug!("File dialog cancelled");
            return;
        }

        let file_count = paths.len();
        let mut success_count = 0;
        let mut last_error: Option<String> = None;

        for path in paths {
            info!("Opening file: {}", path.display());
            let time = self.get_app_time();
            match self.state.open_file(path.clone(), Some(time)) {
                Ok(tab_index) => {
                    success_count += 1;
                    self.pending_cjk_check = true;
                    // Check for auto-save recovery
                    self.check_auto_save_recovery(tab_index);
                }
                Err(e) => {
                    warn!("Failed to open file {}: {}", path.display(), e);
                    last_error = Some(format!("Failed to open {}:\n{}", path.display(), e));
                }
            }
        }

        // Show toast for multiple files opened
        if file_count > 1 && success_count > 0 {
            let time = self.get_app_time();
            self.state
                .show_toast(format!("Opened {} files", success_count), time, 2.0);
        }

        // Show error if any file failed to open
        if let Some(error) = last_error {
            self.state.show_error(error);
        }
    }

    /// Handle the "File > Save" action.
    ///
    /// Saves the current document to its existing file path.
    /// If the document has no path, triggers "Save As" instead.
    pub(crate) fn handle_save_file(&mut self) {
        // Special tabs (settings, about) cannot be saved
        if self.state.active_tab().map(|t| t.is_special()).unwrap_or(false) {
            return;
        }

        // Check if the active tab has a path
        let has_path = self
            .state
            .active_tab()
            .map(|t| t.path.is_some())
            .unwrap_or(false);

        if has_path {
            // Save to existing path
            let path_display = self
                .state
                .active_tab()
                .and_then(|t| t.path.as_ref())
                .map(|p| p.display().to_string())
                .unwrap_or_default();

            // Get tab ID before save for cleanup
            let tab_id = self.state.active_tab().map(|t| t.id);

            match self.state.save_active_tab() {
                Ok(_) => {
                    debug!("File saved successfully");
                    let time = self.get_app_time();
                    self.state
                        .show_toast(format!("Saved: {}", path_display), time, 3.0);
                    
                    // Clean up auto-save temp file after successful manual save
                    if let Some(id) = tab_id {
                        self.cleanup_auto_save_for_tab(id);
                    }
                    
                    // Trigger git status refresh after successful save
                    self.request_git_refresh();

                    // Update backlink index incrementally for the saved file
                    if let Some(path) = self.state.active_tab().and_then(|t| t.path.clone()) {
                        if self.state.backlink_index.is_built {
                            self.state.backlink_index.update_file(&path);
                        }
                        self.backlinks_need_refresh = true;
                    }
                }
                Err(e) => {
                    warn!("Failed to save file: {}", e);
                    self.state
                        .show_error(format!("Failed to save file:\n{}", e));
                }
            }
        } else {
            // No path set, trigger Save As
            self.handle_save_as_file();
        }
    }

    /// Handle the "File > Save As" action.
    ///
    /// Opens a native save dialog and saves the document to the selected location.
    pub(crate) fn handle_save_as_file(&mut self) {
        // Get initial directory from current file or recent files
        let initial_dir = self
            .state
            .active_tab()
            .and_then(|t| t.path.as_ref())
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .or_else(|| {
                self.state
                    .settings
                    .recent_files
                    .first()
                    .and_then(|p| p.parent())
                    .map(|p| p.to_path_buf())
            });

        // Get default filename from current tab
        let default_name = self
            .state
            .active_tab()
            .and_then(|t| t.path.as_ref())
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "untitled.md".to_string());

        // Open the native save dialog
        if let Some(path) = save_file_dialog(initial_dir.as_ref(), Some(&default_name)) {
            info!("Saving file as: {}", path.display());
            
            // Get old path and tab ID before save for cleanup
            let old_path = self.state.active_tab().and_then(|t| t.path.clone());
            let tab_id = self.state.active_tab().map(|t| t.id);

            match self.state.save_active_tab_as(path.clone()) {
                Ok(_) => {
                    let time = self.get_app_time();
                    self.state
                        .show_toast(format!("Saved: {}", path.display()), time, 3.0);
                    
                    // Clean up auto-save temp files after successful manual save
                    // (both old path and new path, in case they differ)
                    if let Some(id) = tab_id {
                        use crate::config::delete_auto_save;
                        // Clean up old path's auto-save
                        delete_auto_save(id, old_path.as_ref());
                        // Clean up new path's auto-save (in case it exists)
                        delete_auto_save(id, Some(&path));
                        debug!("Cleaned up auto-save temp files for tab {}", id);
                    }
                    
                    // Trigger git status refresh after successful save
                    self.request_git_refresh();

                    // Update backlink index incrementally and refresh backlinks
                    if self.state.backlink_index.is_built {
                        self.state.backlink_index.update_file(&path);
                    }
                    self.backlinks_need_refresh = true;
                }
                Err(e) => {
                    warn!("Failed to save file: {}", e);
                    self.state
                        .show_error(format!("Failed to save file:\n{}", e));
                }
            }
        } else {
            debug!("Save dialog cancelled");
        }
    }

    /// Handle the "File > Open Workspace" action.
    ///
    /// Opens a native folder dialog and switches to workspace mode.
    pub(crate) fn handle_open_workspace(&mut self) {
        use crate::files::dialogs::open_folder_dialog;

        // Get initial directory from recent workspaces or recent files
        let initial_dir = self
            .state
            .settings
            .recent_workspaces
            .first()
            .cloned()
            .or_else(|| {
                self.state
                    .settings
                    .recent_files
                    .first()
                    .and_then(|p| p.parent())
                    .map(|p| p.to_path_buf())
            });

        // Open the native folder dialog
        if let Some(folder_path) = open_folder_dialog(initial_dir.as_ref()) {
            info!("Opening workspace: {}", folder_path.display());
            match self.state.open_workspace(folder_path.clone()) {
                Ok(_) => {
                    let time = self.get_app_time();
                    let folder_name = folder_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("folder");
                    self.state
                        .show_toast(format!("Opened workspace: {}", folder_name), time, 2.5);
                    
                    // Auto-load terminal layout if enabled
                    if self.state.settings.terminal_auto_load_layout {
                        let layout_path = folder_path.join("terminal_layout.json");
                        if layout_path.exists() {
                            if let Ok(json) = std::fs::read_to_string(layout_path) {
                                if let Ok(workspace) = serde_json::from_str::<crate::terminal::SavedWorkspace>(&json) {
                                    match self.terminal_panel_state.manager.load_workspace(workspace) {
                                        Ok(fws) => {
                                            self.terminal_panel_state.floating_windows.clear();
                                            for (layout, title, pos, size) in fws {
                                                let leaf = layout.first_leaf();
                                                let id = egui::ViewportId::from_hash_of(egui::Id::new("floating_term").with(leaf));
                                                self.terminal_panel_state.floating_windows.push(crate::ui::FloatingWindow {
                                                    id,
                                                    layout,
                                                    title,
                                                    pos: pos.map(|(x, y)| egui::pos2(x, y)),
                                                    size: egui::vec2(size.0, size.1),
                                                    first_frame: true,
                                                });
                                            }
                                            info!("Auto-loaded terminal layout from workspace root");
                                        }
                                        Err(e) => warn!("Failed to auto-load terminal layout: {}", e),
                                    }
                                }
                            }
                        }
                    }

                    // Immediately save session to persist the workspace path
                    self.force_session_save();
                }
                Err(e) => {
                    warn!("Failed to open workspace: {}", e);
                    self.state
                        .show_error(format!("Failed to open workspace:\n{}", e));
                }
            }
        } else {
            debug!("Open workspace dialog cancelled");
        }
    }

    /// Handle closing the current workspace.
    ///
    /// Returns to single-file mode and hides workspace UI.
    pub(crate) fn handle_close_workspace(&mut self) {
        if self.state.is_workspace_mode() {
            self.state.close_workspace();
            self.state.backlink_index.clear();
            self.backlinks_panel.clear();
            self.backlinks_need_refresh = true;
            let time = self.get_app_time();
            self.state.show_toast("Workspace closed", time, 2.0);
            
            // Immediately save session to persist the mode change
            self.force_session_save();
        }
    }
    
    /// Force an immediate session save (bypasses throttling).
    ///
    /// Use this after important state changes like opening/closing workspaces
    /// to ensure the change is persisted immediately.
    pub(crate) fn force_session_save(&mut self) {
        use crate::config::save_crash_recovery_state;

        let workspace_info = if let Some(root) = self.state.workspace_root() {
            format!("Workspace({})", root.display())
        } else {
            "SingleFile".to_string()
        };
        debug!(
            "Force session save requested: app_mode={}",
            workspace_info
        );

        let mut session_state = self.state.capture_session_state();
        session_state.clean_shutdown = false; // This is a crash recovery snapshot
        self.inject_csv_delimiters(&mut session_state);

        if save_crash_recovery_state(&session_state) {
            self.session_save_throttle.record_save();
            debug!(
                "Forced session save completed successfully: app_mode={}",
                workspace_info
            );
        } else {
            warn!(
                "Failed to force session save: app_mode={}",
                workspace_info
            );
        }
    }

    /// Handle toggling the file tree panel visibility.
    pub(crate) fn handle_toggle_file_tree(&mut self) {
        if self.state.is_workspace_mode() {
            self.state.toggle_file_tree();
            let time = self.get_app_time();
            let msg = if self.state.should_show_file_tree() {
                "File tree shown"
            } else {
                "File tree hidden"
            };
            self.state.show_toast(msg, time, 1.5);
        } else {
            // Not in workspace mode - show a hint
            let time = self.get_app_time();
            self.state
                .show_toast("Open a folder first (📁 button)", time, 2.0);
        }
    }

    /// Handle opening the quick file switcher.
    pub(crate) fn handle_quick_open(&mut self) {
        if self.state.is_workspace_mode() {
            self.quick_switcher.toggle();
        } else {
            // Not in workspace mode - show a hint
            let time = self.get_app_time();
            self.state
                .show_toast("Open a folder first to use quick open", time, 2.0);
        }
    }

    /// Handle opening the search in files panel.
    pub(crate) fn handle_search_in_files(&mut self) {
        if self.state.is_workspace_mode() {
            self.search_panel.toggle();
            // Trigger search if panel is now open
            if self.search_panel.is_open() {
                if let Some(workspace) = &self.state.workspace {
                    let files = workspace.all_files();
                    self.search_panel.search(&files, &workspace.hidden_patterns);
                }
            }
        } else {
            // Not in workspace mode - show a hint
            let time = self.get_app_time();
            self.state
                .show_toast("Open a folder first to use search in files", time, 2.0);
        }
    }

    /// Handle navigation from a search-in-files result click.
    ///
    /// This opens the file (if not already open), scrolls to the match location,
    /// applies a transient highlight, and switches to Raw mode if necessary.
    pub(crate) fn handle_search_navigation(&mut self, target: SearchNavigationTarget) {
        let file_path = target.path.clone();

        // Open the file (or switch to existing tab)
        let time = self.get_app_time();
        match self.state.open_file(file_path.clone(), Some(time)) {
            Ok(_) => {
                self.pending_cjk_check = true;
                debug!(
                    "Opened file from search: {} at line {}, char offset {}",
                    file_path.display(),
                    target.line_number,
                    target.char_offset
                );

                // Get the active tab and apply navigation
                if let Some(tab) = self.state.active_tab_mut() {
                    // Switch to Raw mode if currently in Rendered mode
                    // (search results are based on raw text positions)
                    if tab.view_mode == ViewMode::Rendered {
                        tab.view_mode = ViewMode::Raw;
                        debug!("Switched to Raw mode for search navigation");
                    }

                    // Clear any existing transient highlight from previous navigations
                    tab.clear_transient_highlight();

                    // Set the transient highlight for the matched text
                    let highlight_end = target.char_offset + target.match_len;
                    tab.set_transient_highlight(target.char_offset, highlight_end);

                    // Set cursor position to the match location
                    tab.set_cursor(target.char_offset);

                    // Schedule scroll to the target line (editor will handle this)
                    self.pending_scroll_to_line = Some(target.line_number);

                    debug!(
                        "Set transient highlight at {}..{} and scroll to line {}",
                        target.char_offset, highlight_end, target.line_number
                    );
                }

                // Add to workspace recent files
                if let Some(workspace) = self.state.workspace_mut() {
                    workspace.add_recent_file(file_path);
                }
            }
            Err(e) => {
                warn!("Failed to open file from search: {}", e);
                self.state
                    .show_error(format!("Failed to open file:\n{}", e));
            }
        }
    }

    /// Handle file watcher events from the workspace.
    pub(crate) fn handle_file_watcher_events(&mut self) {
        use crate::workspaces::WorkspaceEvent;

        // Poll for new events
        self.state.poll_file_watcher();

        // Process any pending events
        let events = self.state.take_file_events();
        if events.is_empty() {
            return;
        }

        let mut need_tree_refresh = false;
        let mut modified_files: Vec<std::path::PathBuf> = Vec::new();

        for event in events {
            match event {
                WorkspaceEvent::FileCreated(path) => {
                    debug!("File created: {}", path.display());
                    need_tree_refresh = true;
                }
                WorkspaceEvent::FileDeleted(path) => {
                    debug!("File deleted: {}", path.display());
                    need_tree_refresh = true;

                    // Check if this file is open in a tab and mark it
                    for tab in self.state.tabs() {
                        if tab.path.as_ref() == Some(&path) {
                            // File was deleted externally - we could show a warning
                            // For now, just log it
                            warn!("Open file was deleted: {}", path.display());
                        }
                    }
                }
                WorkspaceEvent::FileModified(path) => {
                    debug!("File modified: {}", path.display());

                    // Notify terminal panel for watch mode
                    self.terminal_panel_state.manager.on_file_changed(&path);

                    // Check if this file is open in a tab
                    for tab in self.state.tabs() {
                        if tab.path.as_ref() == Some(&path) {
                            modified_files.push(path.clone());
                            break;
                        }
                    }
                }
                WorkspaceEvent::FileRenamed(old_path, new_path) => {
                    debug!(
                        "File renamed: {} -> {}",
                        old_path.display(),
                        new_path.display()
                    );
                    need_tree_refresh = true;
                }
                WorkspaceEvent::Error(msg) => {
                    warn!("File watcher error: {}", msg);
                }
            }
        }

        // Refresh file tree if needed
        if need_tree_refresh {
            self.state.refresh_workspace();
            // Also request git refresh since files changed
            self.request_git_refresh();
        }

        // Reload externally modified files that are open in tabs
        if !modified_files.is_empty() {
            let time = self.get_app_time();
            let mut reloaded_count = 0;
            let tab_count = self.state.tab_count();

            for path in &modified_files {
                // Read the updated content from disk
                match std::fs::read(path) {
                    Ok(bytes) => {
                        // Detect encoding and decode
                        let new_content = String::from_utf8(bytes.clone())
                            .unwrap_or_else(|_| String::from_utf8_lossy(&bytes).to_string());

                        // Find the tab with this path and reload if not modified by user
                        for idx in 0..tab_count {
                            let should_reload = self.state.tab(idx)
                                .map(|tab| tab.path.as_ref() == Some(path) && !tab.is_modified())
                                .unwrap_or(false);
                            let has_unsaved = self.state.tab(idx)
                                .map(|tab| tab.path.as_ref() == Some(path) && tab.is_modified())
                                .unwrap_or(false);

                            if should_reload {
                                if let Some(tab) = self.state.tab_mut(idx) {
                                    tab.content = new_content.clone();
                                    // Clamp cursor to new content length
                                    let max_chars = tab.content.chars().count();
                                    let current_cursor = tab.cursors.primary().head.min(max_chars);
                                    tab.pending_cursor_restore = Some(current_cursor);
                                    reloaded_count += 1;
                                    debug!(
                                        "Reloaded externally modified file: {}",
                                        path.display()
                                    );
                                }
                                break;
                            } else if has_unsaved {
                                warn!(
                                    "File modified externally but tab has unsaved changes: {}",
                                    path.display()
                                );
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read externally modified file {}: {}", path.display(), e);
                    }
                }
            }

            // Show appropriate toast
            let msg = if reloaded_count > 0 {
                if reloaded_count == 1 {
                    format!(
                        "Reloaded: {}",
                        modified_files[0]
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                    )
                } else {
                    format!("{} files reloaded from disk", reloaded_count)
                }
            } else if modified_files.len() == 1 {
                format!(
                    "File changed externally (unsaved changes): {}",
                    modified_files[0]
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                )
            } else {
                format!("{} files changed externally (have unsaved changes)", modified_files.len())
            };
            self.state.show_toast(msg, time, 3.0);
        }
    }

    /// Handle automatic Git status refresh.
    ///
    /// This method manages:
    /// - Refresh on window focus gained
    /// - Periodic refresh every 10 seconds when a workspace is open
    /// - Debounced refresh requests (e.g., after file save)
    pub(crate) fn handle_git_auto_refresh(&mut self, ctx: &egui::Context) {
        // Get window focus state
        let is_focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));

        // Update focus state and detect focus gained
        self.git_auto_refresh.update_focus(is_focused);

        // Check if git service is active (workspace with git repo)
        let git_active = self.state.git_service.is_open();

        // Tick the auto-refresh manager
        if self.git_auto_refresh.tick(git_active) {
            // Perform the actual refresh
            self.state.git_service.refresh_status();
            self.git_auto_refresh.mark_refreshed();
            trace!("Git status auto-refreshed");
        }
    }

    /// Request a Git status refresh.
    ///
    /// This triggers a debounced refresh - multiple rapid calls will be batched
    /// into a single refresh after a short delay (500ms).
    pub(crate) fn request_git_refresh(&mut self) {
        if self.state.git_service.is_open() {
            self.git_auto_refresh.request_refresh();
        }
    }

    /// Check if a file path has a supported image extension.
    pub(crate) fn is_supported_image(path: &std::path::Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| {
                matches!(
                    ext.to_lowercase().as_str(),
                    "png" | "jpg" | "jpeg" | "gif" | "webp"
                )
            })
            .unwrap_or(false)
    }

    /// Get the assets directory for storing dropped images.
    ///
    /// Priority:
    /// 1. Relative to the current document's directory (if document is saved)
    /// 2. Workspace root (if in workspace mode)
    /// 3. Current working directory as fallback
    pub(crate) fn get_assets_dir(&self) -> std::path::PathBuf {
        // Try to get the current document's directory
        if let Some(tab) = self.state.active_tab() {
            if let Some(doc_path) = &tab.path {
                if let Some(parent) = doc_path.parent() {
                    return parent.join("assets");
                }
            }
        }

        // Fall back to workspace root
        if let Some(workspace_root) = self.state.workspace_root() {
            return workspace_root.join("assets");
        }

        // Last resort: current directory
        std::path::PathBuf::from("assets")
    }

    /// Generate a unique filename for a dropped image using timestamp.
    ///
    /// Format: YYYYMMDD-HHMMSS-originalname.ext
    pub(crate) fn generate_unique_image_filename(original_path: &std::path::Path) -> String {
        use std::time::SystemTime;

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| {
                // Convert to local time components
                let secs = d.as_secs();
                // Simple timestamp format: YYYYMMDD-HHMMSS
                // Note: This uses UTC, but that's fine for uniqueness
                let days = secs / 86400;
                let time_of_day = secs % 86400;
                let hours = time_of_day / 3600;
                let minutes = (time_of_day % 3600) / 60;
                let seconds = time_of_day % 60;

                // Approximate year/month/day calculation (not accounting for leap years perfectly)
                let years_since_1970 = days / 365;
                let year = 1970 + years_since_1970;
                let remaining_days = days % 365;
                let month = (remaining_days / 30) + 1;
                let day = (remaining_days % 30) + 1;

                format!(
                    "{:04}{:02}{:02}-{:02}{:02}{:02}",
                    year, month, day, hours, minutes, seconds
                )
            })
            .unwrap_or_else(|_| "unknown".to_string());

        let original_name = original_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("image");

        let extension = original_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png");

        format!("{}-{}.{}", timestamp, original_name, extension)
    }

    /// Handle a dropped image file by copying it to assets and inserting markdown link.
    pub(crate) fn handle_dropped_image(&mut self, image_path: &std::path::Path) -> Result<(), String> {
        // Get the assets directory
        let assets_dir = self.get_assets_dir();

        // Create assets directory if it doesn't exist
        if !assets_dir.exists() {
            std::fs::create_dir_all(&assets_dir).map_err(|e| {
                format!(
                    "Failed to create assets directory '{}': {}",
                    assets_dir.display(),
                    e
                )
            })?;
            info!("Created assets directory: {}", assets_dir.display());
        }

        // Generate unique filename
        let new_filename = Self::generate_unique_image_filename(image_path);
        let dest_path = assets_dir.join(&new_filename);

        // Copy the image file
        std::fs::copy(image_path, &dest_path).map_err(|e| {
            format!(
                "Failed to copy image to '{}': {}",
                dest_path.display(),
                e
            )
        })?;
        info!(
            "Copied dropped image to: {} (from {})",
            dest_path.display(),
            image_path.display()
        );

        // Insert markdown link at cursor position in the active tab
        // Uses cursor_position (line, col) which is reliably synced from FerriteEditor,
        // rather than tab.cursors which may be stale.
        if let Some(tab) = self.state.active_tab_mut() {
            // Save state for undo
            let old_content = tab.content.clone();
            let old_cursor = tab.cursors.primary().head;

            // Use cursor_position (line, col) which is reliably synced from FerriteEditor
            let (cursor_line, cursor_col) = tab.cursor_position;

            // Calculate byte position from line/col
            let lines: Vec<&str> = tab.content.split('\n').collect();
            let mut cursor_byte = 0usize;
            for (i, line) in lines.iter().enumerate() {
                if i == cursor_line {
                    cursor_byte += cursor_col.min(line.len());
                    break;
                }
                cursor_byte += line.len() + 1; // +1 for newline
            }
            cursor_byte = cursor_byte.min(tab.content.len());

            // Build markdown image link with relative path
            let markdown_link = format!("![](assets/{})", new_filename);
            let link_len = markdown_link.chars().count();

            // Insert at cursor position
            tab.content.insert_str(cursor_byte, &markdown_link);

            // Position cursor after the inserted link
            let cursor_char_pos = tab.content[..cursor_byte].chars().count();
            let new_cursor_pos = cursor_char_pos + link_len;
            tab.pending_cursor_restore = Some(new_cursor_pos);
            tab.cursors
                .set_single(crate::state::Selection::cursor(new_cursor_pos));
            tab.sync_cursor_from_primary();

            // Record for undo
            tab.record_edit(old_content, old_cursor);

            debug!(
                "Inserted image link '{}' at line {} col {}",
                markdown_link, cursor_line, cursor_col
            );
        }

        Ok(())
    }

    /// Handle file paths received from secondary Ferrite instances.
    ///
    /// When the user double-clicks a file in the OS while Ferrite is already running,
    /// the second process forwards the path here via the single-instance TCP protocol.
    /// This opens the file as a new tab and brings the window to the front.
    pub(crate) fn handle_instance_paths(&mut self, ctx: &egui::Context) {
        let paths = match &self.instance_listener {
            Some(listener) => listener.poll(),
            None => return,
        };

        if paths.is_empty() {
            return;
        }

        info!("Received {} path(s) from secondary instance", paths.len());

        // Bring this window to the front
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);

        let time = self.get_app_time();
        let mut opened = 0;

        for path in paths {
            if path.is_dir() {
                // Open as workspace
                info!("Opening workspace from secondary instance: {}", path.display());
                match self.state.open_workspace(path.clone()) {
                    Ok(_) => {
                        let folder_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("folder");
                        self.state.show_toast(
                            format!("Opened workspace: {}", folder_name),
                            time,
                            2.5,
                        );
                        self.force_session_save();
                    }
                    Err(e) => {
                        warn!("Failed to open workspace from secondary instance: {}", e);
                    }
                }
            } else if path.is_file() {
                // Open as tab
                match self.state.open_file(path.clone(), Some(time)) {
                    Ok(tab_index) => {
                        self.pending_cjk_check = true;
                        self.check_auto_save_recovery(tab_index);
                        opened += 1;
                        debug!("Opened file from secondary instance: {}", path.display());
                    }
                    Err(e) => {
                        warn!("Failed to open file from secondary instance: {}", e);
                    }
                }
            } else {
                warn!("Path from secondary instance does not exist: {}", path.display());
            }
        }

        if opened > 0 {
            let msg = if opened == 1 {
                "Opened file from external request".to_string()
            } else {
                format!("Opened {} files from external request", opened)
            };
            self.state.show_toast(msg, time, 2.5);
        }
    }

    /// Handle files/folders dropped onto the application window.
    pub(crate) fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped_files: Vec<std::path::PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });

        if dropped_files.is_empty() {
            return;
        }

        // Categorize dropped items
        let mut folders: Vec<std::path::PathBuf> = Vec::new();
        let mut images: Vec<std::path::PathBuf> = Vec::new();
        let mut documents: Vec<std::path::PathBuf> = Vec::new();

        for path in dropped_files {
            if path.is_dir() {
                folders.push(path);
            } else if path.is_file() {
                if Self::is_supported_image(&path) {
                    images.push(path);
                } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if matches!(
                        ext.to_lowercase().as_str(),
                        "md" | "markdown" | "mdown" | "mkd" | "mkdn" | "txt" | "csv" | "tsv" | "json" | "yaml" | "yml" | "toml"
                    ) {
                        documents.push(path);
                    }
                }
            }
        }

        // Priority 1: If a folder was dropped, open it as a workspace
        if let Some(folder) = folders.into_iter().next() {
            info!("Opening dropped folder as workspace: {}", folder.display());
            let folder_path = folder.clone();
            match self.state.open_workspace(folder.clone()) {
                Ok(_) => {
                    let time = self.get_app_time();
                    let folder_name = folder
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("folder");
                    self.state
                        .show_toast(format!("Opened workspace: {}", folder_name), time, 2.5);

                    // Auto-load terminal layout if enabled
                    if self.state.settings.terminal_auto_load_layout {
                        let layout_path = folder_path.join("terminal_layout.json");
                        if layout_path.exists() {
                            if let Ok(json) = std::fs::read_to_string(layout_path) {
                                if let Ok(workspace) = serde_json::from_str::<crate::terminal::SavedWorkspace>(&json) {
                                    match self.terminal_panel_state.manager.load_workspace(workspace) {
                                        Ok(fws) => {
                                            self.terminal_panel_state.floating_windows.clear();
                                            for (layout, title, pos, size) in fws {
                                                let leaf = layout.first_leaf();
                                                let id = egui::ViewportId::from_hash_of(egui::Id::new("floating_term").with(leaf));
                                                self.terminal_panel_state.floating_windows.push(crate::ui::FloatingWindow {
                                                    id,
                                                    layout,
                                                    title,
                                                    pos: pos.map(|(x, y)| egui::pos2(x, y)),
                                                    size: egui::vec2(size.0, size.1),
                                                    first_frame: true,
                                                });
                                            }
                                            info!("Auto-loaded terminal layout from workspace root");
                                        }
                                        Err(e) => warn!("Failed to auto-load terminal layout: {}", e),
                                    }
                                }
                            }
                        }
                    }

                    // Immediately save session to persist the workspace path
                    self.force_session_save();
                }
                Err(e) => {
                    warn!("Failed to open workspace: {}", e);
                    self.state
                        .show_error(format!("Failed to open workspace:\n{}", e));
                }
            }
            return; // Prioritize folder over files
        }

        // Priority 2: Handle images (copy to assets and insert markdown links)
        let mut images_inserted = 0;
        for image_path in images {
            match self.handle_dropped_image(&image_path) {
                Ok(_) => {
                    images_inserted += 1;
                }
                Err(e) => {
                    warn!("Failed to handle dropped image: {}", e);
                    self.state.show_error(format!("Failed to add image:\n{}", e));
                }
            }
        }

        if images_inserted > 0 {
            let time = self.get_app_time();
            let msg = if images_inserted == 1 {
                "Image added to assets".to_string()
            } else {
                format!("{} images added to assets", images_inserted)
            };
            self.state.show_toast(msg, time, 2.5);
        }

        // Priority 3: Open document files in tabs
        let time = self.get_app_time();
        for file in documents {
            match self.state.open_file(file.clone(), Some(time)) {
                Ok(_) => {
                    self.pending_cjk_check = true;
                    debug!("Opened dropped file: {}", file.display());
                    // Add to workspace recent files if in workspace mode
                    if let Some(workspace) = self.state.workspace_mut() {
                        workspace.add_recent_file(file);
                    }
                }
                Err(e) => {
                    warn!("Failed to open dropped file: {}", e);
                }
            }
        }
    }

    /// Handle file tree context menu actions.
    pub(crate) fn handle_file_tree_context_action(&mut self, action: FileTreeContextAction) {
        match action {
            FileTreeContextAction::NewFile(parent_path) => {
                self.file_operation_dialog = Some(FileOperationDialog::new_file(parent_path));
            }
            FileTreeContextAction::NewFolder(parent_path) => {
                self.file_operation_dialog = Some(FileOperationDialog::new_folder(parent_path));
            }
            FileTreeContextAction::Rename(path) => {
                self.file_operation_dialog = Some(FileOperationDialog::rename(path));
            }
            FileTreeContextAction::Delete(path) => {
                self.file_operation_dialog = Some(FileOperationDialog::delete(path));
            }
            FileTreeContextAction::RevealInExplorer(path) => {
                // Open the file's parent folder in the system file explorer
                let folder = if path.is_dir() {
                    path.clone()
                } else {
                    path.parent().map(|p| p.to_path_buf()).unwrap_or(path)
                };

                if let Err(e) = open::that(&folder) {
                    warn!("Failed to reveal in explorer: {}", e);
                    self.state
                        .show_error(format!("Failed to open explorer:\n{}", e));
                } else {
                    debug!("Revealed in explorer: {}", folder.display());
                }
            }
            FileTreeContextAction::Refresh => {
                self.state.refresh_workspace();
                let time = self.get_app_time();
                self.state.show_toast("File tree refreshed", time, 1.5);
            }
        }
    }

    /// Handle creating a new file.
    pub(crate) fn handle_create_file(&mut self, path: std::path::PathBuf) {
        use std::fs::File;
        use std::io::Write;

        // Create the file with default markdown content
        let default_content = format!(
            "# {}\n\n",
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
        );

        match File::create(&path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(default_content.as_bytes()) {
                    warn!("Failed to write to new file: {}", e);
                    self.state
                        .show_error(format!("Failed to write file:\n{}", e));
                    return;
                }

                info!("Created new file: {}", path.display());
                let time = self.get_app_time();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
                self.state
                    .show_toast(format!("Created: {}", name), time, 2.0);

                // Refresh file tree
                self.state.refresh_workspace();

                // Open the new file in a tab
                let time = self.get_app_time();
                match self.state.open_file(path.clone(), Some(time)) {
                    Ok(_) => {
                        self.pending_cjk_check = true;
                    }
                    Err(e) => {
                        warn!("Failed to open new file: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to create file: {}", e);
                self.state
                    .show_error(format!("Failed to create file:\n{}", e));
            }
        }
    }

    /// Handle creating a new folder.
    pub(crate) fn handle_create_folder(&mut self, path: std::path::PathBuf) {
        match std::fs::create_dir(&path) {
            Ok(_) => {
                info!("Created new folder: {}", path.display());
                let time = self.get_app_time();
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("folder");
                self.state
                    .show_toast(format!("Created: {}", name), time, 2.0);

                // Refresh file tree
                self.state.refresh_workspace();
            }
            Err(e) => {
                warn!("Failed to create folder: {}", e);
                self.state
                    .show_error(format!("Failed to create folder:\n{}", e));
            }
        }
    }

    /// Handle renaming a file or folder.
    pub(crate) fn handle_rename_file(&mut self, old_path: std::path::PathBuf, new_path: std::path::PathBuf) {
        match std::fs::rename(&old_path, &new_path) {
            Ok(_) => {
                info!("Renamed: {} -> {}", old_path.display(), new_path.display());
                let time = self.get_app_time();
                let new_name = new_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("item");
                self.state
                    .show_toast(format!("Renamed to: {}", new_name), time, 2.0);

                // Update any open tabs with the old path
                for i in 0..self.state.tab_count() {
                    if let Some(tab) = self.state.tab_mut(i) {
                        if tab.path.as_ref() == Some(&old_path) {
                            tab.path = Some(new_path.clone());
                            break;
                        }
                    }
                }

                // Refresh file tree
                self.state.refresh_workspace();
            }
            Err(e) => {
                warn!("Failed to rename: {}", e);
                self.state.show_error(format!("Failed to rename:\n{}", e));
            }
        }
    }

    /// Handle deleting a file or folder.
    ///
    /// # Parameters
    /// - `path` - Path to the file or folder to delete
    /// - `ctx` - Optional egui Context for cleaning up tab state memory
    pub(crate) fn handle_delete_file(&mut self, path: std::path::PathBuf, ctx: Option<&egui::Context>) {
        let is_dir = path.is_dir();
        let result = if is_dir {
            std::fs::remove_dir_all(&path)
        } else {
            std::fs::remove_file(&path)
        };

        match result {
            Ok(_) => {
                info!("Deleted: {}", path.display());
                let time = self.get_app_time();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("item");
                self.state
                    .show_toast(format!("Deleted: {}", name), time, 2.0);

                // Close any tabs with this path
                // Collect both index and tab_id for cleanup after closing
                let tabs_to_close: Vec<(usize, usize)> = self
                    .state
                    .tabs()
                    .iter()
                    .enumerate()
                    .filter(|(_, tab)| {
                        if let Some(tab_path) = &tab.path {
                            tab_path == &path || tab_path.starts_with(&path)
                        } else {
                            false
                        }
                    })
                    .map(|(i, tab)| (i, tab.id))
                    .collect();

                // Close tabs in reverse order to maintain indices
                for &(index, tab_id) in tabs_to_close.iter().rev() {
                    self.state.close_tab(index);
                    self.cleanup_tab_state(tab_id, ctx);
                }

                // Refresh file tree
                self.state.refresh_workspace();
            }
            Err(e) => {
                warn!("Failed to delete: {}", e);
                self.state.show_error(format!("Failed to delete:\n{}", e));
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Wikilink Navigation
    // ─────────────────────────────────────────────────────────────────────────

    /// Navigate to a wikilink target by resolving it to a file path and opening it.
    ///
    /// Resolution order:
    /// 1. Relative to the current file's directory (with and without `.md`)
    /// 2. Relative to the workspace root (if in workspace mode)
    /// 3. If not found, show an error toast
    pub(crate) fn navigate_wikilink(&mut self, target: &str) {
        let current_dir = self
            .state
            .active_tab()
            .and_then(|tab| tab.path.as_ref())
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf());

        let workspace_root = self.state.workspace_root().cloned();

        // Build candidate paths
        let resolved = resolve_wikilink_target(target, current_dir.as_deref(), workspace_root.as_deref());

        match resolved {
            Some(path) => {
                info!("Wikilink [[{}]] resolved to: {}", target, path.display());
                let time = self.get_app_time();
                match self.state.open_file(path.clone(), Some(time)) {
                    Ok(tab_index) => {
                        self.pending_cjk_check = true;
                        self.check_auto_save_recovery(tab_index);
                        debug!("Opened wikilink target in tab {}", tab_index);
                    }
                    Err(e) => {
                        warn!("Failed to open wikilink target '{}': {}", target, e);
                        let time = self.get_app_time();
                        self.state.show_toast(
                            format!("Could not open [[{}]]: {}", target, e),
                            time,
                            3.0,
                        );
                    }
                }
            }
            None => {
                warn!("Wikilink target '{}' not found", target);
                let time = self.get_app_time();
                self.state.show_toast(
                    format!("[[{}]] — file not found", target),
                    time,
                    3.0,
                );
            }
        }
    }
}

/// Resolve a wikilink target string to a file path.
///
/// Tries these candidates in order:
/// 1. `{current_dir}/{target}` (exact)
/// 2. `{current_dir}/{target}.md`
/// 3. `{workspace_root}/{target}` (exact)
/// 4. `{workspace_root}/{target}.md`
/// 5. Recursive search in workspace for `{target}.md` (same-folder-first, shortest path)
///
/// Returns the first existing path found, or `None`.
fn resolve_wikilink_target(
    target: &str,
    current_dir: Option<&Path>,
    workspace_root: Option<&Path>,
) -> Option<PathBuf> {
    // Normalize the target: trim whitespace
    let target = target.trim();
    if target.is_empty() {
        return None;
    }

    // Helper: check exact path and path with .md extension
    let check_with_md = |dir: &Path| -> Option<PathBuf> {
        // Exact match first
        let exact = dir.join(target);
        if exact.is_file() {
            return Some(exact);
        }
        // With .md extension (only if target doesn't already end with .md)
        if !target.to_lowercase().ends_with(".md") {
            let with_md = dir.join(format!("{}.md", target));
            if with_md.is_file() {
                return Some(with_md);
            }
        }
        None
    };

    // 1. Relative to current file's directory
    if let Some(dir) = current_dir {
        if let Some(found) = check_with_md(dir) {
            return Some(found);
        }
    }

    // 2. Relative to workspace root
    if let Some(root) = workspace_root {
        if let Some(found) = check_with_md(root) {
            return Some(found);
        }

        // 3. Recursive search in workspace for matching file
        // Build the expected filename
        let filename_md = if target.to_lowercase().ends_with(".md") {
            target.to_string()
        } else {
            format!("{}.md", target)
        };
        let filename_lower = filename_md.to_lowercase();

        // Walk the workspace looking for the file
        let mut candidates: Vec<PathBuf> = Vec::new();
        collect_matching_files(root, &filename_lower, &mut candidates);

        if !candidates.is_empty() {
            // Tie-breaking: prefer same folder, then shortest path
            candidates.sort_by(|a, b| {
                let a_same_dir = current_dir.map_or(false, |d| a.parent() == Some(d));
                let b_same_dir = current_dir.map_or(false, |d| b.parent() == Some(d));
                // Same-folder first
                b_same_dir.cmp(&a_same_dir)
                    .then_with(|| {
                        // Shorter path wins
                        a.components().count().cmp(&b.components().count())
                    })
            });
            return Some(candidates.into_iter().next().unwrap());
        }
    }

    None
}

/// Recursively collect files matching a given lowercase filename.
fn collect_matching_files(dir: &Path, filename_lower: &str, results: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden directories and common non-content dirs
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') || name == "node_modules" || name == "target" {
                    continue;
                }
            }
            collect_matching_files(&path, filename_lower, results);
        } else if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.to_lowercase() == filename_lower {
                    results.push(path);
                }
            }
        }
    }
}
