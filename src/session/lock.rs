//! A session's lock, so that hooks running at the same time do not lose each other's updates of its state
//! file. A lock file next to it is created exclusively; a hook that cannot get the lock in time goes on
//! without it rather than hold Claude up, and a lock left by a hook that died is taken over.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

/// How long a hook waits for the lock.
const WAIT: Duration = Duration::from_millis(1500);
/// A lock this old was left by a hook that died; it is taken over.
const STALE: Duration = Duration::from_secs(10);
const RETRY: Duration = Duration::from_millis(5);

/// Held while the state file is read and written. The file holds this hook's token, so that dropping the
/// lock removes it only while it is still this hook's.
#[derive(Debug)]
pub struct Lock {
    path: PathBuf,
    token: String,
}

impl Drop for Lock {
    fn drop(&mut self) {
        if fs::read_to_string(&self.path).is_ok_and(|held| held == self.token) {
            let _ = fs::remove_file(&self.path);
        }
    }
}

pub fn path(dir: &Path, session_id: &str) -> PathBuf {
    dir.join(format!("{session_id}.lock"))
}

/// The session's lock, or `None` when another hook keeps it past [`WAIT`] or the file cannot be made.
pub fn acquire(dir: &Path, session_id: &str) -> Option<Lock> {
    let path = path(dir, session_id);
    let token = token();
    let deadline = Instant::now() + WAIT;
    std::iter::repeat(())
        .take_while(|()| Instant::now() < deadline)
        .find_map(|()| try_lock(&path, &token))
        .flatten()
}

/// This process and the moment it asks, unique among the hooks that may race for a lock.
fn token() -> String {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    format!("{} {nanos}", std::process::id())
}

/// `Some(Some(lock))` once taken, `Some(None)` to give up, `None` to try again.
#[expect(
    clippy::option_option,
    reason = "three outcomes: taken, give up, retry"
)]
fn try_lock(path: &Path, token: &str) -> Option<Option<Lock>> {
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(mut file) => {
            let written = file.write_all(token.as_bytes());
            Some(written.ok().map(|()| Lock {
                path: path.to_path_buf(),
                token: token.to_owned(),
            }))
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            if stale(path) {
                take_over(path, token);
            } else {
                std::thread::sleep(RETRY);
            }
            None
        }
        Err(_) => Some(None),
    }
}

/// Moves a stale lock aside; only one hook's rename succeeds. Should another hook have replaced it with a
/// fresh lock in the meantime, that lock is put back.
fn take_over(path: &Path, token: &str) {
    let aside = path.with_extension(format!("lock.{}", token.replace(' ', "-")));
    if fs::rename(path, &aside).is_err() {
        return;
    }
    if !stale(&aside) {
        let _ = fs::hard_link(&aside, path);
    }
    let _ = fs::remove_file(&aside);
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

    #[test]
    fn a_hook_leaves_a_lock_taken_over_from_it_alone() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let mine = acquire(dir.path(), "s");
        fs::write(path(dir.path(), "s"), "other 1")?;
        drop(mine);
        assert_eq!(fs::read_to_string(path(dir.path(), "s"))?, "other 1");
        Ok(())
    }

    #[test]
    fn a_stale_lock_is_taken_over() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let file = fs::File::create(path(dir.path(), "s"))?;
        file.set_modified(SystemTime::now() - STALE * 2)?;
        drop(file);
        let started = Instant::now();
        let taken = acquire(dir.path(), "s");
        assert!(taken.is_some() && started.elapsed() < WAIT);
        assert_ne!(fs::read_to_string(path(dir.path(), "s"))?, "");
        Ok(())
    }
}
