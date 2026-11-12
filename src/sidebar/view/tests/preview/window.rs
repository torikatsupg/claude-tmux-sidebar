//! The panel of a window without Claude.

use super::*;

/// The last of two windows, working in `path` under the home `/Users/me`.
fn window_model(command: &str, path: &str, branch: &str) -> Model {
    let model = windows_model(2, Mode::Pane, 30);
    let rows = model
        .snapshot
        .rows
        .iter()
        .map(|row| match row {
            Row::Window(p) => Row::Window(Pane {
                command: command.to_owned(),
                path: path.to_owned(),
                ..p.clone()
            }),
            other => other.clone(),
        })
        .collect();
    Model {
        snapshot: Snapshot {
            rows,
            ..model.snapshot.clone()
        },
        git: Some(Git {
            branch: branch.to_owned(),
            ..git(path)
        }),
        home: "/Users/me".to_owned(),
        ..model
    }
}

fn panel(model: &Model, width: u16) -> Vec<String> {
    lines(model, &EN, width, 30)
        .iter()
        .skip(21)
        .take(4)
        .map(text)
        .collect()
}

#[test]
fn a_long_directory_keeps_its_end_under_a_tilde() {
    let path = "/Users/me/ghq/github.com/torikatsupg/claude-tmux-sidebar";
    let model = window_model("nv\u{202e}im", path, "main");
    assert_eq!(
        panel(&model, WIDTH),
        [
            "nvim",
            "…/torikatsupg/claude-tmux-sidebar",
            "main",
            "± +3 -1 (1 file)"
        ]
    );
    let short = window_model("zsh", "/Users/me/src", "main");
    assert_eq!(
        panel(&short, WIDTH).get(1).map(String::as_str),
        Some("~/src")
    );
}

#[test]
fn a_narrow_panel_cuts_the_branch_and_changes_at_their_end() {
    let model = window_model("zsh", "/Users/me/src", "feature/issue-117-preview");
    assert_eq!(
        panel(&model, 12),
        ["zsh", "~/src", "feature/iss…", "± +3 -1 (1 …"]
    );
}
