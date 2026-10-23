//! Atomic file writes for document saves.
//!
//! A plain `fs::write` truncates the destination before writing, so a crash,
//! power loss, or full disk mid-write leaves the user's document half-written.
//! This module writes to a sibling temp file, fsyncs it, then renames it over
//! the target — `rename(2)` is atomic within a filesystem, so readers either
//! see the old file or the complete new one.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Write `bytes` to `path` atomically, preserving the target's permissions.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
    let dir = parent.unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "path has no file name"))?;

    let tmp_path = temp_path(dir, &file_name.to_string_lossy());

    let result = (|| {
        let mut file = File::create(&tmp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;

        // Keep the original mode (rename would otherwise apply the temp file's 0600/0644).
        if let Ok(meta) = fs::metadata(path) {
            let _ = fs::set_permissions(&tmp_path, meta.permissions());
        }

        fs::rename(&tmp_path, path)?;
        // Best-effort directory fsync so the rename survives a power cut.
        if let Ok(dir_handle) = File::open(dir) {
            let _ = dir_handle.sync_all();
        }
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    result
}

fn temp_path(dir: &Path, file_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    dir.join(format!(".{file_name}.tmp-{}-{nonce}", std::process::id()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_new_file_and_leaves_no_temp() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("note.md");
        write_atomic(&path, b"# hello").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"# hello");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn replaces_existing_content() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("note.md");
        fs::write(&path, "old").unwrap();
        write_atomic(&path, b"new content").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "new content");
    }

    #[test]
    fn preserves_unix_mode() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("script.md");
        fs::write(&path, "old").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).unwrap();
        write_atomic(&path, b"new").unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o640);
    }
}
