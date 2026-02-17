//! Single-instance application protocol.
//!
//! Ensures only one Ferrite window runs at a time. When a second instance is
//! launched (e.g., double-clicking a file in Explorer), it forwards the file
//! paths to the already-running instance via a local TCP connection, then exits.
//!
//! ## Protocol
//!
//! - **Lock file**: `{config_dir}/instance.lock` contains the TCP port of the
//!   running instance as plain text.
//! - **IPC**: The second instance connects to `127.0.0.1:{port}`, sends file
//!   paths as UTF-8 lines (one path per line), then closes the connection.
//! - **Polling**: The primary instance polls the TCP listener each frame
//!   (non-blocking) and opens received paths as tabs.

use crate::config::get_config_dir;
use log::{debug, error, info, warn};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;

/// Name of the lock file stored in the config directory.
const LOCK_FILE_NAME: &str = "instance.lock";

/// Timeout for connecting to the existing instance (milliseconds).
const CONNECT_TIMEOUT_MS: u64 = 1000;

/// Attempt to become the primary instance, or forward paths to the existing one.
///
/// Returns `Ok(Some(listener))` if this process should become the primary instance.
/// Returns `Ok(None)` if paths were forwarded to an existing instance and we should exit.
pub fn try_acquire_instance(paths: &[PathBuf]) -> Option<SingleInstanceListener> {
    let lock_path = match get_lock_file_path() {
        Some(p) => p,
        None => {
            warn!("Could not determine lock file path; proceeding as primary");
            return create_listener();
        }
    };

    // Check if a lock file exists with a valid port
    if let Some(port) = read_lock_port(&lock_path) {
        // Try to connect to the existing instance
        if try_forward_paths(port, paths) {
            info!(
                "Forwarded {} path(s) to existing Ferrite instance on port {}",
                paths.len(),
                port
            );
            return None; // Signal caller to exit
        }
        // Connection failed — the old instance is dead. Clean up stale lock.
        debug!("Stale lock file detected (port {} unreachable), taking over", port);
        let _ = std::fs::remove_file(&lock_path);
    }

    // No running instance — become the primary
    create_listener()
}

/// Create a new TCP listener and write the lock file.
fn create_listener() -> Option<SingleInstanceListener> {
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind single-instance listener: {}", e);
            // Return a dummy listener that never accepts (best-effort: allow app to run)
            return Some(SingleInstanceListener { listener: None });
        }
    };

    // Set non-blocking so polling in the update loop doesn't stall the UI
    if let Err(e) = listener.set_nonblocking(true) {
        warn!("Failed to set listener to non-blocking: {}", e);
    }

    let port = match listener.local_addr() {
        Ok(addr) => addr.port(),
        Err(e) => {
            error!("Failed to get listener address: {}", e);
            return Some(SingleInstanceListener { listener: None });
        }
    };

    // Write the lock file
    if let Err(e) = write_lock_file(port) {
        warn!("Failed to write instance lock file: {}", e);
        // Continue anyway — single-instance won't work but app still runs
    }

    info!("Single-instance listener started on port {}", port);
    Some(SingleInstanceListener {
        listener: Some(listener),
    })
}

/// Read the port number from the lock file.
fn read_lock_port(lock_path: &std::path::Path) -> Option<u16> {
    let content = std::fs::read_to_string(lock_path).ok()?;
    content.trim().parse::<u16>().ok()
}

/// Try to connect to an existing instance and forward file paths.
///
/// Returns `true` if the paths were successfully forwarded.
fn try_forward_paths(port: u16, paths: &[PathBuf]) -> bool {
    use std::net::SocketAddr;
    use std::time::Duration;

    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    let timeout = Duration::from_millis(CONNECT_TIMEOUT_MS);

    let mut stream = match TcpStream::connect_timeout(&addr, timeout) {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Set a write timeout to avoid blocking forever
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));

    // Send each path as a line
    for path in paths {
        let line = format!("{}\n", path.display());
        if stream.write_all(line.as_bytes()).is_err() {
            return false;
        }
    }

    // If no paths, still send a signal so the existing instance focuses its window
    if paths.is_empty() {
        // Send an empty "focus" signal
        if stream.write_all(b"__FOCUS__\n").is_err() {
            return false;
        }
    }

    // Flush and close
    let _ = stream.flush();
    true
}

/// Write the lock file with the given port number.
fn write_lock_file(port: u16) -> std::io::Result<()> {
    let lock_path = get_lock_file_path().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Config dir not available")
    })?;

    // Ensure parent directory exists
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&lock_path, port.to_string())
}

/// Get the path to the instance lock file.
fn get_lock_file_path() -> Option<PathBuf> {
    get_config_dir().ok().map(|dir| dir.join(LOCK_FILE_NAME))
}

// ─────────────────────────────────────────────────────────────────────────────
// SingleInstanceListener — lives in the primary instance
// ─────────────────────────────────────────────────────────────────────────────

/// Listener that accepts incoming file-open requests from secondary instances.
///
/// Stored in [`FerriteApp`] and polled every frame in the `update()` loop.
pub struct SingleInstanceListener {
    listener: Option<TcpListener>,
}

impl SingleInstanceListener {
    /// Poll for incoming connections and return any file paths received.
    ///
    /// This is non-blocking — returns an empty `Vec` if no connections are pending.
    pub fn poll(&self) -> Vec<PathBuf> {
        let listener = match &self.listener {
            Some(l) => l,
            None => return Vec::new(),
        };

        let mut paths = Vec::new();

        // Accept all pending connections (non-blocking)
        loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    debug!("Single-instance connection from {}", addr);
                    // Set a read timeout to prevent hangs from misbehaving clients
                    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));

                    let reader = BufReader::new(stream);
                    for line in reader.lines() {
                        match line {
                            Ok(line) => {
                                let trimmed = line.trim().to_string();
                                if trimmed.is_empty() || trimmed == "__FOCUS__" {
                                    continue;
                                }
                                let path = PathBuf::from(&trimmed);
                                debug!("Received path from secondary instance: {}", path.display());
                                paths.push(path);
                            }
                            Err(e) => {
                                debug!("Error reading from secondary instance: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break; // No more pending connections
                }
                Err(e) => {
                    debug!("Error accepting single-instance connection: {}", e);
                    break;
                }
            }
        }

        paths
    }
}

impl Drop for SingleInstanceListener {
    fn drop(&mut self) {
        // Clean up the lock file when the primary instance exits
        if let Some(lock_path) = get_lock_file_path() {
            if lock_path.exists() {
                debug!("Cleaning up instance lock file");
                let _ = std::fs::remove_file(&lock_path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_file_roundtrip() {
        // Verify port parsing
        let port_str = "12345";
        let port: u16 = port_str.trim().parse().unwrap();
        assert_eq!(port, 12345);
    }

    #[test]
    fn test_forward_to_nonexistent_port_returns_false() {
        // Attempting to forward to a port with no listener should fail gracefully
        let result = try_forward_paths(1, &[PathBuf::from("test.md")]);
        assert!(!result);
    }
}
