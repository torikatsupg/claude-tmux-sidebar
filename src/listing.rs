//! Claude sessions grouped by tmux session, in tmux's own session / window / pane order.

use itertools::Itertools;

use crate::session::live::Claude;
use crate::tmux::pane::{self, Pane};

/// A group heading: a tmux session, or the sessions running outside tmux.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GroupName {
    Session(String),
    Outside,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub name: GroupName,
    pub claudes: Vec<Claude>,
}

fn group_name(claude: &Claude) -> GroupName {
    claude.pane.as_ref().map_or(GroupName::Outside, |p| {
        GroupName::Session(p.session.clone())
    })
}

/// Where a Claude sits in its group: (window, pane).
pub fn position(claude: &Claude) -> (u32, u32) {
    claude
        .pane
        .as_ref()
        .map_or((0, 0), |p| (p.window_index, p.pane_index))
}

/// Every tmux session (with or without Claude) in tmux's order, then the sessions outside tmux.
pub fn groups(sessions: &[String], claudes: &[Claude]) -> Vec<Group> {
    let names = claudes.iter().map(group_name);
    sessions
        .iter()
        .cloned()
        .map(GroupName::Session)
        .chain(names.clone().filter(|n| *n != GroupName::Outside))
        .chain(names.filter(|n| *n == GroupName::Outside))
        .unique()
        .map(|name| Group {
            claudes: claudes
                .iter()
                .filter(|c| group_name(c) == name)
                .cloned()
                .sorted_by_key(position)
                .collect(),
            name,
        })
        .collect()
}

/// The windows running no Claude, each as the pane the user works in there. A window linked into several
/// sessions is listed once, under the last of them in tmux's order, as a Claude in such a window is.
pub fn plain_windows(panes: &[Pane], claudes: &[Claude]) -> Vec<Pane> {
    let busy: Vec<_> = claudes
        .iter()
        .filter_map(|c| c.pane.as_ref())
        .map(|p| p.window_id)
        .collect();
    // `list-panes -a` repeats a linked window's panes for each session; reversed, the last copy wins.
    let reversed: Vec<Pane> = panes.iter().rev().cloned().collect();
    panes
        .iter()
        .map(|p| p.window_id)
        .unique()
        .filter(|w| !busy.contains(w))
        .filter_map(|w| pane::focused(&reversed, w))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests;
