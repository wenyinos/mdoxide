//! Version Control System integration for Ferrite
//!
//! This module provides Git integration features including:
//! - Repository detection
//! - Current branch display
//! - File status tracking (modified, staged, untracked, ignored)
//! - Automatic status refresh on save, focus, and periodic intervals

mod git;

pub use git::{GitAutoRefresh, GitFileStatus, GitService};
