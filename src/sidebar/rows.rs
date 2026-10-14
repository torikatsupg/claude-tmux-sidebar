//! The list the sidebar shows: one row per group heading, Claude session, placeholder or gap.

use crate::listing::{Group, GroupName};
use crate::session::live::Claude;
use crate::tmux::pane::Pane;

/// The first screen row of the list (below the title and the rule).
pub const FIRST_ROW: u16 = 2;
/// Rows kept free at the bottom for the help line.
const FOOTER_ROWS: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Row {
    Gap,
    Heading(GroupName),
    Session(Claude),
    /// A tmux session without Claude.
    Empty,
}

/// A selectable row: a tmux session heading, or a Claude session by id.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RowKey {
    Session(String),
    Claude(String),
}

pub fn rows(groups: &[Group]) -> Vec<Row> {
    groups
        .iter()
        .enumerate()
        .flat_map(|(i, g)| {
            let gap = (i > 0).then_some(Row::Gap);
            let empty = g.claudes.is_empty().then_some(Row::Empty);
            gap.into_iter()
                .chain([Row::Heading(g.name.clone())])
                .chain(g.claudes.iter().cloned().map(Row::Session))
                .chain(empty)
        })
        .collect()
}

/// The key of a selectable row; sessions outside tmux have no heading to jump to.
pub fn key(row: &Row) -> Option<RowKey> {
    match row {
        Row::Heading(GroupName::Session(name)) => Some(RowKey::Session(name.clone())),
        Row::Session(c) => Some(RowKey::Claude(c.record.session_id.clone())),
        Row::Heading(GroupName::Outside) | Row::Gap | Row::Empty => None,
    }
}

/// Selectable rows in display order.
pub fn keys(rows: &[Row]) -> Vec<RowKey> {
    rows.iter().filter_map(key).collect()
}

/// The row that stands for a pane: its Claude session, or else its tmux session.
pub fn key_for_pane(rows: &[Row], pane: &Pane) -> RowKey {
    rows.iter()
        .find_map(|r| match r {
            Row::Session(c) if c.record.pane_id() == Some(pane.id) => {
                Some(RowKey::Claude(c.record.session_id.clone()))
            }
            _ => None,
        })
        .unwrap_or_else(|| RowKey::Session(pane.session.clone()))
}

/// Rows with the screen line they are drawn on, for a terminal `height` lines tall.
pub fn placed(rows: &[Row], height: u16) -> impl Iterator<Item = (u16, &Row)> {
    (FIRST_ROW..height.saturating_sub(FOOTER_ROWS)).zip(rows)
}

/// The selectable row drawn on screen line `y`.
pub fn at(rows: &[Row], height: u16, y: u16) -> Option<RowKey> {
    placed(rows, height)
        .find(|&(line, _)| line == y)
        .and_then(|(_, row)| key(row))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Record;

    fn groups() -> Vec<Group> {
        let claude = Claude {
            record: Record {
                session_id: "c".to_owned(),
                ..Record::default()
            },
            pane: None,
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
                Row::Empty,
                Row::Gap,
                Row::Heading(GroupName::Outside),
                Row::Session(_)
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
        assert_eq!(
            at(&rows, 20, FIRST_ROW),
            Some(RowKey::Session("a".to_owned()))
        );
        assert_eq!(at(&rows, 20, FIRST_ROW + 1), None);
        assert_eq!(at(&rows, 4, FIRST_ROW + 4), None);
    }
}
