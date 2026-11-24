//! Atomic replacement of a file: temporary file, `fsync`, `rename`, then `fsync` of the folder.
//! A crash at any point leaves either the old or the new content under the real name; at worst a
//! stale `.tmp` file stays behind and is overwritten by the next write.

use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// `<file>.tmp` next to the file.
pub(crate) fn tmp_path(path: &Path) -> PathBuf {
    with_suffix(path, ".tmp")
}

/// `<file>.bak` next to the file: the previous version.
pub fn backup_path(path: &Path) -> PathBuf {
    with_suffix(path, ".bak")
}

/// `<file>.broken`: where a damaged file is moved when its backup is restored.
pub(crate) fn broken_path(path: &Path) -> PathBuf {
    with_suffix(path, ".broken")
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name: OsString = path.file_name().unwrap_or_default().to_owned();
    name.push(suffix);
    path.with_file_name(name)
}

/// Replace `path` with `bytes` atomically. With `backup`, the current file is kept as `.bak`
/// first (as a hard link, or a copy where links are not possible).
pub(crate) fn write_atomic(path: &Path, bytes: &[u8], backup: bool) -> io::Result<()> {
    let tmp = tmp_path(path);
    {
        let mut file = File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    if backup && path.exists() {
        let bak = backup_path(path);
        match fs::remove_file(&bak) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
        if fs::hard_link(path, &bak).is_err() {
            fs::copy(path, &bak)?;
        }
    }
    fs::rename(&tmp, path)?;
    sync_dir(path)
}

/// Make the rename itself durable.
fn sync_dir(path: &Path) -> io::Result<()> {
    match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => File::open(dir)?.sync_all(),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_content_and_keeps_the_previous_as_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("items.json");
        write_atomic(&path, b"one", true).unwrap();
        assert!(!backup_path(&path).exists());
        write_atomic(&path, b"two", true).unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"two");
        assert_eq!(fs::read(backup_path(&path)).unwrap(), b"one");
        write_atomic(&path, b"three", false).unwrap();
        assert_eq!(fs::read(backup_path(&path)).unwrap(), b"one");
        assert!(!tmp_path(&path).exists());
    }

    #[test]
    fn a_stale_temporary_file_is_overwritten() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("items.json");
        fs::write(tmp_path(&path), b"half a fi").unwrap();
        write_atomic(&path, b"whole", true).unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"whole");
        assert!(!tmp_path(&path).exists());
    }
}
