//! The working tree's change summary for the preview, read with `git` at most every [`INTERVAL`] seconds.

/// Seconds between two reads of the same directory.
pub const INTERVAL: u64 = 10;

/// `git diff --shortstat HEAD`: tracked files changed since the last commit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Changes {
    pub files: u32,
    pub added: u32,
    pub removed: u32,
}

/// What `git` last said about a directory, and when it was asked.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Git {
    pub dir: String,
    pub asked: u64,
    /// Empty outside a repository.
    pub branch: String,
    /// `None` outside a repository, without commits, or with nothing changed.
    pub changes: Option<Changes>,
}

/// A read of `dir`, and of the repositories that `others` (other sessions' directories) belong to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitAsk {
    pub dir: String,
    pub others: Vec<String>,
}

/// The answer to one read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitRead {
    pub dir: String,
    pub branch: String,
    pub changes: Option<Changes>,
    /// Each directory asked about and its repository's top directory; empty outside a repository.
    pub tops: Vec<(String, String)>,
}

/// ` 3 files changed, 128 insertions(+), 14 deletions(-)`; `None` when nothing changed or for anything else.
pub fn shortstat(output: &str) -> Option<Changes> {
    let count = |unit: &str| {
        output
            .split(',')
            .find(|part| part.contains(unit))
            .and_then(|part| part.split_whitespace().next())
            .and_then(|n| n.parse().ok())
    };
    let changes = Changes {
        files: count("changed")?,
        added: count("insertion").unwrap_or(0),
        removed: count("deletion").unwrap_or(0),
    };
    Some(changes).filter(|c| c.files > 0)
}

/// Whether `dir` should be read again at `now`: it is new, or the last read is [`INTERVAL`] old.
pub fn due(git: Option<&Git>, dir: &str, now: u64) -> bool {
    git.is_none_or(|g| g.dir != dir || now >= g.asked.saturating_add(INTERVAL))
}

/// `git` after asking about `dir` at `now`: what is known about the same directory stays until the answer.
pub fn asked(git: Option<Git>, dir: &str, now: u64) -> Git {
    let known = git.filter(|g| g.dir == dir).unwrap_or_default();
    Git {
        dir: dir.to_owned(),
        asked: now,
        ..known
    }
}

/// `git` with an answer, which counts only for the directory last asked about.
pub fn answered(git: Option<Git>, read: GitRead) -> Option<Git> {
    git.map(|g| {
        if g.dir == read.dir {
            Git {
                branch: read.branch,
                changes: read.changes,
                ..g
            }
        } else {
            g
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_shortstat_line() {
        let changes = |files, added, removed| {
            Some(Changes {
                files,
                added,
                removed,
            })
        };
        assert_eq!(
            shortstat(" 3 files changed, 128 insertions(+), 14 deletions(-)\n"),
            changes(3, 128, 14)
        );
        assert_eq!(
            shortstat(" 1 file changed, 2 insertions(+)"),
            changes(1, 2, 0)
        );
        assert_eq!(
            shortstat(" 1 file changed, 5 deletions(-)"),
            changes(1, 0, 5)
        );
        assert_eq!(shortstat(""), None);
        assert_eq!(shortstat("fatal: not a git repository"), None);
    }

    #[test]
    fn a_directory_is_read_again_only_after_the_interval_or_when_it_changes() {
        let git = asked(None, "/a", 100);
        assert!(due(None, "/a", 100));
        assert!(!due(Some(&git), "/a", 109));
        assert!(due(Some(&git), "/a", 110));
        assert!(due(Some(&git), "/b", 101));
    }

    #[test]
    fn an_answer_counts_only_for_the_directory_asked_about() {
        let read = |dir: &str| GitRead {
            dir: dir.to_owned(),
            branch: "main".to_owned(),
            changes: None,
            tops: vec![],
        };
        let git = answered(Some(asked(None, "/a", 1)), read("/a"));
        assert_eq!(git.as_ref().map(|g| g.branch.as_str()), Some("main"));
        let stale = answered(Some(asked(git, "/b", 2)), read("/a"));
        assert_eq!(stale.map(|g| g.branch), Some(String::new()));
        let again = asked(Some(asked(None, "/a", 1)), "/a", 20);
        assert_eq!(again.asked, 20);
    }
}
