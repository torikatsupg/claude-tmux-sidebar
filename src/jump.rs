//! `next` / `prev`: go to the next or previous done or waiting Claude after the current pane, wrapping around.

use crate::context::Context;
use crate::i18n;
use crate::listing::{self, Group, GroupName};
use crate::session::live;
use crate::tmux::{
    self,
    pane::{self, Pane, PaneId},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Next,
    Prev,
}

/// Where a pane sits in the listing: (group, window, pane).
type Position = (usize, u32, u32);

fn position(groups: &[Group], pane: &Pane) -> Position {
    let session = GroupName::Session(pane.session.clone());
    let group = groups
        .iter()
        .position(|g| g.name == session)
        .unwrap_or(groups.len());
    (group, pane.window_index, pane.pane_index)
}

/// The pane to go to from `here`, or `None` when no Claude needs the user.
pub fn target(direction: Direction, groups: &[Group], here: Option<&Pane>) -> Option<PaneId> {
    let targets: Vec<(Position, PaneId)> = groups
        .iter()
        .flat_map(|g| &g.claudes)
        .filter(|c| c.record.state.needs_attention())
        .filter_map(|c| c.pane.as_ref())
        .map(|p| (position(groups, p), p.id))
        .collect();
    let Some(here) = here.map(|p| position(groups, p)) else {
        return targets.first().map(|&(_, id)| id);
    };
    match direction {
        Direction::Next => targets
            .iter()
            .find(|(pos, _)| *pos > here)
            .or_else(|| targets.first()),
        Direction::Prev => targets
            .iter()
            .rev()
            .find(|(pos, _)| *pos < here)
            .or_else(|| targets.last()),
    }
    .map(|&(_, id)| id)
}

/// Runs the jump from `current` (a sidebar counts as the pane it sits next to) on `client`.
pub fn run(ctx: &Context, direction: Direction, current: Option<PaneId>, client: Option<&str>) {
    let panes = tmux::panes();
    let claudes = live::load(&ctx.state_dir, &ctx.socket, &panes);
    let groups = listing::groups(&tmux::sessions(), &claudes);
    let here = current
        .and_then(|id| panes.iter().find(|p| p.id == id))
        .and_then(|p| {
            if p.sidebar {
                pane::focused(&panes, p.window_id)
            } else {
                Some(p)
            }
        });
    if let Some(id) = target(direction, &groups, here) {
        tmux::goto(id, client);
    } else {
        let lang = tmux::global_option("@cts_lang");
        tmux::message(client, i18n::messages(lang.as_deref()).nothing);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::live::Claude;
    use crate::session::{Record, Status};
    use crate::tmux::pane::fixture::pane;

    fn group(states: &[(u32, Status)]) -> Vec<Group> {
        let claudes = states
            .iter()
            .map(|&(id, state)| Claude {
                record: Record {
                    state,
                    ..Record::default()
                },
                pane: Some(pane(id, id)),
                latest: None,
            })
            .collect();
        vec![Group {
            name: GroupName::Session("s".to_owned()),
            claudes,
        }]
    }

    #[test]
    fn visits_only_sessions_that_need_the_user_and_wraps() {
        let groups = group(&[
            (1, Status::Done),
            (2, Status::Running),
            (3, Status::Waiting),
        ]);
        let id = |n| pane(n, n).id;
        assert_eq!(
            target(Direction::Next, &groups, Some(&pane(1, 1))),
            Some(id(3))
        );
        assert_eq!(
            target(Direction::Next, &groups, Some(&pane(3, 3))),
            Some(id(1))
        );
        assert_eq!(
            target(Direction::Prev, &groups, Some(&pane(1, 1))),
            Some(id(3))
        );
        assert_eq!(target(Direction::Prev, &groups, None), Some(id(1)));
    }

    #[test]
    fn nothing_to_visit() {
        assert_eq!(
            target(Direction::Next, &group(&[(1, Status::Running)]), None),
            None
        );
    }
}
