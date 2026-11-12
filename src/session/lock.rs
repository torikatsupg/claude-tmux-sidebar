//! A session's lock, so that hooks running at the same time do not lose each other's updates of its state
//! file. A lock file next to it is created exclusively; a hook that cannot get the lock in time goes on
//! without it rather than hold Claude up.

use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

/// How long a hook waits for the lock.
const WAIT: Duration = Duration::from_millis(1500);
/// A lock this old was left by a hook that died; it is taken over.
const STALE: Duration = Duration::from_secs(10);
const RETRY: Duration = Duration::from_millis(5);

/// Held while the state file is read and written; removed when dropped.
#[derive(Debug)]
pub struct Lock(PathBuf);

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub fn path(dir: &Path, session_id: &str) -> PathBuf {
    dir.join(format!("{session_id}.lock"))
}

/// The session's lock, or `None` when another hook keeps it past [`WAIT`] or the file cannot be made.
pub fn acquire(dir: &Path, session_id: &str) -> Option<Lock> {
    let path = path(dir, session_id);
    let deadline = Instant::now() + WAIT;
    std::iter::repeat(())
        .take_while(|()| Instant::now() < deadline)
        .find_map(|()| try_lock(&path))
        .flatten()
}

/// `Some(Some(lock))` once taken, `Some(None)` to give up, `None` to try again.
#[expect(
    clippy::option_option,
    reason = "three outcomes: taken, give up, retry"
)]
fn try_lock(path: &Path) -> Option<Option<Lock>> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(_) => Some(Some(Lock(path.to_path_buf()))),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            if stale(path) {
                let _ = fs::remove_file(path);
            } else {
                std::thread::sleep(RETRY);
            }
            None
        }
        Err(_) => Some(None),
    }
}

fn stale(path: &Path) -> bool {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age > STALE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_hook_holds_the_lock_at_a_time_and_drops_it() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let first = acquire(dir.path(), "s");
        assert!(first.is_some());
        assert!(path(dir.path(), "s").exists());
        drop(first);
        assert!(!path(dir.path(), "s").exists());
        assert!(acquire(dir.path(), "s").is_some());
        Ok(())
    }
}
