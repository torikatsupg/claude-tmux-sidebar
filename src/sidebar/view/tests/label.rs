use super::*;
use crate::sidebar::headings::with_headings;
use crate::sidebar::label::{Label, SingleWindow};

fn window_model(name: &str) -> Model {
    let mut claude = busy_claude(1, None);
    claude.pane = claude.pane.map(|p| Pane {
        window_name: name.to_owned(),
        ..p
    });
    Model {
        label: Label::Window,
        ..model_of(claude, false)
    }
}

#[test]
fn the_window_label_replaces_the_directory_and_branch() {
    let screen = screen(&window_model("editor"), &EN);
    assert_eq!(
        screen.get(3).map(String::as_str),
        Some("✓ 2:editor                     2s ")
    );
    assert_eq!(
        screen.get(4).map(String::as_str),
        Some("└ fix the failing sidebar test p… ")
    );
}

#[test]
fn a_narrow_row_cuts_the_window_label() {
    let model = window_model("a long window name");
    let row = lines(&model, &EN, 16, model.height).get(3).map(text);
    assert_eq!(row.as_deref(), Some("✓ 2:a long … 2s "));
}

fn auto_model(named: bool, windows: u32, single: SingleWindow) -> Model {
    let mut claude = busy_claude(windows, None);
    claude.pane = claude.pane.map(|p| Pane {
        session: "work".to_owned(),
        window_name: "mywork".to_owned(),
        window_named: named,
        ..p
    });
    let model = model_of(claude, false);
    let label = Label::Auto(single);
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

#[test]
fn a_named_window_is_a_heading_over_indented_rows() {
    let screen = screen(&auto_model(true, 2, SingleWindow::Inline), &EN);
    assert_eq!(screen.get(3).map(String::as_str), Some(" 2:mywork"));
    let row = screen.get(4).cloned().unwrap_or_default();
    assert!(row.starts_with("  ✓ ") && !row.contains("2:"), "{row}");
    assert!(row.contains("feat"), "the branch stays: {row}");
    let detail = screen.get(5).cloned().unwrap_or_default();
    assert!(detail.starts_with("  └ "), "{detail}");
}

#[test]
fn a_window_tmux_named_is_a_heading_with_its_number() {
    let screen = screen(&auto_model(false, 2, SingleWindow::Inline), &EN);
    assert_eq!(screen.get(3).map(String::as_str), Some(" 2"));
    assert!(screen.get(4).is_some_and(|row| row.starts_with("  ✓ ")));
}

#[test]
fn indented_rows_keep_the_width() {
    let model = auto_model(true, 2, SingleWindow::Inline);
    let width = usize::from(WIDTH);
    let rows = screen(&model, &EN);
    assert_eq!(rows.get(4).map(|r| text::width(r)), Some(width));
    assert_eq!(rows.get(5).map(|r| text::width(r)), Some(width));
}

#[test]
fn a_named_window_alone_joins_the_session_line() {
    let screen = screen(&auto_model(true, 1, SingleWindow::Inline), &EN);
    assert_eq!(screen.get(2).map(String::as_str), Some("work/mywork"));
    assert!(screen.get(3).is_some_and(|row| row.starts_with(" ✓ ")));
}

#[test]
fn a_window_alone_without_a_name_puts_its_rows_under_the_session() {
    let screen = screen(&auto_model(false, 1, SingleWindow::Inline), &EN);
    assert_eq!(screen.get(2).map(String::as_str), Some("work"));
    assert!(screen.get(3).is_some_and(|row| row.starts_with(" ✓ ")));
}

#[test]
fn the_lines_option_gives_a_named_window_alone_its_own_heading() {
    let screen = screen(&auto_model(true, 1, SingleWindow::Lines), &EN);
    assert_eq!(screen.get(2).map(String::as_str), Some("work"));
    assert_eq!(screen.get(3).map(String::as_str), Some(" 2:mywork"));
    assert!(screen.get(4).is_some_and(|row| row.starts_with("  ✓ ")));
}
