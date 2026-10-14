//! Which state files describe live sessions of this tmux server.

use std::collections::BTreeMap;
use std::path::Path;

use itertools::Itertools;

use super::{Record, store};
use crate::process;
use crate::tmux::pane::{Pane, PaneId};

/// A live Claude session and, when it runs in this server, its pane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claude {
    pub record: Record,
    pub pane: Option<Pane>,
}

/// What to do with one state file.
#[derive(Debug, PartialEq, Eq)]
pub enum Disposition {
    Keep(Box<Claude>),
    /// Its process or pane is gone.
    Remove,
    /// It belongs to another tmux server.
    Skip,
}

/// The tmux server the sidebar runs in.
pub struct Server<'a> {
    pub socket: &'a str,
    pub panes: &'a BTreeMap<PaneId, Pane>,
}

/// Decides about one state file, given whether its process is alive.
pub fn dispose(record: Record, alive: bool, server: &Server<'_>) -> Disposition {
    let same_server = record.socket == server.socket;
    let pane = record
        .pane_id()
        .and_then(|id| server.panes.get(&id))
        .cloned();
    let pane_gone = same_server && record.pane_id().is_some() && pane.is_none();
    if !alive || pane_gone {
        Disposition::Remove
    } else if !record.socket.is_empty() && !same_server {
        Disposition::Skip
    } else {
        Disposition::Keep(Box::new(Claude {
            pane: pane.filter(|_| same_server),
            record,
        }))
    }
}

/// Keeps the most recently updated session per pane (a pane can only run one Claude at a time).
pub fn newest_per_pane(claudes: Vec<Claude>) -> Vec<Claude> {
    let key = |c: &Claude| {
        c.record
            .pane_id()
            .map_or_else(|| c.record.session_id.clone(), |p| p.to_string())
    };
    claudes
        .into_iter()
        .into_group_map_by(key)
        .into_values()
        .filter_map(|group| group.into_iter().max_by_key(|c| c.record.updated_at))
        .collect()
}

/// Loads the live sessions of this server, deleting state files whose session is gone.
pub fn load(dir: &Path, socket: &str, panes: &[Pane]) -> Vec<Claude> {
    let panes: BTreeMap<PaneId, Pane> = panes.iter().map(|p| (p.id, p.clone())).collect();
    let server = Server {
        socket,
        panes: &panes,
    };
    let kept = store::read_all(dir)
        .into_iter()
        .filter_map(|(path, record)| keep_or_remove(&path, record, &server))
        .collect();
    newest_per_pane(kept)
}

fn keep_or_remove(path: &Path, record: Record, server: &Server<'_>) -> Option<Claude> {
    let alive = record.pid.is_none_or(process::alive);
    match dispose(record, alive, server) {
        Disposition::Keep(claude) => Some(*claude),
        Disposition::Remove => {
            store::remove(path);
            None
        }
        Disposition::Skip => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmux::pane::fixture::pane;

    fn record(id: &str, pane: &str, socket: &str, updated_at: u64) -> Record {
        Record {
            session_id: id.to_owned(),
            pane: pane.to_owned(),
            socket: socket.to_owned(),
            updated_at,
            ..Record::default()
        }
    }

    fn dispose_in(record: Record, alive: bool) -> Disposition {
        let panes: BTreeMap<PaneId, Pane> = [pane(1, 0)].into_iter().map(|p| (p.id, p)).collect();
        dispose(
            record,
            alive,
            &Server {
                socket: "s",
                panes: &panes,
            },
        )
    }

    fn kept(disposition: Disposition) -> Option<Claude> {
        match disposition {
            Disposition::Keep(claude) => Some(*claude),
            Disposition::Remove | Disposition::Skip => None,
        }
    }

    #[test]
    fn keeps_sessions_of_this_server_with_their_pane() {
        assert_eq!(
            kept(dispose_in(record("a", "%1", "s", 0), true)).and_then(|c| c.pane),
            Some(pane(1, 0))
        );
    }

    #[test]
    fn removes_dead_or_paneless_sessions() {
        assert_eq!(
            dispose_in(record("a", "%1", "s", 0), false),
            Disposition::Remove
        );
        assert_eq!(
            dispose_in(record("a", "%9", "s", 0), true),
            Disposition::Remove
        );
    }

    #[test]
    fn skips_other_servers_but_keeps_sessions_outside_tmux() {
        assert_eq!(
            dispose_in(record("a", "%9", "other", 0), true),
            Disposition::Skip
        );
        assert_eq!(
            kept(dispose_in(record("a", "", "", 0), true)).map(|c| c.pane),
            Some(None)
        );
    }

    #[test]
    fn keeps_the_newest_session_per_pane() {
        let claudes = [
            record("old", "%1", "s", 1),
            record("new", "%1", "s", 2),
            record("x", "", "", 0),
        ]
        .into_iter()
        .map(|record| Claude { record, pane: None })
        .collect();
        let ids: Vec<String> = newest_per_pane(claudes)
            .into_iter()
            .map(|c| c.record.session_id)
            .sorted()
            .collect();
        assert_eq!(ids, ["new", "x"]);
    }
}
