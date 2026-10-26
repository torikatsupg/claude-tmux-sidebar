//! The transcripts the sidebar has read, so a file is only read again once it changed.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use super::{Latest, read};

/// A file's size and modification time; a transcript is re-read when either changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamp {
    pub len: u64,
    pub modified: Option<SystemTime>,
}

/// What each transcript said when it was last read, by path.
#[derive(Debug, Default)]
pub struct Transcripts(HashMap<PathBuf, (Stamp, Option<Latest>)>);

impl Transcripts {
    /// Brings `paths` up to date from disk and forgets every other path.
    #[must_use]
    pub fn refresh<'a>(self, paths: impl IntoIterator<Item = &'a Path>) -> Self {
        let stamped = paths.into_iter().map(|p| (p.to_path_buf(), stamp(p)));
        self.renew(stamped, read)
    }

    /// The pure part of [`refresh`](Self::refresh): `read` runs only for files that are new or changed.
    /// A missing file is left out.
    #[must_use]
    pub fn renew(
        mut self,
        stamped: impl IntoIterator<Item = (PathBuf, Option<Stamp>)>,
        mut read: impl FnMut(&Path) -> Option<Latest>,
    ) -> Self {
        let entries = stamped
            .into_iter()
            .filter_map(|(path, stamp)| {
                let stamp = stamp?;
                let entry = match self.0.remove(&path) {
                    Some((seen, latest)) if seen == stamp => (seen, latest),
                    _ => (stamp, read(&path)),
                };
                Some((path, entry))
            })
            .collect();
        Self(entries)
    }

    /// What the transcript at `path` said when last read; `None` when it is unknown or unreadable.
    pub fn latest(&self, path: &str) -> Option<Latest> {
        self.0
            .get(Path::new(path))
            .and_then(|(_, latest)| latest.clone())
    }
}

fn stamp(path: &Path) -> Option<Stamp> {
    let meta = fs::metadata(path).ok()?;
    Some(Stamp {
        len: meta.len(),
        modified: meta.modified().ok(),
    })
}

#[cfg(test)]
mod tests;
