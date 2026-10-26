use super::*;
use crate::session::Status;
use crate::session::transcript::Latest;

fn change(status: Status) -> Change {
    Change {
        status,
        detail: String::new(),
        background: false,
        agent: String::new(),
        answered: false,
    }
}

fn origin(now: u64) -> Origin {
    Origin {
        pane: "%1".to_owned(),
        socket: "s".to_owned(),
        pid: Some(9),
        now,
        transcript: None,
    }
}

#[test]
fn since_moves_only_on_a_status_change() {
    let payload = Payload {
        session_id: "a".to_owned(),
        cwd: "/x".to_owned(),
        ..Payload::default()
    };
    let first = compose(&payload, None, change(Status::Running), &origin(10));
    let same = compose(&payload, Some(&first), change(Status::Running), &origin(20));
    let changed = compose(&payload, Some(&same), change(Status::Done), &origin(30));
    assert_eq!(
        (first.since, same.since, changed.since),
        (Some(10), Some(10), Some(30))
    );
    assert_eq!(same.updated_at, 20);
}

#[test]
fn keeps_the_previous_cwd_when_the_payload_has_none() {
    let prev = Record {
        cwd: "/x".to_owned(),
        ..Record::default()
    };
    let record = compose(
        &Payload::default(),
        Some(&prev),
        change(Status::Done),
        &origin(0),
    );
    assert_eq!(record.cwd, "/x");
}

#[test]
fn a_subagent_does_not_move_the_cwd() {
    let prev = Record {
        cwd: "/x".to_owned(),
        ..Record::default()
    };
    let payload = Payload {
        cwd: "/x/worktree".to_owned(),
        agent_id: "a1".to_owned(),
        ..Payload::default()
    };
    let record = compose(&payload, Some(&prev), change(Status::Running), &origin(0));
    assert_eq!(record.cwd, "/x");
}

#[test]
fn keeps_the_prompt_branch_and_context_until_new_ones_arrive() {
    let submit = Payload {
        session_id: "a".to_owned(),
        prompt: "fix\n  the   bug".to_owned(),
        ..Payload::default()
    };
    let seen = Origin {
        transcript: Some(Latest {
            branch: "main".to_owned(),
            context: Some(42),
        }),
        ..origin(0)
    };
    let first = compose(&submit, None, change(Status::Running), &seen);
    assert_eq!(
        (first.prompt.as_str(), first.branch.as_str(), first.context),
        ("fix the bug", "main", Some(42))
    );
    let later = compose(
        &Payload::default(),
        Some(&first),
        change(Status::Done),
        &origin(1),
    );
    assert_eq!(
        (later.prompt.as_str(), later.branch.as_str(), later.context),
        ("fix the bug", "main", Some(42))
    );
}

#[test]
fn a_transcript_without_usage_clears_the_previous_context() {
    let prev = Record {
        context: Some(91),
        ..Record::default()
    };
    let compacted = Origin {
        transcript: Some(Latest {
            branch: "main".to_owned(),
            context: None,
        }),
        ..origin(0)
    };
    let record = compose(
        &Payload::default(),
        Some(&prev),
        change(Status::Done),
        &compacted,
    );
    assert_eq!(record.context, None);
    let unread = compose(
        &Payload::default(),
        Some(&prev),
        change(Status::Done),
        &origin(1),
    );
    assert_eq!(unread.context, Some(91));
}
