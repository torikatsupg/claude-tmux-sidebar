//! The list the sidebar shows: one row per group heading, Claude session, its detail line, placeholder or gap.

use crate::listing::{Group, GroupName};
use crate::session::live::Claude;
use crate::tmux::pane::Pane;

/// The first screen row of the list (below the title and the rule).
pub const FIRST_ROW: u16 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Row {
    Gap,
    Heading(GroupName),
    Session(Claude),
    /// The line under a Claude session: its last prompt and context usage.
    Detail(Claude),
    /// The placeholder under a tmux session without Claude; it stands for that session.
    Empty(String),
}

/// A selectable row: a tmux session without Claude (its placeholder), or a Claude session by id.
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
            let empty = match &g.name {
                GroupName::Session(name) if g.claudes.is_empty() => Some(Row::Empty(name.clone())),
                _ => None,
            };
            gap.into_iter()
                .chain([Row::Heading(g.name.clone())])
                .chain(g.claudes.iter().flat_map(session_rows))
                .chain(empty)
        })
        .collect()
}

/// A Claude session, followed by its detail line when there is anything to show.
fn session_rows(claude: &Claude) -> Vec<Row> {
    let detail = !claude.record.prompt.is_empty() || claude.record.context.is_some();
    [Row::Session(claude.clone())]
        .into_iter()
        .chain(detail.then(|| Row::Detail(claude.clone())))
        .collect()
}

/// The key of a selectable row: the cursor stops on Claude sessions, and on the placeholder of a
/// tmux session without one, never on a heading.
pub fn key(row: &Row) -> Option<RowKey> {
    match row {
        Row::Empty(name) => Some(RowKey::Session(name.clone())),
        Row::Session(c) => Some(RowKey::Claude(c.record.session_id.clone())),
        Row::Heading(_) | Row::Detail(_) | Row::Gap => None,
    }
}

/// The selectable row a line belongs to: a detail line belongs to its session.
pub fn owner(row: &Row) -> Option<RowKey> {
    match row {
        Row::Detail(c) => Some(RowKey::Claude(c.record.session_id.clone())),
        _ => key(row),
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

/// Rows with the screen line they are drawn on, above screen line `end`.
pub fn placed(rows: &[Row], end: u16) -> impl Iterator<Item = (u16, &Row)> {
    (FIRST_ROW..end).zip(rows)
}

/// The selectable row drawn on screen line `y`.
pub fn at(rows: &[Row], end: u16, y: u16) -> Option<RowKey> {
    placed(rows, end)
        .find(|&(line, _)| line == y)
        .and_then(|(_, row)| owner(row))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Record;

    fn groups() -> Vec<Group> {
        let claude = Claude {
            record: Record {
                session_id: "c".to_owned(),
                prompt: "fix it".to_owned(),
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
}
