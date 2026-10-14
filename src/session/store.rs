//! Reading and writing state files. The directory and files are private to the user.

use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::Record;

const PRIVATE_DIR: u32 = 0o700;
const PRIVATE_FILE: u32 = 0o600;

pub fn path(dir: &Path, session_id: &str) -> PathBuf {
    dir.join(format!("{session_id}.json"))
}

pub fn ensure_dir(dir: &Path) -> io::Result<()> {
    DirBuilder::new()
        .recursive(true)
        .mode(PRIVATE_DIR)
        .create(dir)
}

/// The directory's modification time, which changes whenever a state file is added, replaced or removed.
pub fn mtime(dir: &Path) -> Option<SystemTime> {
    fs::metadata(dir).and_then(|m| m.modified()).ok()
}

pub fn read(dir: &Path, session_id: &str) -> Option<Record> {
    read_path(&path(dir, session_id))
}

fn read_path(path: &Path) -> Option<Record> {
    fs::read(path)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
}

/// Every readable state file with its path; unreadable or foreign files are skipped.
pub fn read_all(dir: &Path) -> Vec<(PathBuf, Record)> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|e| e == "json"))
        .filter_map(|p| read_path(&p).map(|record| (p, record)))
        .collect()
}

/// Replaces the session's state file atomically.
pub fn write(dir: &Path, record: &Record) -> io::Result<()> {
    let target = path(dir, &record.session_id);
    let tmp = target.with_extension(format!("json.{}", std::process::id()));
    let json = serde_json::to_vec(record).map_err(io::Error::other)?;
    write_private(&tmp, &json)
        .and_then(|()| fs::rename(&tmp, &target))
        .inspect_err(|_| remove(&tmp))
}

fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(PRIVATE_FILE)
        .open(path)?
        .write_all(bytes)
}

/// Appends a line to a private log file.
pub fn append(path: &Path, line: &str) -> io::Result<()> {
    OpenOptions::new()
        .append(true)
        .create(true)
        .mode(PRIVATE_FILE)
        .open(path)?
        .write_all(format!("{line}\n").as_bytes())
}

/// Removes a file, ignoring one that is already gone.
pub fn remove(path: &Path) {
    let _ = fs::remove_file(path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn writes_private_files_and_reads_them_back() -> io::Result<()> {
        let dir = tempfile::tempdir()?;
        let state = dir.path().join("state");
        ensure_dir(&state)?;
        let record = Record {
            session_id: "s1".to_owned(),
            ..Record::default()
        };
        write(&state, &record)?;
        let mode = fs::metadata(path(&state, "s1"))?.permissions().mode() & 0o777;
        assert_eq!(mode, PRIVATE_FILE);
        assert_eq!(read(&state, "s1"), Some(record));
        assert_eq!(read_all(&state).len(), 1);
        Ok(())
    }
}
