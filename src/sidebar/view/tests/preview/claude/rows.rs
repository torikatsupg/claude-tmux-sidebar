//! What other rows change: a session sharing the directory, and the list row of a waiting session.

use super::*;

/// [`claude_model`] with another session working in `cwd`.
fn beside(cwd: &str) -> Model {
    let model = claude_model(Status::Done, replied("ok"), 30);
    let other = Claude {
        record: Record {
            session_id: "b".to_owned(),
            cwd: cwd.to_owned(),
            ..record(Status::Running, "")
        },
        pane: Some(pane(2, 3)),
        latest: None,
    };
    let rows = model
        .snapshot
        .rows
        .iter()
        .cloned()
        .chain([Row::Session(other)])
        .collect();
    Model {
        snapshot: Snapshot {
            rows,
            ..model.snapshot.clone()
        },
        ..model
    }
}

#[test]
fn changes_shared_within_the_same_tree_say_so() {
    let shared = "± +128 -14 (3 files, shared)".to_owned();
    assert!(panel(&beside("/src/api/web"), 8).contains(&shared));
    assert!(panel(&beside("/src"), 8).contains(&shared));
    assert!(!panel(&beside("/src/apix"), 8).contains(&shared));
}

#[test]
fn a_finished_session_leaves_out_monitors() {
    let model = with_record(claude_model(Status::Done, replied("ok"), 30), |record| {
        Record {
            tasks: vec![task("monitor", "", "watch CI", 0)],
            ..record
        }
    });
    assert_eq!(panel(&model, 3), ["ok", "± +128 -14 (3 files)", ""]);
}

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

/// A waiting session whose hook reported `detail` (and `agent`), with `pending` calls in its transcript.
fn waiting_on(detail: &str, agent: &str, pending: Vec<Ask>) -> Model {
    let said = Said {
        reply: String::new(),
        pending,
    };
    with_record(claude_model(Status::Waiting, said, 30), |record| Record {
        detail: detail.to_owned(),
        agent: agent.to_owned(),
        ..record
    })
}

#[test]
fn a_subagents_prompt_shows_only_its_tool() {
    let model = waiting_on(
        "Bash",
        "a1",
        vec![call("Agent", "Survey"), call("Bash", "ls")],
    );
    assert_eq!(panel(&model, 1), ["◆ Bash"]);
}

#[test]
fn the_ask_is_the_first_pending_call_of_the_waiting_tool() {
    let pending = vec![
        call("Read", "/a"),
        call("Bash", "make"),
        call("Bash", "make test"),
    ];
    assert_eq!(panel(&waiting_on("Bash", "", pending), 1), ["◆ Bash: make"]);
    let question = vec![call("Bash", "ls"), call("AskUserQuestion", "Which DB?")];
    assert_eq!(
        panel(&waiting_on("Question", "", question), 1),
        ["◆ Which DB?"]
    );
    let unknown = vec![call("Read", "/a")];
    assert_eq!(panel(&waiting_on("Bash", "", unknown), 1), ["◆ Bash"]);
}
