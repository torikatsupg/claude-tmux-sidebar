use super::*;

fn window_model(here: bool) -> Model {
    let window = Pane {
        window_name: "zsh".to_owned(),
        ..pane(4, 3)
    };
    let groups = [Group {
        name: GroupName::Session("s".to_owned()),
        claudes: vec![],
    }];
    let snapshot = Snapshot {
        rows: rows::with_windows(&groups, &[window]),
        window: here.then(|| pane(4, 3).window_id),
        ..Snapshot::default()
    };
    Model {
        snapshot,
        height: 8,
        ..Model::default()
    }
}

#[test]
fn a_window_without_claude_is_a_quiet_index_and_name() {
    let model = window_model(false);
    let lines = lines(&model, &EN, WIDTH, model.height);
    let row = lines.get(3).cloned().unwrap_or_default();
    assert_eq!(text(&row), " · 3:zsh");
    let theme = Theme::DEFAULT;
    assert!(
        row.spans
            .iter()
            .filter(|s| !s.content.trim().is_empty())
            .all(|s| [Some(theme.dim), Some(theme.muted)].contains(&s.style.fg))
    );
}

#[test]
fn the_window_you_are_in_is_marked() {
    let model = window_model(true);
    assert_eq!(
        screen(&model, &EN).get(3).map(String::as_str),
        Some("▌· 3:zsh")
    );
}

/// Ten windows in a pane of six lines, with the cursor on the last one.
fn scrolled_model() -> Model {
    let windows: Vec<Pane> = (0..10)
        .map(|i| Pane {
            session: "s".to_owned(),
            window_name: format!("w{i}"),
            ..pane(i, i)
        })
        .collect();
    let groups = [Group {
        name: GroupName::Session("s".to_owned()),
        claudes: vec![],
    }];
    let rows = rows::with_windows(&groups, &windows);
    let selected = rows::keys(&rows).last().cloned();
    Model {
        snapshot: Snapshot {
            rows,
            ..Snapshot::default()
        },
        selection: Selection {
            selected,
            ..Selection::default()
        },
        height: 6,
        help: true,
        ..Model::default()
    }
}

#[test]
fn the_list_scrolls_to_keep_the_cursor_on_screen() {
    let screen = screen(&scrolled_model(), &EN);
    assert_eq!(screen.len(), 6);
    assert_eq!(screen.get(4).map(|s| s.trim_end()), Some(" · 9:w9"));
    assert!(screen.get(5).is_some_and(|s| s.contains("jump")));
}

#[test]
fn in_the_tree_a_window_without_claude_sits_at_the_window_level() {
    use crate::sidebar::label::{Label, SingleWindow};
    let model = Model {
        label: Label::Auto(SingleWindow::Inline),
        ..window_model(false)
    };
    let lines = lines(&model, &EN, WIDTH, model.height);
    let row = lines.get(3).cloned().unwrap_or_default();
    assert_eq!(text(&row), "   3:zsh");
    assert!(
        row.spans
            .iter()
            .filter(|s| !s.content.trim().is_empty())
            .all(|s| s.style.fg == Some(Theme::DEFAULT.dim))
    );
}
