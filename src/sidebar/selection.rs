//! Which row is selected, and how it moves.

use super::rows::{self, RowKey};
use super::snapshot::Snapshot;
use crate::jump::Direction;
use crate::tmux::pane::PaneId;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    pub selected: Option<RowKey>,
    /// The focused pane and its row when the selection last followed it.
    pub synced: Option<(PaneId, RowKey)>,
}

/// Re-selects after a reload. The selection belongs to the user only while the sidebar has the focus;
/// otherwise it mirrors the focused pane, and it always follows when the focus moves.
pub fn follow(selection: Selection, snapshot: &Snapshot, popup: bool) -> Selection {
    let keys = rows::keys(&snapshot.rows);
    let focus = snapshot
        .focus
        .as_ref()
        .map(|p| (p.id, rows::key_for_pane(&snapshot.rows, p)))
        .filter(|(_, key)| keys.contains(key));
    let mirror = !popup && !snapshot.sidebar_active;
    let synced = focus
        .clone()
        .filter(|f| mirror || selection.synced.as_ref() != Some(f));
    let selected = synced
        .as_ref()
        .map(|(_, key)| key.clone())
        .or(selection.selected)
        .filter(|key| keys.contains(key))
        .or_else(|| focus.clone().map(|(_, key)| key))
        .or_else(|| keys.first().cloned());
    Selection {
        selected,
        synced: synced.or(selection.synced),
    }
}

/// Moves by `delta` rows, wrapping around past either end.
pub fn moved(keys: &[RowKey], selected: Option<&RowKey>, delta: isize) -> Option<RowKey> {
    let len = isize::try_from(keys.len()).ok().filter(|&n| n > 0)?;
    let current = selected
        .and_then(|s| keys.iter().position(|k| k == s))
        .and_then(|i| isize::try_from(i).ok())
        .unwrap_or(0);
    let target = usize::try_from(current.wrapping_add(delta).rem_euclid(len)).ok()?;
    keys.get(target).cloned()
}

/// The next / previous of `hits` (rows that need the user), wrapping around.
pub fn attention(
    keys: &[RowKey],
    hits: &[RowKey],
    selected: Option<&RowKey>,
    direction: Direction,
) -> Option<RowKey> {
    let index = |k: &RowKey| keys.iter().position(|x| x == k);
    let here = selected.and_then(index);
    let after = |k: &&RowKey| index(k) > here;
    let before = |k: &&RowKey| index(k) < here && here.is_some();
    match direction {
        Direction::Next => hits.iter().find(after).or_else(|| hits.first()),
        Direction::Prev => hits.iter().rev().find(before).or_else(|| hits.last()),
    }
    .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(n: u8) -> RowKey {
        RowKey::Claude(n.to_string())
    }

    #[test]
    fn moves_wrap_around() {
        let keys = [key(1), key(2), key(3)];
        assert_eq!(moved(&keys, Some(&key(1)), 1), Some(key(2)));
        assert_eq!(moved(&keys, Some(&key(1)), -1), Some(key(3)));
        assert_eq!(moved(&keys, Some(&key(3)), 1), Some(key(1)));
        assert_eq!(moved(&keys, None, 2), Some(key(3)));
        assert_eq!(moved(&[], None, 1), None);
    }

    #[test]
    fn attention_wraps_both_ways() {
        let keys = [key(1), key(2), key(3)];
        let hits = [key(1), key(3)];
        assert_eq!(
            attention(&keys, &hits, Some(&key(1)), Direction::Next),
            Some(key(3))
        );
        assert_eq!(
            attention(&keys, &hits, Some(&key(3)), Direction::Next),
            Some(key(1))
        );
        assert_eq!(
            attention(&keys, &hits, Some(&key(2)), Direction::Prev),
            Some(key(1))
        );
        assert_eq!(
            attention(&keys, &hits, Some(&key(1)), Direction::Prev),
            Some(key(3))
        );
    }
}
