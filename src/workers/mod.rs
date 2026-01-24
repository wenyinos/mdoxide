//! Generic worker infrastructure for background async operations.
//!
//! This module provides a pattern for running async (tokio) operations in background threads
//! without blocking the main UI thread (egui is single-threaded and must never block).
//!
//! # Architecture
//!
//! - Main thread (egui): Sends commands via mpsc, polls for responses
//! - Worker thread: Runs tokio runtime, processes commands, sends responses
//! - Communication: std::sync::mpsc channels (cross thread-boundary safe)
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::workers::{WorkerHandle, WorkerCommand, WorkerResponse, echo_worker};
//!
//! // Spawn worker in background thread
//! let handle = WorkerHandle::spawn(echo_worker);
//!
//! // Send command from UI thread
//! handle.command_tx.send(WorkerCommand::Echo("Hello".to_string())).unwrap();
//!
//! // Poll for responses (non-blocking)
//! if let Ok(response) = handle.response_rx.try_recv() {
//!     match response {
//!         WorkerResponse::Echo(msg) => println!("{}", msg),
//!         _ => {}
//!     }
//! }
//! ```

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// Commands sent from UI thread to worker thread.
#[derive(Debug, Clone)]
pub enum WorkerCommand {
    /// Echo command for testing the worker pattern.
    Echo(String),
    /// Gracefully shutdown the worker thread.
    Shutdown,
}

/// Responses sent from worker thread to UI thread.
#[derive(Debug, Clone)]
pub enum WorkerResponse {
    /// Worker initialized and ready.
    Ready,
    /// Echo response with processed message.
    Echo(String),
}

/// Handle for communicating with a background worker thread.
///
/// The worker runs a tokio runtime in its own thread, separate from the main UI thread.
pub struct WorkerHandle {
    /// Channel to send commands to the worker.
    pub command_tx: Sender<WorkerCommand>,
    /// Channel to receive responses from the worker.
    pub response_rx: Receiver<WorkerResponse>,
}

impl WorkerHandle {
    /// Spawn a new worker thread running the given worker function.
    ///
    /// # Arguments
    ///
    /// * `worker_fn` - Function that receives command/response channels and runs the worker loop.
    ///                 This function will be called in a new background thread.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let handle = WorkerHandle::spawn(|cmd_rx, resp_tx| {
    ///     // Create tokio runtime in THIS thread (not main)
    ///     let rt = tokio::runtime::Runtime::new().unwrap();
    ///
    ///     // Send ready signal
    ///     resp_tx.send(WorkerResponse::Ready).unwrap();
    ///
    ///     // Process commands...
    /// });
    /// ```
    pub fn spawn<F>(worker_fn: F) -> Self
    where
        F: FnOnce(Receiver<WorkerCommand>, Sender<WorkerResponse>) + Send + 'static,
    {
        // Create channels for bidirectional communication
        // Using std::sync::mpsc (NOT tokio::sync::mpsc) because we cross thread boundaries
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (resp_tx, resp_rx) = mpsc::channel();

        // Spawn background thread to run the worker
        thread::spawn(move || {
            worker_fn(cmd_rx, resp_tx);
        });

        Self {
            command_tx: cmd_tx,
            response_rx: resp_rx,
        }
    }

    /// Send a command to the worker (non-blocking).
    ///
    /// Returns error if worker thread has terminated.
    pub fn send_command(&self, cmd: WorkerCommand) -> Result<(), mpsc::SendError<WorkerCommand>> {
        self.command_tx.send(cmd)
    }

    /// Try to receive a response from the worker (non-blocking).
    ///
    /// Returns `Ok(response)` if one is available, `Err` if none ready.
    pub fn try_recv_response(&self) -> Result<WorkerResponse, mpsc::TryRecvError> {
        self.response_rx.try_recv()
    }

    /// Shutdown the worker gracefully.
    pub fn shutdown(&self) {
        let _ = self.send_command(WorkerCommand::Shutdown);
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        // Attempt graceful shutdown when handle is dropped
        let _ = self.command_tx.send(WorkerCommand::Shutdown);
    }
}
