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
    .map(|record| Claude {
        record,
        pane: None,
        latest: None,
    })
    .collect();
    let ids: Vec<String> = newest_per_pane(claudes)
        .into_iter()
        .map(|c| c.record.session_id)
        .sorted()
        .collect();
    assert_eq!(ids, ["new", "x"]);
}

fn claude(record: Record, latest: Option<Latest>) -> Claude {
    Claude {
        record,
        pane: None,
        latest,
    }
}

#[test]
fn shows_the_branch_and_context_a_hook_before_0_7_4_wrote() {
    let record: Record = serde_json::from_str(r#"{"session_id":"a","branch":"main","context":64}"#)
        .unwrap_or_default();
    let claude = claude(record, None);
    assert_eq!(claude.branch(), "main");
    assert_eq!(claude.context(), Some(64));
}

#[test]
fn prefers_the_transcript_to_the_old_branch_and_context() {
    let record = Record {
        transcript: "/t".to_owned(),
        branch: "old".to_owned(),
        context: Some(64),
        ..Record::default()
    };
    let latest = Latest {
        branch: "new".to_owned(),
        context: None,
    };
    let read = claude(record.clone(), Some(latest));
    assert_eq!(read.branch(), "new");
    assert_eq!(read.context(), None);

    let unread = claude(record, None);
    assert_eq!(unread.branch(), "");
    assert_eq!(unread.context(), None);
}
