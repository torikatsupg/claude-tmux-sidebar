use super::*;
use crate::sidebar::headings::with_headings;
use crate::sidebar::label::{Label, SingleWindow};

const ACTIVE: Color = Theme::DEFAULT.active;
const HERE: u32 = 2;

fn here(mut model: Model) -> Model {
    model.snapshot.window = Some(pane(1, HERE).window_id);
    model
}

fn backgrounds(model: &Model, row: usize) -> Vec<Option<Color>> {
    let lines = lines(model, &EN, WIDTH, model.height);
    let line = lines.get(row).cloned().unwrap_or_default();
    line.spans.iter().map(|s| s.style.bg).collect()
}

fn is_active(model: &Model, row: usize) -> bool {
    let bgs = backgrounds(model, row);
    !bgs.is_empty() && bgs.iter().all(|bg| *bg == Some(ACTIVE))
}

fn is_plain(model: &Model, row: usize) -> bool {
    backgrounds(model, row).iter().all(Option::is_none)
}

fn tree(model: Model) -> Model {
    let label = Label::Auto(SingleWindow::Inline);
    let rows = with_headings(&model.snapshot.rows, label);
    Model {
        label,
        snapshot: Snapshot {
            rows,
            ..model.snapshot
        },
        ..model
    }
}

fn named_pair() -> Model {
    let mut claude = busy_claude(2, None);
    claude.pane = claude.pane.map(|p| Pane {
        window_named: true,
        window_name: "mywork".to_owned(),
        ..p
    });
    model_of(claude, false)
}

fn window_only() -> Model {
    let window = Pane {
        window_name: "zsh".to_owned(),
        ..pane(1, HERE)
    };
    let groups = [Group {
        name: GroupName::Session("s".to_owned()),
        claudes: vec![],
    }];
    let rows = rows::with_windows(&groups, &[window]);
    let snapshot = Snapshot {
        rows,
        ..Snapshot::default()
    };
    Model {
        snapshot,
        height: 8,
        ..Model::default()
    }
}

#[test]
fn the_session_row_and_its_detail_row_have_the_active_background() {
    let model = here(model_of(busy_claude(1, None), false));
    assert!(is_active(&model, 3) && is_active(&model, 4));
}

#[test]
fn the_window_heading_has_the_active_background() {
    let model = here(tree(named_pair()));
    assert!(is_active(&model, 3));
    assert!(is_active(&model, 4) && is_active(&model, 5));
}

#[test]
fn a_window_without_claude_has_the_active_background() {
    assert!(is_active(&here(window_only()), 3));
}

#[test]
fn rows_of_other_windows_have_no_background() {
    let session = model_of(busy_claude(1, None), false);
    assert!(is_plain(&session, 3) && is_plain(&session, 4));
    let mut other = here(tree(named_pair()));
    other.snapshot.window = Some(pane(1, 9).window_id);
    assert!((3..6).all(|row| is_plain(&other, row)));
    assert!(is_plain(&window_only(), 3));
}

#[test]
fn the_title_and_the_session_heading_have_no_background() {
    let model = here(model_of(busy_claude(1, None), false));
    assert!(is_plain(&model, 0) && is_plain(&model, 2));
}

fn accent_spans(model: &Model, row: usize) -> Vec<String> {
    let lines = lines(model, &EN, WIDTH, model.height);
    let line = lines.get(row).cloned().unwrap_or_default();
    let accent = Theme::DEFAULT.accent;
    let spans = line.spans.iter();
    spans
        .filter(|s| s.style.fg == Some(accent))
        .map(|s| s.content.to_string())
        .collect()
}

#[test]
fn the_cursor_wins_over_active_and_the_window_number_takes_the_accent() {
    let model = here(model_of(busy_claude(3, None), true));
    let bgs = backgrounds(&model, 3);
    assert!(bgs.iter().all(|bg| *bg == Some(CURSOR)), "{bgs:?}");
    assert_eq!(accent_spans(&model, 3), ["2:"]);
    let detail = backgrounds(&model, 4);
    assert!(detail.iter().all(|bg| *bg == Some(CURSOR)), "{detail:?}");
}

#[test]
fn the_accent_is_off_without_the_cursor_or_outside_the_window() {
    let model = here(model_of(busy_claude(3, None), false));
    assert!(accent_spans(&model, 3).is_empty());
    let elsewhere = model_of(busy_claude(3, None), true);
    assert!(accent_spans(&elsewhere, 3).is_empty());
}

#[test]
fn a_window_without_claude_takes_the_cursor_and_the_accent_number() {
    let mut model = here(window_only());
    model.selection.selected = rows::keys(&model.snapshot.rows).last().cloned();
    let bgs = backgrounds(&model, 3);
    assert!(bgs.iter().all(|bg| *bg == Some(CURSOR)), "{bgs:?}");
    assert_eq!(accent_spans(&model, 3), ["2:"]);
}
