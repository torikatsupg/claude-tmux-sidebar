//! Decisions about adding and removing the sidebar pane, and the layout saved around it.

use itertools::Itertools;

use crate::tmux::pane::{Pane, PaneId};

/// Window options that remember the layout from before the sidebar was added.
pub const LAYOUT_OPTION: &str = "@cts_layout";
pub const PANES_OPTION: &str = "@cts_panes";

/// The layout of a window before its sidebar was added, and the panes it described.
#[derive(Debug, PartialEq, Eq)]
pub struct Saved {
    pub layout: String,
    pub panes: Vec<PaneId>,
}

/// The window's own panes, in tmux's order.
pub fn real_panes(panes: &[Pane]) -> Vec<PaneId> {
    panes.iter().filter(|p| !p.sidebar).map(|p| p.id).collect()
}

/// Pane ids as stored in [`PANES_OPTION`].
pub fn encode(ids: &[PaneId]) -> String {
    ids.iter().join(" ")
}

pub fn decode(s: &str) -> Vec<PaneId> {
    s.split_whitespace().filter_map(PaneId::parse).collect()
}

/// A window gets a sidebar unless it has one; a zoomed window would unzoom, so it gets one on the next visit.
pub fn needs_sidebar(panes: &[Pane], zoomed: bool) -> bool {
    !zoomed && !panes.iter().any(|p| p.sidebar)
}

/// The layout to restore after the sidebar is gone: only while the window still has the panes it was saved with.
pub fn restorable<'a>(saved: Option<&'a Saved>, current: &[Pane]) -> Option<&'a str> {
    saved
        .filter(|s| s.panes == real_panes(current))
        .map(|s| s.layout.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmux::pane::fixture::pane;

    fn saved(ids: &str) -> Saved {
        Saved {
            layout: "abcd,80x24,0,0".to_owned(),
            panes: decode(ids),
        }
    }

    #[test]
    fn pane_lists_round_trip_and_read_older_formats() {
        let ids = decode("%1 %2 ");
        assert_eq!(encode(&ids), "%1 %2");
    }

    #[test]
    fn restores_only_with_the_same_panes() {
        let current = [
            pane(1, 0),
            Pane {
                sidebar: true,
                ..pane(5, 0)
            },
            pane(2, 0),
        ];
        assert_eq!(
            restorable(Some(&saved("%1 %2")), &current),
            Some("abcd,80x24,0,0")
        );
        assert_eq!(restorable(Some(&saved("%1 %2 %3")), &current), None);
        assert_eq!(restorable(None, &current), None);
    }

    #[test]
    fn skips_windows_with_a_sidebar_or_zoomed() {
        let plain = [pane(1, 0)];
        assert!(needs_sidebar(&plain, false));
        assert!(!needs_sidebar(&plain, true));
        assert!(!needs_sidebar(
            &[Pane {
                sidebar: true,
                ..pane(1, 0)
            }],
            false
        ));
    }
}
