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

fn ids(panes: &[Pane]) -> Vec<(String, u32)> {
    panes
        .iter()
        .map(|p| (p.session.clone(), p.window_index))
        .collect()
}

#[test]
fn plain_windows_skip_windows_with_claude_in_any_pane() {
    let sidebar = Pane {
        sidebar: true,
        active: true,
        ..pane(9, 1)
    };
    let panes = [pane(1, 1), pane(2, 1), sidebar, pane(3, 2), pane(4, 3)];
    let claudes = [claude("c", Some(pane(2, 1)))];
    let windows = plain_windows(&panes, &claudes);
    assert_eq!(ids(&windows), [("s".to_owned(), 2), ("s".to_owned(), 3)]);
    assert!(windows.iter().all(|p| !p.sidebar));
}

#[test]
fn a_linked_window_is_listed_once_under_its_last_session() {
    let panes = [in_session("a", 1), in_session("a", 2), in_session("b", 2)];
    assert_eq!(
        ids(&plain_windows(&panes, &[])),
        [("a".to_owned(), 1), ("b".to_owned(), 2)]
    );
}
