//! Pseudo-terminal (PTY) handling for terminal emulation.
//!
//! This module provides cross-platform PTY management using the `portable-pty` crate.
//! It handles spawning shell processes and communicating with them.

use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use std::io::{Read, Write};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;

/// PTY wrapper for terminal communication.
pub struct TerminalPty {
    /// Writer to send data to the PTY
    writer: Box<dyn Write + Send>,
    /// Receiver for data from the PTY (read in background thread)
    reader_rx: Receiver<Vec<u8>>,
    /// Channel to signal the reader thread to stop
    _stop_tx: Sender<()>,
    /// PTY pair (kept for resize operations)
    pty_pair: PtyPair,
    /// Whether the child process is still running
    child_running: bool,
}

impl TerminalPty {
    /// Create a new PTY with the given size and optional working directory.
    pub fn new(cols: u16, rows: u16, working_dir: Option<std::path::PathBuf>) -> Result<Self, String> {
        // Get the native PTY system
        let pty_system = native_pty_system();

        // Create PTY pair with specified size
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        // Build the shell command based on platform
        let cmd = Self::build_shell_command(working_dir);

        // Spawn the shell process
        let _child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        // Get writer for the master side
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to get PTY writer: {}", e))?;

        // Get reader for the master side
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to get PTY reader: {}", e))?;

        // Create channels for communication
        let (data_tx, data_rx) = mpsc::channel::<Vec<u8>>();
        let (stop_tx, stop_rx) = mpsc::channel::<()>();

        // Spawn background thread to read from PTY
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                // Check if we should stop
                match stop_rx.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => {}
                }

                // Read from PTY (non-blocking would be better, but portable-pty doesn't support it well)
                match reader.read(&mut buf) {
                    Ok(0) => {
                        // EOF - process has exited
                        break;
                    }
                    Ok(n) => {
                        if data_tx.send(buf[..n].to_vec()).is_err() {
                            // Receiver dropped
                            break;
                        }
                    }
                    Err(e) => {
                        log::debug!("PTY read error: {}", e);
                        break;
                    }
                }
            }
            log::debug!("PTY reader thread exiting");
        });

        Ok(Self {
            writer,
            reader_rx: data_rx,
            _stop_tx: stop_tx,
            pty_pair: pair,
            child_running: true,
        })
    }

    /// Build the shell command for the current platform.
    fn build_shell_command(working_dir: Option<std::path::PathBuf>) -> CommandBuilder {
        let mut cmd = if cfg!(windows) {
            // On Windows, prefer PowerShell, fall back to cmd
            let shell = std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string());

            // Check if PowerShell is available
            if std::path::Path::new("C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe").exists() {
                CommandBuilder::new("powershell.exe")
            } else {
                CommandBuilder::new(shell)
            }
        } else {
            // On Unix, use SHELL environment variable or fall back to /bin/sh
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
            let mut cmd = CommandBuilder::new(&shell);

            // Add -l flag for login shell behavior (loads profile)
            if shell.contains("bash") || shell.contains("zsh") {
                cmd.arg("-l");
            }

            cmd
        };

        // Set working directory: use provided path, fall back to current dir, then home
        if let Some(dir) = working_dir {
            cmd.cwd(dir);
        } else if let Ok(cwd) = std::env::current_dir() {
            cmd.cwd(cwd);
        } else if let Some(home) = dirs::home_dir() {
            cmd.cwd(home);
        }

        // Set TERM environment variable for proper terminal behavior
        cmd.env("TERM", "xterm-256color");

        // Set COLORTERM for true color support
        cmd.env("COLORTERM", "truecolor");

        cmd
    }

    /// Write data to the PTY (send to shell).
    pub fn write(&mut self, data: &[u8]) -> Result<(), String> {
        self.writer
            .write_all(data)
            .map_err(|e| format!("Failed to write to PTY: {}", e))?;
        self.writer
            .flush()
            .map_err(|e| format!("Failed to flush PTY: {}", e))?;
        Ok(())
    }

    /// Read available data from the PTY (non-blocking).
    /// Returns Ok(Some(data)) if data is available, Ok(None) if no data.
    pub fn read(&mut self) -> Result<Option<Vec<u8>>, String> {
        match self.reader_rx.try_recv() {
            Ok(data) => Ok(Some(data)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => {
                self.child_running = false;
                Err("PTY reader disconnected".to_string())
            }
        }
    }

    /// Check if the child process is still running.
    pub fn is_running(&self) -> bool {
        self.child_running
    }

    /// Resize the PTY.
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<(), String> {
        self.pty_pair
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to resize PTY: {}", e))
    }
}

impl Drop for TerminalPty {
    fn drop(&mut self) {
        // Stop signal is sent automatically when _stop_tx is dropped
        log::debug!("TerminalPty dropped");
    }
}
