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
