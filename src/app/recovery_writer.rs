//! Single background writer for crash-recovery snapshots.
//!
//! The UI thread captures metadata + changed tab bodies and submits one job;
//! JSON serialization, fsync and file writes happen on this worker so a large
//! document cannot stall a frame.

use crate::config::{
    save_crash_recovery_state, save_recovery_content, RecoveryTabJob, SessionState,
};
use log::{debug, warn};
use std::sync::mpsc::{self, Sender};
use std::thread::JoinHandle;

pub(crate) struct RecoveryJob {
    pub session: SessionState,
    pub tabs: Vec<RecoveryTabJob>,
}

pub(crate) struct RecoveryWriter {
    tx: Option<Sender<RecoveryJob>>,
    handle: Option<JoinHandle<()>>,
}

impl RecoveryWriter {
    pub(crate) fn new() -> Self {
        let (tx, rx) = mpsc::channel::<RecoveryJob>();
        let handle = std::thread::Builder::new()
            .name("mdoxide-recovery-writer".to_string())
            .spawn(move || {
                while let Ok(job) = rx.recv() {
                    if !save_crash_recovery_state(&job.session) {
                        warn!("Recovery writer: failed to save session metadata");
                        continue;
                    }
                    for tab in job.tabs {
                        if !save_recovery_content(
                            tab.tab_id,
                            &tab.content,
                            tab.path.as_deref(),
                            tab.disk_hash,
                        ) {
                            warn!(
                                "Recovery writer: failed to save recovery content for tab {}",
                                tab.tab_id
                            );
                        }
                    }
                    debug!("Recovery writer: snapshot written");
                }
            })
            .ok();

        Self { tx: Some(tx), handle }
    }

    /// Queue a snapshot. Returns false if the worker is gone.
    pub(crate) fn submit(&self, job: RecoveryJob) -> bool {
        self.tx
            .as_ref()
            .is_some_and(|tx| tx.send(job).is_ok())
    }

    /// Drop the queue sender and wait for any in-flight write. Call before the
    /// synchronous shutdown write so an older snapshot cannot overwrite it.
    pub(crate) fn shutdown(&mut self) {
        self.tx.take();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
