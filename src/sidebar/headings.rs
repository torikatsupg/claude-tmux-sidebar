//! The window headings of the tree (`@cts_label auto`), added to the finished list.

use super::label::Label;
use super::rows::Row;
use crate::listing::GroupName;
use crate::tmux::pane::{Pane, WindowId};

/// `rows` with a heading above the Claude rows of each window the label asks one for, and the name of a
/// named window alone in its session on the session's line when the label says so.
pub fn with_headings(rows: &[Row], label: Label) -> Vec<Row> {
    let inline = |name: &str| {
        rows.iter()
            .filter_map(session_pane)
            .find(|p| p.session == name && label.inline(p))
    };
    let renamed = rows.iter().map(|row| match row {
        Row::Heading(GroupName::Session(name)) => inline(name).map_or_else(
            || row.clone(),
            |pane| Row::NamedSession {
                session: pane.session.clone(),
                window: pane.window_name.clone(),
            },
        ),
        _ => row.clone(),
    });
    renamed
        .scan(None, |last: &mut Option<WindowId>, row| {
            let pane = session_pane(&row).filter(|p| label.window_heading(p));
            let first = pane.filter(|p| *last != Some(p.window_id)).cloned();
            *last = session_pane(&row).map_or(*last, |p| Some(p.window_id));
            Some(first.map(Row::WindowHeading).into_iter().chain([row]))
        })
        .flatten()
        .collect()
}

fn session_pane(row: &Row) -> Option<&Pane> {
    match row {
        Row::Session(claude) => claude.pane.as_ref(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::listing::Group;
    use crate::session::Record;
    use crate::session::live::Claude;
    use crate::sidebar::label::SingleWindow;
    use crate::sidebar::rows;
    use crate::tmux::pane::fixture::pane;

    const AUTO: Label = Label::Auto(SingleWindow::Inline);

    fn claude(id: u32, window: u32, named: bool, session_windows: u32) -> Claude {
        Claude {
            record: Record {
                session_id: format!("c{id}"),
                ..Record::default()
            },
            pane: Some(Pane {
                window_name: format!("w{window}"),
                window_named: named,
                session_windows,
                ..pane(id, window)
            }),
            latest: None,
        }
    }

    fn listed(claudes: Vec<Claude>) -> Vec<Row> {
        rows::rows(&[Group {
            name: GroupName::Session("s".to_owned()),
            claudes,
        }])
    }

    fn shape(rows: &[Row]) -> Vec<String> {
        rows.iter()
            .map(|row| match row {
                Row::Heading(_) => "session".to_owned(),
                Row::NamedSession { session, window } => format!("{session}/{window}"),
                Row::WindowHeading(p) => format!("window {}", p.window_name),
                Row::Session(c) => c.record.session_id.clone(),
                _ => "other".to_owned(),
            })
            .collect()
    }

    #[test]
    fn claudes_of_a_named_window_share_one_heading() {
        let rows = listed(vec![
            claude(1, 0, true, 2),
            claude(2, 0, true, 2),
            claude(3, 1, false, 2),
        ]);
        assert_eq!(
            shape(&with_headings(&rows, AUTO)),
            ["session", "window w0", "c1", "c2", "window w1", "c3"]
        );
    }

    #[test]
    fn each_named_window_gets_its_own_heading() {
        let rows = listed(vec![claude(1, 0, true, 2), claude(2, 1, true, 2)]);
        assert_eq!(
            shape(&with_headings(&rows, AUTO)),
            ["session", "window w0", "c1", "window w1", "c2"]
        );
    }

    #[test]
    fn a_named_window_alone_joins_the_session_line() {
        let rows = listed(vec![claude(1, 0, true, 1)]);
        assert_eq!(shape(&with_headings(&rows, AUTO)), ["s/w0", "c1"]);
    }

    #[test]
    fn the_lines_option_keeps_the_window_on_its_own_line() {
        let rows = listed(vec![claude(1, 0, true, 1)]);
        let lines = Label::Auto(SingleWindow::Lines);
        assert_eq!(
            shape(&with_headings(&rows, lines)),
            ["session", "window w0", "c1"]
        );
    }

    #[test]
    fn other_labels_and_windows_stay_as_they_are() {
        let named = listed(vec![claude(1, 0, true, 2)]);
        for label in [Label::Dir, Label::Window] {
            assert_eq!(with_headings(&named, label), named);
        }
        let automatic = listed(vec![claude(1, 0, false, 1)]);
        assert_eq!(with_headings(&automatic, AUTO), automatic);
    }

    #[test]
    fn headings_are_not_selectable() {
        let rows = with_headings(&listed(vec![claude(1, 0, true, 2)]), AUTO);
        let heading = rows.iter().filter(|r| matches!(r, Row::WindowHeading(_)));
        assert_eq!(heading.filter(|r| rows::owner(r).is_some()).count(), 0);
        assert_eq!(rows::keys(&rows).len(), 1);
    }
}
