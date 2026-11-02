//! tmux panes as parsed from `list-panes`, and the ids that name them.

use std::fmt;

/// A pane id such as `%12`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PaneId(u32);

/// A window id such as `@3`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(u32);

impl PaneId {
    pub fn parse(s: &str) -> Option<Self> {
        s.strip_prefix('%')?.parse().ok().map(Self)
    }
}

impl WindowId {
    pub fn parse(s: &str) -> Option<Self> {
        s.strip_prefix('@')?.parse().ok().map(Self)
    }
}

impl fmt::Display for PaneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.0)
    }
}

/// One pane of the tmux server.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // mirrors tmux's own flags
pub struct Pane {
    pub id: PaneId,
    pub session: String,
    pub window_index: u32,
    pub window_name: String,
    /// The window name was set by hand: tmux turns `automatic-rename` off for such a window.
    pub window_named: bool,
    pub pane_index: u32,
    /// How many windows the pane's session has.
    pub session_windows: u32,
    pub window_id: WindowId,
    pub sidebar: bool,
    pub active: bool,
    pub last: bool,
    pub window_active: bool,
}

/// The `list-panes -F` format that [`parse`] reads.
pub const FORMAT: &str = "#{pane_id}\t#{session_name}\t#{window_index}\t#{pane_index}\t#{window_id}\t\
                          #{@cts_sidebar}\t#{pane_active}\t#{pane_last}\t#{window_active}\t#{session_windows}\t#{automatic-rename}\t#{window_name}";

/// Parses `list-panes -F FORMAT` output, skipping malformed lines.
pub fn parse(output: &str) -> Vec<Pane> {
    output.lines().filter_map(parse_line).collect()
}

fn parse_line(line: &str) -> Option<Pane> {
    // The window name is last and takes the rest of the line, because a name may contain a tab.
    let fields: Vec<&str> = line.splitn(12, '\t').collect();
    let [fields @ .., automatic_rename, window_name] = fields.as_slice() else {
        return None;
    };
    Some(Pane {
        window_name: (*window_name).to_owned(),
        window_named: *automatic_rename == "0",
        ..parse_fields(fields)?
    })
}

/// The fields before the window's name.
fn parse_fields(fields: &[&str]) -> Option<Pane> {
    let [
        id,
        session,
        window_index,
        pane_index,
        window_id,
        sidebar,
        active,
        last,
        window_active,
        session_windows,
    ] = fields
    else {
        return None;
    };
    Some(Pane {
        id: PaneId::parse(id)?,
        session: (*session).to_owned(),
        window_index: window_index.parse().ok()?,
        window_name: String::new(),
        window_named: false,
        pane_index: pane_index.parse().ok()?,
        session_windows: session_windows.parse().ok()?,
        window_id: WindowId::parse(window_id)?,
        sidebar: *sidebar == "1",
        active: *active == "1",
        last: *last == "1",
        window_active: *window_active == "1",
    })
}

/// The pane the user works in: the active non-sidebar pane, or the previous one while the sidebar has the focus.
pub fn focused(panes: &[Pane], window: WindowId) -> Option<&Pane> {
    let real: Vec<&Pane> = panes
        .iter()
        .filter(|p| p.window_id == window && !p.sidebar)
        .collect();
    real.iter()
        .find(|p| p.active)
        .or_else(|| real.iter().find(|p| p.last))
        .or_else(|| real.first())
        .copied()
}

/// The sidebar pane of a window, if it has one.
pub fn sidebar_of(panes: &[Pane], window: WindowId) -> Option<&Pane> {
    panes.iter().find(|p| p.window_id == window && p.sidebar)
}

#[cfg(test)]
pub mod fixture {
    use super::{Pane, PaneId, WindowId};

    /// A pane of session `s` with pane index equal to its id.
    pub fn pane(id: u32, window: u32) -> Pane {
        Pane {
            id: PaneId(id),
            session: "s".to_owned(),
            window_index: window,
            window_name: "zsh".to_owned(),
            window_named: false,
            pane_index: id,
            session_windows: 1,
            window_id: WindowId(window),
            sidebar: false,
            active: false,
            last: false,
            window_active: true,
        }
    }
}

#[cfg(test)]
mod tests;
