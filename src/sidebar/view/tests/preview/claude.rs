//! The status panel of a Claude row, by status.

use super::*;
use crate::session::Task;
use crate::session::transcript::{Ask, Said};

fn task(kind: &str, agent: &str, text: &str, since: u64) -> Task {
    Task {
        id: format!("{kind}{since}"),
        kind: kind.to_owned(),
        agent: agent.to_owned(),
        text: text.to_owned(),
        since,
    }
}

/// A Claude session in `/src/api` at time 1000, with two subagents and a background shell.
fn claude(state: Status, said: Said) -> Claude {
    Claude {
        record: Record {
            tasks: vec![
                task("subagent", "Explore", "docsを調査", 940),
                task("subagent", "review", "PR #118", 760),
                task("shell", "", "cargo build", 970),
            ],
            since: Some(880),
            ..record(state, "Bash")
        },
        pane: Some(pane(1, 2)),
        latest: Some(Latest {
            said,
            usage: Some(Usage {
                tokens: 84_000,
                model: "claude-opus-4-6".to_owned(),
            }),
            ..Latest::default()
        }),
    }
}

fn claude_model(state: Status, said: Said, height: u16) -> Model {
    let model = model_of(claude(state, said), true);
    Model {
        snapshot: Snapshot {
            focused: true,
            ..model.snapshot.clone()
        },
        preview: Preview {
            mode: Mode::Pane,
            ..Preview::default()
        },
        git: Some(Git {
            changes: Some(Changes {
                files: 3,
                added: 128,
                removed: 14,
            }),
            ..git("/src/api")
        }),
        now: 1000,
        height,
        ..model
    }
}

fn asking(command: &str) -> Said {
    Said {
        reply: "All done.".to_owned(),
        ask: Some(Ask {
            tool: "Bash".to_owned(),
            detail: command.to_owned(),
        }),
    }
}

fn replied(text: &str) -> Said {
    Said {
        reply: text.to_owned(),
        ask: None,
    }
}

fn panel(model: &Model, lines: usize) -> Vec<String> {
    screen(model, &EN)
        .into_iter()
        .skip(usize::from(model.list_end(model.height)) + 1)
        .take(lines)
        .collect()
}

#[test]
fn a_waiting_session_shows_what_it_asks_for_then_its_tasks_and_changes() {
    let model = claude_model(Status::Waiting, asking("cargo test --all"), 30);
    assert_eq!(
        panel(&model, 8),
        [
            "◆ Bash: cargo test --all",
            "⠋ agents 2",
            "  Explore  docsを調査          1m ",
            "  review   PR #118             4m ",
            "⠋ bg 1",
            "  cargo build                 30s ",
            "± +128 -14 (3 files)",
            "",
        ]
    );
}

#[test]
fn a_long_ask_wraps_and_goes_before_everything_else() {
    let command = "cargo test --workspace --all-features -- --nocapture sidebar::preview";
    let model = claude_model(Status::Waiting, asking(command), 18);
    assert_eq!(
        panel(&model, 4),
        [
            "◆ Bash: cargo test --workspace",
            "  --all-features -- --nocapture",
            "  sidebar::preview",
            "⠋ agents 2",
        ]
    );
}

#[test]
fn a_running_session_shows_its_tool_and_tasks_without_changes() {
    let model = claude_model(Status::Running, asking("cargo build"), 30);
    let lines = panel(&model, 8);
    assert_eq!(
        lines.first().map(String::as_str),
        Some("⠋ Bash: cargo build")
    );
    assert_eq!(
        lines.get(5).map(String::as_str),
        Some("  cargo build                 30s ")
    );
    assert!(lines.iter().all(|l| !l.contains('±')));
    let bare = claude_model(Status::Running, Said::default(), 30);
    assert_eq!(panel(&bare, 1), ["⠋ Bash"]);
}

#[test]
fn a_short_running_preview_keeps_the_counts() {
    let model = claude_model(Status::Running, asking("cargo build"), 15);
    assert_eq!(
        panel(&model, 3),
        ["⠋ Bash: cargo build", "⠋ agents 2", "⠋ bg 1"]
    );
}

#[test]
fn a_finished_session_shows_the_end_of_its_reply_then_tasks_and_changes() {
    let reply = "## Summary\n\nI added retry with backoff to the HTTP client.\n```go\nfor {}\n```\n全部のテストが通りました。";
    let model = claude_model(Status::Done, replied(reply), 30);
    assert_eq!(
        panel(&model, 8),
        [
            "",
            "I added retry with backoff to the",
            "HTTP client.",
            "for {}",
            "全部のテストが通りました。",
            "⠋ agents 2",
            "⠋ bg 1",
            "± +128 -14 (3 files)",
        ]
    );
}

#[test]
fn a_short_finished_preview_keeps_the_end_of_the_reply_and_the_changes() {
    let model = claude_model(Status::Idle, replied("one\ntwo\nthree\nfour"), 18);
    assert_eq!(
        panel(&model, 4),
        ["four", "⠋ agents 2", "⠋ bg 1", "± +128 -14 (3 files)"]
    );
}

mod rows;
