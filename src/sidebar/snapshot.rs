//! What the sidebar knows about tmux and Claude at one moment.

use std::collections::HashMap;

use super::rows::{self, Row};
use crate::context::Context;
use crate::listing;
use crate::session::live;
use crate::tmux::{
    self,
    pane::{self, Pane, PaneId, WindowId},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Snapshot {
    pub rows: Vec<Row>,
    /// The window the sidebar belongs to (or the popup was opened from).
    pub window: Option<WindowId>,
    /// The pane the user works in, in that window.
    pub focus: Option<Pane>,
    /// The sidebar pane itself has the focus.
    pub sidebar_active: bool,
    /// This sidebar should go: its window has no real pane left, or an older sidebar (concurrent hooks can add two).
    pub redundant: bool,
    /// Each tmux session's current window.
    pub session_windows: HashMap<String, WindowId>,
}

/// Reads tmux and the state files. `me` is the sidebar pane, or the pane a popup was opened from.
pub fn load(ctx: &Context, me: Option<PaneId>, popup: bool) -> Snapshot {
    let panes = tmux::panes();
    let claudes = live::load(&ctx.state_dir, &ctx.socket, &panes);
    let groups = listing::groups(&tmux::sessions(), &claudes);
    from_parts(rows::rows(&groups), &panes, me, popup)
}

/// The pure part of [`load`].
pub fn from_parts(rows: Vec<Row>, panes: &[Pane], me: Option<PaneId>, popup: bool) -> Snapshot {
    let mine = me.and_then(|id| panes.iter().find(|p| p.id == id));
    let window = mine.map(|p| p.window_id);
    Snapshot {
        rows,
        window,
        focus: window.and_then(|w| pane::focused(panes, w)).cloned(),
        sidebar_active: !popup && mine.is_some_and(|p| p.active),
        redundant: !popup && mine.is_some_and(|p| redundant(panes, p)),
        session_windows: panes
            .iter()
            .filter(|p| p.window_active)
            .map(|p| (p.session.clone(), p.window_id))
            .collect(),
    }
}

fn redundant(panes: &[Pane], me: &Pane) -> bool {
    let window: Vec<&Pane> = panes
        .iter()
        .filter(|p| p.window_id == me.window_id)
        .collect();
    let oldest_sidebar = window.iter().filter(|p| p.sidebar).map(|p| p.id).min();
    window.iter().all(|p| p.sidebar) || oldest_sidebar != Some(me.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmux::pane::fixture::pane;

    fn sidebar(id: u32) -> Pane {
        Pane {
            sidebar: true,
            ..pane(id, 0)
        }
    }

    #[test]
    fn a_lone_or_newer_sidebar_is_redundant() {
        let me = sidebar(5);
        assert!(from_parts(vec![], std::slice::from_ref(&me), Some(me.id), false).redundant);
        assert!(
            from_parts(
                vec![],
                &[sidebar(4), me.clone(), pane(1, 0)],
                Some(me.id),
                false
            )
            .redundant
        );
        assert!(!from_parts(vec![], &[me.clone(), pane(1, 0)], Some(me.id), false).redundant);
    }

    #[test]
    fn popups_never_count_as_active_or_redundant() {
        let origin = Pane {
            active: true,
            ..pane(1, 0)
        };
        let snapshot = from_parts(vec![], std::slice::from_ref(&origin), Some(origin.id), true);
        assert!(!snapshot.sidebar_active && !snapshot.redundant);
        assert_eq!(snapshot.focus.map(|p| p.id), Some(origin.id));
    }
}
