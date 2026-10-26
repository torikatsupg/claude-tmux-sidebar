//! Claude sessions grouped by tmux session, in tmux's own session / window / pane order.

use itertools::Itertools;

use crate::session::live::Claude;

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

fn position(claude: &Claude) -> (u32, u32) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Record;
    use crate::tmux::pane::{Pane, fixture::pane};

    fn claude(id: &str, pane: Option<Pane>) -> Claude {
        Claude {
            record: Record {
                session_id: id.to_owned(),
                ..Record::default()
            },
            pane,
            latest: None,
        }
    }

    fn in_session(session: &str, window: u32) -> Pane {
        Pane {
            session: session.to_owned(),
            ..pane(window, window)
        }
    }

    #[test]
    fn follows_tmux_order_and_puts_outside_last() {
        let claudes = [
            claude("out", None),
            claude("b2", Some(in_session("b", 2))),
            claude("b1", Some(in_session("b", 1))),
        ];
        let groups = groups(&["a".to_owned(), "b".to_owned()], &claudes);
        let names: Vec<&GroupName> = groups.iter().map(|g| &g.name).collect();
        assert_eq!(
            names,
            [
                &GroupName::Session("a".into()),
                &GroupName::Session("b".into()),
                &GroupName::Outside
            ]
        );
        let b: Vec<&str> = groups
            .get(1)
            .into_iter()
            .flat_map(|g| &g.claudes)
            .map(|c| c.record.session_id.as_str())
            .collect();
        assert_eq!(b, ["b1", "b2"]);
    }
}
