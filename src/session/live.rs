//! Which state files describe live sessions of this tmux server.

use std::collections::BTreeMap;
use std::path::Path;

use itertools::Itertools;

use super::transcript::Latest;
use super::{Record, store};
use crate::process;
use crate::tmux::pane::{Pane, PaneId};

/// A live Claude session and, when it runs in this server, its pane.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claude {
    pub record: Record,
    pub pane: Option<Pane>,
    /// What its transcript says, when the sidebar has read it.
    pub latest: Option<Latest>,
}

impl Claude {
    /// The git branch Claude last worked on; empty when unknown.
    pub fn branch(&self) -> &str {
        self.latest.as_ref().map_or("", |l| l.branch.as_str())
    }

    /// How full the context window is, in percent.
    pub fn context(&self) -> Option<u8> {
        self.latest.as_ref().and_then(|l| l.context)
    }
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
            latest: None,
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
mod tests;
