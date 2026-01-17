//! Path Utilities
//!
//! This module provides utilities for normalizing file paths, particularly
//! on Windows where `canonicalize()` returns verbatim paths with `\\?\` prefix.
//!
//! # Problem
//! On Windows, `std::fs::canonicalize()` returns paths with the extended-length
//! path prefix `\\?\` (e.g., `\\?\G:\DEV\project` instead of `G:\DEV\project`).
//!
//! This causes several issues:
//! - Paths appear confusing to users with the `\\?\` prefix
//! - Same path can appear as duplicates (with and without prefix)
//! - Some libraries (like git2) may not handle verbatim paths properly
//! - Path comparisons fail because `\\?\G:\path` != `G:\path`
//!
//! # Solution
//! Use `normalize_path()` after `canonicalize()` to strip the verbatim prefix
//! and ensure consistent path representation throughout the application.
//!
//! # Example
//! ```ignore
//! use crate::path_utils::normalize_path;
//!
//! let canonical = path.canonicalize()?;
//! let normalized = normalize_path(canonical);
//! // normalized is now "G:\DEV\project" instead of "\\?\G:\DEV\project"
//! ```

use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────────────────────────
// Path Normalization
// ─────────────────────────────────────────────────────────────────────────────

/// Normalize a path by stripping Windows extended-length path prefixes.
///
/// On Windows, this removes the `\\?\` prefix that `canonicalize()` adds.
/// On other platforms, this is a no-op and returns the path unchanged.
///
/// # Windows Extended-Length Paths
/// Windows uses these prefixes for extended-length paths:
/// - `\\?\` - Verbatim disk path (e.g., `\\?\C:\path`)
/// - `\\.\` - Verbatim device path (e.g., `\\.\COM1`)
/// - `\??\` - NT namespace path
///
/// This function handles the common `\\?\` prefix that `canonicalize()` adds.
///
/// # Example
/// ```ignore
/// use std::path::PathBuf;
/// use crate::path_utils::normalize_path;
///
/// let path = PathBuf::from(r"\\?\G:\DEV\project");
/// let normalized = normalize_path(path);
/// assert_eq!(normalized, PathBuf::from(r"G:\DEV\project"));
/// ```
pub fn normalize_path(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        normalize_windows_path(path)
    }

    #[cfg(not(windows))]
    {
        path
    }
}

/// Normalize a path reference, returning an owned PathBuf.
///
/// This is useful when you have a path reference and need a normalized owned path.
#[allow(dead_code)] // Public API for path normalization
pub fn normalize_path_ref(path: &Path) -> PathBuf {
    normalize_path(path.to_path_buf())
}

#[cfg(windows)]
fn normalize_windows_path(path: PathBuf) -> PathBuf {
    use std::path::Prefix;

    // Check if the path has a verbatim prefix
    if let Some(std::path::Component::Prefix(prefix)) = path.components().next() {
        match prefix.kind() {
            // \\?\C:\... -> C:\...
            Prefix::VerbatimDisk(disk) => {
                let drive = (disk as char).to_ascii_uppercase();
                let rest: PathBuf = path
                    .components()
                    .skip(1) // Skip the prefix
                    .collect();

                let mut normalized = PathBuf::from(format!("{}:", drive));
                if !rest.as_os_str().is_empty() {
                    normalized.push(rest);
                } else {
                    // Ensure we have a root (C: -> C:\)
                    normalized.push(std::path::MAIN_SEPARATOR.to_string());
                }
                return normalized;
            }
            // \\?\UNC\server\share -> \\server\share
            Prefix::VerbatimUNC(server, share) => {
                let rest: PathBuf = path.components().skip(1).collect();
                let mut normalized = PathBuf::from(format!(
                    r"\\{}\{}",
                    server.to_string_lossy(),
                    share.to_string_lossy()
                ));
                if !rest.as_os_str().is_empty() {
                    normalized.push(rest);
                }
                return normalized;
            }
            // Other prefixes (Disk, UNC, etc.) are already normalized
            _ => {}
        }
    }

    // Path doesn't have a verbatim prefix, return as-is
    path
}

/// Canonicalize a path and normalize it to remove Windows verbatim prefixes.
///
/// This is a convenience function that combines `canonicalize()` and `normalize_path()`.
/// Returns `None` if canonicalization fails (e.g., path doesn't exist).
///
/// # Example
/// ```ignore
/// use crate::path_utils::canonicalize_and_normalize;
///
/// if let Some(path) = canonicalize_and_normalize(&some_path) {
///     // path is fully resolved and normalized (no \\?\ prefix)
/// }
/// ```
#[allow(dead_code)] // Public API for path canonicalization
pub fn canonicalize_and_normalize(path: &Path) -> Option<PathBuf> {
    path.canonicalize().ok().map(normalize_path)
}

/// Canonicalize a path with fallback, and normalize the result.
///
/// If canonicalization fails, returns the original path (potentially cleaned).
/// Always normalizes the result to remove Windows verbatim prefixes.
///
/// # Example
/// ```ignore
/// use crate::path_utils::canonicalize_or_normalize;
///
/// // Even if path doesn't exist, returns a usable path
/// let path = canonicalize_or_normalize(&some_path);
/// ```
#[allow(dead_code)] // Public API for path canonicalization
pub fn canonicalize_or_normalize(path: &Path) -> PathBuf {
    match path.canonicalize() {
        Ok(canonical) => normalize_path(canonical),
        Err(_) => {
            // If canonicalization fails, try to at least normalize what we have
            // This handles the case where the file doesn't exist yet
            normalize_path(path.to_path_buf())
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_regular_path() {
        // Regular paths should be unchanged
        let path = PathBuf::from("/home/user/project");
        assert_eq!(normalize_path(path.clone()), path);
    }

    #[cfg(windows)]
    mod windows_tests {
        use super::*;

        #[test]
        fn test_normalize_verbatim_disk_path() {
            let path = PathBuf::from(r"\\?\G:\DEV\markDownNotepad");
            let normalized = normalize_path(path);
            assert_eq!(normalized, PathBuf::from(r"G:\DEV\markDownNotepad"));
        }

        #[test]
        fn test_normalize_verbatim_disk_root() {
            let path = PathBuf::from(r"\\?\C:");
            let normalized = normalize_path(path);
            // Should have trailing separator for root
            assert!(normalized.to_string_lossy().starts_with("C:"));
        }

        #[test]
        fn test_normalize_regular_windows_path() {
            // Regular Windows paths should be unchanged
            let path = PathBuf::from(r"G:\DEV\project");
            assert_eq!(normalize_path(path.clone()), path);
        }

        #[test]
        fn test_normalize_unc_path() {
            // Regular UNC paths should be unchanged
            let path = PathBuf::from(r"\\server\share\folder");
            assert_eq!(normalize_path(path.clone()), path);
        }

        #[test]
        fn test_normalize_lowercase_drive() {
            // Should normalize drive letter to uppercase
            let path = PathBuf::from(r"\\?\g:\dev\project");
            let normalized = normalize_path(path);
            assert!(normalized.to_string_lossy().starts_with("G:"));
        }

        #[test]
        fn test_canonicalize_and_normalize() {
            // Test with current directory (which should exist)
            if let Some(normalized) = canonicalize_and_normalize(Path::new(".")) {
                // Should not contain \\?\
                assert!(
                    !normalized.to_string_lossy().contains(r"\\?\"),
                    "Path should not contain verbatim prefix: {:?}",
                    normalized
                );
            }
        }

        #[test]
        fn test_canonicalize_or_normalize_nonexistent() {
            // Even for non-existent paths, should return something usable
            let path = Path::new(r"\\?\C:\nonexistent\path");
            let result = canonicalize_or_normalize(path);
            // Should strip the prefix even if canonicalization fails
            assert!(
                !result.to_string_lossy().starts_with(r"\\?\"),
                "Path should not start with verbatim prefix: {:?}",
                result
            );
        }
    }

    #[test]
    fn test_normalize_path_ref() {
        let path = PathBuf::from("/some/path");
        let normalized = normalize_path_ref(&path);
        assert_eq!(normalized, path);
    }
}
