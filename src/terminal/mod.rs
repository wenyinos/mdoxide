//! Terminal emulation module for Ferrite
//!
//! This module provides an integrated terminal emulator using:
//! - `portable-pty` for cross-platform pseudo-terminal handling
//! - `vte` for ANSI escape sequence parsing
//!
//! The terminal supports:
//! - Full ANSI color support (16, 256, and true color)
//! - Scrollback buffer
//! - Multiple terminal instances (tabs)
//! - Cross-platform shell spawning (cmd/PowerShell on Windows, bash/zsh on Unix)

mod handler;
mod pty;
mod screen;
mod widget;

pub use handler::TerminalHandler;
pub use pty::TerminalPty;
pub use screen::TerminalScreen;
pub use widget::TerminalWidget;

use std::sync::{Arc, Mutex};

/// Terminal instance that combines PTY, screen buffer, and VTE parser.
pub struct Terminal {
    /// Pseudo-terminal for shell communication
    pty: TerminalPty,
    /// Screen buffer for terminal content
    screen: Arc<Mutex<TerminalScreen>>,
    /// VTE parser for ANSI escape sequences
    parser: vte::Parser,
    /// Terminal title (from OSC sequences)
    title: String,
    /// Unique ID for this terminal
    id: usize,
    /// Whether the terminal is active/running
    running: bool,
}

impl Terminal {
    /// Create a new terminal instance with the given ID and optional working directory.
    pub fn new(id: usize, cols: u16, rows: u16, working_dir: Option<std::path::PathBuf>) -> Result<Self, String> {
        let screen = Arc::new(Mutex::new(TerminalScreen::new(cols, rows)));
        let pty = TerminalPty::new(cols, rows, working_dir)?;
        
        Ok(Self {
            pty,
            screen,
            parser: vte::Parser::new(),
            title: format!("Terminal {}", id),
            id,
            running: true,
        })
    }

    /// Get the terminal ID.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Get the terminal title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set the terminal title.
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    /// Check if the terminal is still running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get mutable access to the screen buffer.
    pub fn screen(&self) -> &Arc<Mutex<TerminalScreen>> {
        &self.screen
    }

    /// Process input from the user (keyboard).
    pub fn write_input(&mut self, data: &[u8]) {
        if let Err(e) = self.pty.write(data) {
            log::warn!("Failed to write to terminal: {}", e);
        }
    }

    /// Write a string to the terminal.
    pub fn write_str(&mut self, s: &str) {
        self.write_input(s.as_bytes());
    }

    /// Read and process output from the PTY.
    /// Returns true if new data was processed.
    pub fn poll(&mut self) -> bool {
        let mut processed = false;
        
        // Read available data from PTY
        match self.pty.read() {
            Ok(Some(data)) => {
                // Parse through VTE
                let mut screen = self.screen.lock().unwrap();
                let mut handler = TerminalHandler::new(&mut screen);
                
                for byte in data {
                    self.parser.advance(&mut handler, byte);
                }
                
                // Check for title updates
                if let Some(title) = handler.take_title() {
                    self.title = title;
                }
                
                processed = true;
            }
            Ok(None) => {
                // No data available
            }
            Err(e) => {
                log::debug!("PTY read error (may be closed): {}", e);
                self.running = false;
            }
        }
        
        // Check if process is still running
        if !self.pty.is_running() {
            self.running = false;
        }
        
        processed
    }

    /// Resize the terminal.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        if let Err(e) = self.pty.resize(cols, rows) {
            log::warn!("Failed to resize PTY: {}", e);
        }
        
        let mut screen = self.screen.lock().unwrap();
        screen.resize(cols, rows);
    }

    /// Get the current terminal size (cols, rows).
    pub fn size(&self) -> (u16, u16) {
        let screen = self.screen.lock().unwrap();
        screen.size()
    }
}

/// Manager for multiple terminal instances.
pub struct TerminalManager {
    /// All terminal instances
    terminals: Vec<Terminal>,
    /// Index of the active terminal
    active_index: usize,
    /// Counter for generating unique terminal IDs
    next_id: usize,
    /// Default terminal size
    default_cols: u16,
    default_rows: u16,
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalManager {
    /// Create a new terminal manager.
    pub fn new() -> Self {
        Self {
            terminals: Vec::new(),
            active_index: 0,
            next_id: 1,
            default_cols: 80,
            default_rows: 24,
        }
    }

    /// Set the default terminal size.
    pub fn set_default_size(&mut self, cols: u16, rows: u16) {
        self.default_cols = cols;
        self.default_rows = rows;
    }

    /// Create a new terminal and return its index.
    /// If working_dir is provided, the terminal will start in that directory.
    pub fn create_terminal(&mut self, working_dir: Option<std::path::PathBuf>) -> Result<usize, String> {
        let id = self.next_id;
        self.next_id += 1;

        let terminal = Terminal::new(id, self.default_cols, self.default_rows, working_dir)?;
        self.terminals.push(terminal);
        
        let index = self.terminals.len() - 1;
        self.active_index = index;
        
        log::info!("Created terminal {} (index {})", id, index);
        Ok(index)
    }

    /// Get the active terminal.
    pub fn active_terminal(&self) -> Option<&Terminal> {
        self.terminals.get(self.active_index)
    }

    /// Get mutable access to the active terminal.
    pub fn active_terminal_mut(&mut self) -> Option<&mut Terminal> {
        self.terminals.get_mut(self.active_index)
    }

    /// Get a terminal by index.
    pub fn terminal(&self, index: usize) -> Option<&Terminal> {
        self.terminals.get(index)
    }

    /// Get mutable access to a terminal by index.
    pub fn terminal_mut(&mut self, index: usize) -> Option<&mut Terminal> {
        self.terminals.get_mut(index)
    }

    /// Set the active terminal by index.
    pub fn set_active(&mut self, index: usize) {
        if index < self.terminals.len() {
            self.active_index = index;
        }
    }

    /// Close a terminal by index.
    pub fn close_terminal(&mut self, index: usize) {
        if index < self.terminals.len() {
            self.terminals.remove(index);
            
            // Adjust active index if needed
            if self.active_index >= self.terminals.len() && !self.terminals.is_empty() {
                self.active_index = self.terminals.len() - 1;
            }
        }
    }

    /// Get the number of terminals.
    pub fn terminal_count(&self) -> usize {
        self.terminals.len()
    }

    /// Get the active terminal index.
    pub fn active_index(&self) -> usize {
        self.active_index
    }

    /// Check if there are any terminals.
    pub fn has_terminals(&self) -> bool {
        !self.terminals.is_empty()
    }

    /// Poll all terminals for new data.
    /// Returns true if any terminal had new data.
    pub fn poll_all(&mut self) -> bool {
        let mut any_data = false;
        for terminal in &mut self.terminals {
            if terminal.poll() {
                any_data = true;
            }
        }
        any_data
    }

    /// Get terminal titles for tab display.
    pub fn terminal_titles(&self) -> Vec<(usize, String, bool)> {
        self.terminals
            .iter()
            .enumerate()
            .map(|(i, t)| (i, t.title().to_string(), i == self.active_index))
            .collect()
    }

    /// Resize all terminals to a new size.
    pub fn resize_all(&mut self, cols: u16, rows: u16) {
        self.default_cols = cols;
        self.default_rows = rows;
        
        for terminal in &mut self.terminals {
            terminal.resize(cols, rows);
        }
    }
}
