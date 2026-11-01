//! What the sidebar knows about tmux and Claude at one moment.

use std::collections::HashMap;
use std::path::Path;

use super::rows::{self, Row};
use crate::context::Context;
use crate::listing;
use crate::session::context_window::{self, ContextWindows};
use crate::session::live::{self, Claude};
use crate::session::transcript::cache::Transcripts;
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
    /// The sizes set in `@cts_context_windows`.
    pub context_windows: ContextWindows,
}

/// Whose list a snapshot is for, and what it lists.
pub struct Scope {
    /// The sidebar pane, or the pane a popup was opened from.
    pub me: Option<PaneId>,
    pub popup: bool,
    /// List the windows without Claude too (`@cts_all_windows`).
    pub all_windows: bool,
}

/// Reads tmux, the state files and the transcripts that changed since `transcripts`.
pub fn load(ctx: &Context, scope: &Scope, transcripts: Transcripts) -> (Snapshot, Transcripts) {
    let panes = tmux::panes();
    let claudes = live::load(&ctx.state_dir, &ctx.socket, &panes);
    let (claudes, transcripts) = with_transcripts(claudes, transcripts);
    let groups = listing::groups(&tmux::sessions(), &claudes);
    let windows = if scope.all_windows {
        listing::plain_windows(&panes, &claudes)
    } else {
        Vec::new()
    };
    let context_windows = tmux::global_option(context_window::OPTION)
        .map(|option| ContextWindows::parse(&option))
        .unwrap_or_default();
    let snapshot = Snapshot {
        context_windows,
        ..from_parts(
            rows::with_windows(&groups, &windows),
            &panes,
            scope.me,
            scope.popup,
        )
    };
    (snapshot, transcripts)
}

/// `claudes` with what their transcripts say, reading only the transcripts that changed.
fn with_transcripts(claudes: Vec<Claude>, transcripts: Transcripts) -> (Vec<Claude>, Transcripts) {
    let transcripts = transcripts.refresh(
        claudes
            .iter()
            .map(|c| Path::new(c.record.transcript.as_str()))
            .filter(|p| !p.as_os_str().is_empty()),
    );
    let claudes = claudes
        .into_iter()
        .map(|c| Claude {
            latest: transcripts.latest(&c.record.transcript),
            ..c
        })
        .collect();
    (claudes, transcripts)
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
        context_windows: ContextWindows::default(),
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
