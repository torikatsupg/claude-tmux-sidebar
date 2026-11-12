//! What the preview shows: the rule, and the text under it.

use super::*;
use crate::session::transcript::{Ask, Said};

#[test]
fn long_screen_lines_wrap_instead_of_being_cut() {
    let model = Model {
        captured: Some(Captured {
            pane: pane(1, 1).id,
            lines: vec![format!("error: {}end", "word ".repeat(10))],
        }),
        ..windows_model(2, Mode::Pane, 30)
    };
    let screen = screen(&model, &EN);
    assert_eq!(
        screen.get(21).map(String::as_str),
        Some("error: word word word word word")
    );
    assert_eq!(
        screen.get(22).map(String::as_str),
        Some("word word word word word end")
    );
    assert!(screen.iter().all(|l| !l.contains('…')));
}

#[test]
fn the_rule_names_the_row_and_takes_the_border_color() {
    let model = Model {
        preview: Preview {
            mode: Mode::Pane,
            rule: Some(Color::Indexed(238)),
            ..Preview::default()
        },
        ..windows_model(2, Mode::Pane, 30)
    };
    let rule = lines_of(&model, 20);
    assert!(
        rule.spans
            .iter()
            .all(|s| s.style.fg == Some(Color::Indexed(238)))
    );
    let dim = lines_of(&windows_model(2, Mode::Pane, 30), 20);
    assert!(
        dim.spans
            .iter()
            .all(|s| s.style.fg == Some(Theme::DEFAULT.dim))
    );
    let narrow = lines(&windows_model(30, Mode::Pane, 30), &EN, 10, 30);
    assert_eq!(narrow.get(20).map(text), Some("─ s/29:… ─".to_owned()));
}

fn lines_of(model: &Model, y: usize) -> Line<'static> {
    lines(model, &EN, WIDTH, 30)
        .get(y)
        .cloned()
        .unwrap_or_default()
}

fn claude_model(state: Status, said: Said) -> Model {
    let claude = Claude {
        record: record(state, ""),
        pane: Some(pane(1, 2)),
        latest: Some(Latest {
            branch: String::new(),
            usage: None,
            said,
        }),
    };
    let model = model_of(claude, true);
    Model {
        snapshot: Snapshot {
            focused: true,
            ..model.snapshot.clone()
        },
        preview: Preview {
            mode: Mode::Pane,
            ..Preview::default()
        },
        height: 30,
        ..model
    }
}

fn reply(text: &str) -> Said {
    Said {
        reply: text.to_owned(),
        ask: None,
    }
}

#[test]
fn a_claude_row_shows_the_end_of_its_last_reply_wrapped() {
    let text = "# Summary\n\n\n\nI fixed the failing test in the sidebar rows module.\n```rust\nlet x = 1;\n```\n日本語の説明もここに少し長めに書いておきます。";
    let screen = screen(&claude_model(Status::Done, reply(text)), &EN);
    assert_eq!(screen.get(20), Some(&rule("s/api", 34)));
    let body: Vec<&str> = screen.iter().skip(21).take(8).map(String::as_str).collect();
    assert_eq!(
        body,
        [
            "Summary",
            "",
            "I fixed the failing test in the",
            "sidebar rows module.",
            "let x = 1;",
            "日本語の説明もここに少し長めに書い",
            "ておきます。",
            "",
        ]
    );
}

#[test]
fn a_waiting_claude_shows_what_it_asks_for() {
    let said = Said {
        reply: "Let me clean up.".to_owned(),
        ask: Some(Ask {
            tool: "Bash".to_owned(),
            detail: "rm -rf build".to_owned(),
        }),
    };
    let waiting = screen(&claude_model(Status::Waiting, said.clone()), &EN);
    assert_eq!(
        waiting.get(21).map(String::as_str),
        Some("Bash: rm -rf build")
    );
    let done = screen(&claude_model(Status::Done, said), &EN);
    assert_eq!(done.get(21).map(String::as_str), Some("Let me clean up."));
    let question = Said {
        ask: Some(Ask {
            tool: "AskUserQuestion".to_owned(),
            detail: "Which database?".to_owned(),
        }),
        ..reply("")
    };
    let asked = screen(&claude_model(Status::Waiting, question), &EN);
    assert_eq!(asked.get(21).map(String::as_str), Some("Which database?"));
    let unknown = screen(&claude_model(Status::Waiting, reply("Only text.")), &EN);
    assert_eq!(unknown.get(21).map(String::as_str), Some("Only text."));
}
