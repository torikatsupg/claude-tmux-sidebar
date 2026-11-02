use super::*;
use crate::session::Record;
use crate::tmux::pane::fixture::pane;

fn groups() -> Vec<Group> {
    let claude = Claude {
        record: Record {
            session_id: "c".to_owned(),
            prompt: "fix it".to_owned(),
            ..Record::default()
        },
        pane: None,
        latest: None,
    };
    vec![
        Group {
            name: GroupName::Session("a".to_owned()),
            claudes: vec![],
        },
        Group {
            name: GroupName::Outside,
            claudes: vec![claude],
        },
    ]
}

#[test]
fn lays_out_groups_with_gaps_and_placeholders() {
    let rows = rows(&groups());
    assert!(matches!(
        rows.as_slice(),
        [
            Row::Heading(GroupName::Session(_)),
            Row::Empty(_),
            Row::Gap,
            Row::Heading(GroupName::Outside),
            Row::Session(_),
            Row::Detail(_)
        ]
    ));
    assert_eq!(
        keys(&rows),
        [
            RowKey::Session("a".to_owned()),
            RowKey::Claude("c".to_owned())
        ]
    );
}

#[test]
fn maps_screen_lines_to_rows() {
    let rows = rows(&groups());
    assert_eq!(at(&rows, None, 20, FIRST_ROW), None);
    assert_eq!(
        at(&rows, None, 20, FIRST_ROW + 1),
        Some(RowKey::Session("a".to_owned()))
    );
    assert_eq!(at(&rows, None, 20, FIRST_ROW + 2), None);
    assert_eq!(
        at(&rows, None, 20, FIRST_ROW + 5),
        Some(RowKey::Claude("c".to_owned()))
    );
    assert_eq!(at(&rows, None, 4, FIRST_ROW + 4), None);
}

fn in_window(window: u32) -> Pane {
    Pane {
        session: "a".to_owned(),
        ..pane(window, window)
    }
}

/// Session `a` runs Claude `c` in window 2; windows 1 and 3 run no Claude.
fn with_plain_windows() -> Vec<Row> {
    let claude = Claude {
        record: Record {
            session_id: "c".to_owned(),
            ..Record::default()
        },
        pane: Some(in_window(2)),
        latest: None,
    };
    let groups = [Group {
        name: GroupName::Session("a".to_owned()),
        claudes: vec![claude],
    }];
    with_windows(&groups, &[in_window(1), in_window(3)])
}

#[test]
fn windows_without_claude_sit_in_tmux_order_among_sessions() {
    let rows = with_plain_windows();
    assert!(matches!(
        rows.as_slice(),
        [
            Row::Heading(_),
            Row::Window(w1),
            Row::Session(_),
            Row::Window(w3)
        ] if w1.window_index == 1 && w3.window_index == 3
    ));
    assert_eq!(
        keys(&rows),
        [
            RowKey::Window(in_window(1).window_id),
            RowKey::Claude("c".to_owned()),
            RowKey::Window(in_window(3).window_id)
        ]
    );
    assert_eq!(
        key_for_pane(&rows, &in_window(3)),
        RowKey::Window(in_window(3).window_id)
    );
}

#[test]
fn without_windows_the_rows_are_unchanged() {
    assert_eq!(with_windows(&groups(), &[]), rows(&groups()));
}

/// A heading and ten windows without Claude: eleven rows.
fn long_list() -> Vec<Row> {
    let groups = [Group {
        name: GroupName::Session("s".to_owned()),
        claudes: vec![],
    }];
    let windows: Vec<Pane> = (0..10)
        .map(|i| Pane {
            session: "s".to_owned(),
            ..pane(i, i)
        })
        .collect();
    with_windows(&groups, &windows)
}

fn last_key(rows: &[Row]) -> Option<RowKey> {
    keys(rows).last().cloned()
}

#[test]
fn a_list_that_fits_does_not_scroll() {
    let rows = long_list();
    let last = last_key(&rows);
    assert_eq!(offset(&rows, last.as_ref(), FIRST_ROW + 11), 0);
    assert_eq!(offset(&rows, None, FIRST_ROW + 3), 0);
}

#[test]
fn the_list_scrolls_just_enough_to_show_the_selected_row() {
    let rows = long_list();
    let last = last_key(&rows);
    let end = FIRST_ROW + 5;
    assert_eq!(offset(&rows, last.as_ref(), end), 6);
    let first = keys(&rows).first().cloned();
    assert_eq!(offset(&rows, first.as_ref(), end), 0);
}

#[test]
fn a_click_lands_on_the_row_drawn_after_scrolling() {
    let rows = long_list();
    let last = last_key(&rows);
    let end = FIRST_ROW + 5;
    assert_eq!(at(&rows, last.as_ref(), end, end - 1), last);
}

#[test]
fn the_detail_line_scrolls_in_with_its_session() {
    let rows = rows(&groups());
    let claude = RowKey::Claude("c".to_owned());
    assert_eq!(offset(&rows, Some(&claude), FIRST_ROW + 2), 4);
    assert_eq!(offset(&rows, Some(&claude), FIRST_ROW + 3), 3);
}

#[test]
fn a_single_line_list_keeps_the_session_over_its_detail() {
    let rows = rows(&groups());
    let claude = RowKey::Claude("c".to_owned());
    assert_eq!(offset(&rows, Some(&claude), FIRST_ROW + 1), 4);
}
