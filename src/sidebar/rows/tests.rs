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
    assert_eq!(at(&rows, 20, FIRST_ROW), None);
    assert_eq!(
        at(&rows, 20, FIRST_ROW + 1),
        Some(RowKey::Session("a".to_owned()))
    );
    assert_eq!(at(&rows, 20, FIRST_ROW + 2), None);
    assert_eq!(
        at(&rows, 20, FIRST_ROW + 5),
        Some(RowKey::Claude("c".to_owned()))
    );
    assert_eq!(at(&rows, 4, FIRST_ROW + 4), None);
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
