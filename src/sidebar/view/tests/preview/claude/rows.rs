//! What other rows change: a session sharing the directory, and the list row of a waiting session.

use super::*;

#[test]
fn changes_shared_with_another_session_say_so() {
    let model = claude_model(Status::Done, replied("ok"), 30);
    let other = Claude {
        record: Record {
            session_id: "b".to_owned(),
            ..record(Status::Running, "")
        },
        pane: Some(pane(2, 3)),
        latest: None,
    };
    let mut rows = model.snapshot.rows.clone();
    rows.push(Row::Session(other));
    let model = Model {
        snapshot: Snapshot {
            rows,
            ..model.snapshot.clone()
        },
        ..model
    };
    assert!(panel(&model, 8).contains(&"± +128 -14 (3 files, shared)".to_owned()));
}

#[test]
fn a_waiting_row_shows_what_it_asks_for_in_place_of_the_prompt() {
    let model = claude_model(Status::Waiting, asking("cargo test --all-targets"), 30);
    assert_eq!(
        screen(&model, &EN).get(4).map(String::as_str),
        Some("└ Bash: cargo test --all-tar… 42% ")
    );
}
