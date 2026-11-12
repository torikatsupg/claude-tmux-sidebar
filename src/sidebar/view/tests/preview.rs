use super::*;
use crate::sidebar::preview::{Captured, Mode, Preview};

/// `count` windows without Claude in one session, each one row.
fn window_rows(count: u32) -> Vec<Row> {
    let windows: Vec<Pane> = (0..count)
        .map(|i| Pane {
            window_name: format!("w{i}"),
            ..pane(i, i)
        })
        .collect();
    let groups = [Group {
        name: GroupName::Session("s".to_owned()),
        claudes: vec![],
    }];
    rows::with_windows(&groups, &windows)
}

/// `count` windows, the cursor on the last, and a capture of its pane.
fn windows_model(count: u32, mode: Mode, height: u16) -> Model {
    let rows = window_rows(count);
    let selected = rows::keys(&rows).last().cloned();
    let last = count.saturating_sub(1);
    Model {
        snapshot: Snapshot {
            rows,
            focused: true,
            ..Snapshot::default()
        },
        selection: Selection {
            selected,
            synced: None,
        },
        preview: Preview {
            mode,
            ..Preview::default()
        },
        captured: Some(Captured {
            pane: pane(last, last).id,
            lines: (1..=20).map(|n| format!("out {n}")).collect(),
        }),
        height,
        help: true,
        ..Model::default()
    }
}

/// `─ name ───…` across `width` cells.
fn rule(name: &str, width: usize) -> String {
    let used = text::width(name) + 3;
    format!("─ {name} {}", "─".repeat(width.saturating_sub(used)))
}

#[test]
fn the_preview_shows_the_tail_of_the_selected_pane_above_the_help() {
    let model = windows_model(2, Mode::Pane, 30);
    let screen = screen(&model, &EN);
    assert_eq!(screen.len(), 30);
    assert_eq!(model.list_end(30), 20);
    assert_eq!(screen.get(20), Some(&rule("s/1:w1", 34)));
    assert_eq!(screen.get(21).map(String::as_str), Some("out 13"));
    assert_eq!(screen.get(28).map(String::as_str), Some("out 20"));
    assert!(screen.last().is_some_and(|l| l.contains("jump")));
}

#[test]
fn a_capture_of_another_pane_is_not_shown() {
    let model = Model {
        selection: Selection::default(),
        ..windows_model(2, Mode::Pane, 30)
    };
    assert!(screen(&model, &EN).iter().all(|l| !l.contains("out")));
}

#[test]
fn the_list_scrolls_above_the_preview() {
    let model = windows_model(30, Mode::Pane, 30);
    let screen = screen(&model, &EN);
    let at = screen.iter().position(|l| l.contains("29:w29"));
    assert_eq!(at, Some(19));
    assert!(screen.iter().all(|l| !l.contains("0:w0")));
}

#[test]
fn the_preview_takes_at_most_a_third_and_none_on_short_panes() {
    assert_eq!(windows_model(2, Mode::Pane, 100).list_end(100), 88);
    assert_eq!(windows_model(2, Mode::Pane, 12).list_end(12), 8);
    assert_eq!(windows_model(2, Mode::Pane, 8).list_end(8), 7);
}

#[test]
fn off_draws_the_list_as_before() {
    let model = Model {
        preview: Preview::default(),
        captured: Some(Captured {
            pane: pane(1, 2).id,
            lines: vec!["out".to_owned()],
        }),
        height: 30,
        ..model(Status::Waiting, "Edit", true)
    };
    let model = Model {
        snapshot: Snapshot {
            focused: true,
            ..model.snapshot.clone()
        },
        ..model
    };
    let expected: Vec<String> = [
        "Claude ◆1 ",
        &"─".repeat(34),
        "work",
        "◆ api                     Edit 2s ",
        "└                                 ",
    ]
    .into_iter()
    .map(str::to_owned)
    .chain(std::iter::repeat_n(String::new(), 24))
    .chain(["⏎ jump  j/k move  q close".to_owned()])
    .collect();
    assert_eq!(screen(&model, &EN), expected);
}

fn unfocused(model: Model) -> Model {
    Model {
        snapshot: Snapshot {
            focused: false,
            ..model.snapshot.clone()
        },
        ..model
    }
}

#[test]
fn without_the_focus_the_list_takes_the_full_height() {
    let model = unfocused(windows_model(2, Mode::Pane, 30));
    assert_eq!(model.list_end(30), 29);
    assert!(screen(&model, &EN).iter().all(|l| !l.contains("out")));
}

#[test]
fn scrolling_and_clicks_follow_the_list_height_as_the_focus_moves() {
    let focused = windows_model(30, Mode::Pane, 30);
    let last = rows::keys(&focused.snapshot.rows).last().cloned();
    for (model, y) in [(unfocused(focused.clone()), 28), (focused, 19)] {
        let end = model.list_end(30);
        let screen = screen(&model, &EN);
        assert_eq!(
            screen.iter().position(|l| l.contains("29:w29")),
            Some(usize::from(y))
        );
        assert_eq!(rows::at(&model.snapshot.rows, last.as_ref(), end, y), last);
    }
}

#[test]
fn the_help_shows_the_popup_key_only_for_popup() {
    let help = |mode, messages| {
        screen(&windows_model(2, mode, 30), messages)
            .last()
            .cloned()
    };
    assert_eq!(
        help(Mode::Popup, &EN).as_deref(),
        Some("⏎ jump  j/k move  q close  ␣ peek")
    );
    assert_eq!(
        help(Mode::Popup, &JA).as_deref(),
        Some("⏎ 移動  j/k 選択  q 閉じる  ␣ 中身")
    );
    assert_eq!(
        help(Mode::Pane, &EN).as_deref(),
        Some("⏎ jump  j/k move  q close")
    );
    let other_key = Model {
        preview: Preview {
            mode: Mode::Popup,
            key: 'v',
            ..Preview::default()
        },
        ..windows_model(2, Mode::Popup, 30)
    };
    assert!(
        screen(&other_key, &EN)
            .last()
            .is_some_and(|l| l.ends_with("v peek"))
    );
}

mod content;
