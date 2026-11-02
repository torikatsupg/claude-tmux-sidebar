//! Reading and writing state files. The directory and files are private to the user.

use std::fs::{self, DirBuilder, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::Record;

const PRIVATE_DIR: u32 = 0o700;
const PRIVATE_FILE: u32 = 0o600;
const WINDOW_EXTENSION: &str = "window";

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

/// The state file's sibling that holds the session's context window size.
fn window_path(state_file: &Path) -> PathBuf {
    state_file.with_extension(WINDOW_EXTENSION)
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
        .map(|(path, record)| {
            let context_window = read_window_path(&window_path(&path));
            let record = Record {
                context_window,
                ..record
            };
            (path, record)
        })
        .collect()
}

pub fn read_window(dir: &Path, session_id: &str) -> Option<u64> {
    read_window_path(&window_path(&path(dir, session_id)))
}

fn read_window_path(path: &Path) -> Option<u64> {
    let size = fs::read_to_string(path).ok()?.trim().parse().ok()?;
    (size > 0).then_some(size)
}

/// Records the session's context window size. It has its own file because the status line and the hooks
/// run concurrently: rewriting the state file here could undo a status a hook just wrote.
pub fn write_window(dir: &Path, session_id: &str, size: u64) -> io::Result<()> {
    replace(
        &window_path(&path(dir, session_id)),
        size.to_string().as_bytes(),
    )
}

/// Replaces the session's state file atomically.
pub fn write(dir: &Path, record: &Record) -> io::Result<()> {
    let json = serde_json::to_vec(record).map_err(io::Error::other)?;
    replace(&path(dir, &record.session_id), &json)
}

fn replace(target: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = target.with_extension(format!("tmp.{}", std::process::id()));
    write_private(&tmp, bytes)
        .and_then(|()| fs::rename(&tmp, target))
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

/// Removes the size files left without a state file: a session can end while the status line writes,
/// and hooks before 0.9 remove only the state file.
pub fn sweep_windows(dir: &Path) {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|p| p.extension().is_some_and(|e| e == WINDOW_EXTENSION))
        .filter(|p| !p.with_extension("json").exists())
        .for_each(|p| remove(&p));
}

/// Removes a state file and what goes with it.
pub fn remove_session(state_file: &Path) {
    remove(state_file);
    remove(&window_path(state_file));
}

/// Removes a file, ignoring one that is already gone.
pub fn remove(path: &Path) {
    let _ = fs::remove_file(path);
}

#[cfg(test)]
mod tests;
