//! The preview of the cursor row's pane (`@cts_preview`): at the bottom of the list, in a tmux popup, or off.

mod body;
mod fit;
pub mod git;
mod panel;
mod work;
mod wrap;

pub use body::{Subject, ask, subject, tidy};
pub use git::{Changes, Git, GitRead, shortstat};
pub use panel::{Context, Group, Item, items};
pub use wrap::wrap;

use ratatui::style::Color;

use super::rows::{self, RowKey};
use super::snapshot::Snapshot;
use super::theme;
use crate::tmux::pane::PaneId;

/// The most lines the preview takes, its rule included.
const MAX_HEIGHT: u16 = 11;
/// Below this the preview would show too little to be worth the list lines it takes.
const MIN_HEIGHT: u16 = 3;
/// The list's own keys, which the popup key must not take over.
const LIST_KEYS: [char; 9] = ['j', 'k', 'n', 'p', 'g', 'G', 'l', 'q', 'r'];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// The option's default is `Pane`; `Default` keeps a bare model drawing the list alone.
    #[default]
    Off,
    Pane,
    Popup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preview {
    pub mode: Mode,
    /// The key that opens the popup.
    pub key: char,
    /// The color of the rule above the preview, from tmux's pane borders; `None` takes the dim color.
    pub rule: Option<Color>,
}

impl Default for Preview {
    fn default() -> Self {
        Self {
            mode: Mode::Off,
            key: ' ',
            rule: None,
        }
    }
}

impl Preview {
    /// `@cts_preview` (`pane` unless `popup` or `off`) and `@cts_preview_key` (`Space` or one character
    /// that is not one of the list's keys).
    pub fn from_options(option: impl Fn(&str) -> Option<String>) -> Self {
        let mode = match option("@cts_preview").as_deref() {
            Some("popup") => Mode::Popup,
            Some("off") => Mode::Off,
            _ => Mode::Pane,
        };
        let key = option("@cts_preview_key")
            .as_deref()
            .and_then(key)
            .unwrap_or(' ');
        Self {
            mode,
            key,
            rule: None,
        }
    }

    /// The key as the help line shows it.
    pub fn key_label(self) -> String {
        match self.key {
            ' ' => "␣".to_owned(),
            key => key.to_string(),
        }
    }
}

fn key(value: &str) -> Option<char> {
    let mut chars = value.chars();
    match (chars.next(), chars.next()) {
        _ if value.eq_ignore_ascii_case("space") => Some(' '),
        (Some(c), None) if !c.is_control() && !LIST_KEYS.contains(&c) => Some(c),
        _ => None,
    }
}

/// The fg of a `pane-border-style`: tmux's default (the terminal's text color) when it sets none, and
/// `None` when it cannot be read.
pub fn border_color(style: Option<&str>) -> Option<Color> {
    let fg = style
        .into_iter()
        .flat_map(|s| s.split([',', ' ']))
        .find_map(|attribute| attribute.trim().strip_prefix("fg="));
    fg.map_or(Some(Color::Reset), theme::color)
}

/// The directory whose changes the panel shows: a Claude session's, or a window's pane's.
pub fn directory(snapshot: &Snapshot, selected: Option<&RowKey>) -> Option<String> {
    let dir = match subject(snapshot, selected)? {
        Subject::Claude(claude) => claude.record.cwd.clone(),
        Subject::Window(pane) => pane.path.clone(),
    };
    Some(dir).filter(|d| !d.is_empty())
}

/// The pane the cursor row stands for: a Claude session's or a window's; none for a tmux session's placeholder.
pub fn target(snapshot: &Snapshot, selected: Option<&RowKey>) -> Option<PaneId> {
    let selected = selected?;
    snapshot
        .rows
        .iter()
        .filter(|r| rows::key(r).as_ref() == Some(selected))
        .find_map(rows::pane)
        .map(|p| p.id)
}

/// The preview's lines, rule included, below a list area of `area` lines.
pub fn height(mode: Mode, area: u16) -> u16 {
    let height = (area / 3).min(MAX_HEIGHT);
    if mode == Mode::Pane && height >= MIN_HEIGHT {
        height
    } else {
        0
    }
}

/// The popup's `-w` and `-h` from `#{pane_width} #{pane_height} #{client_width} #{client_height}`: the pane
/// and its border, so that no line wraps, within the client. 80% each when tmux did not answer.
pub fn popup_size(sizes: &str) -> (String, String) {
    let numbers: Vec<u16> = sizes
        .split_whitespace()
        .filter_map(|n| n.parse().ok())
        .collect();
    match numbers.as_slice() {
        [width, height, client_width, client_height] => (
            width.saturating_add(2).min(*client_width).to_string(),
            height.saturating_add(2).min(*client_height).to_string(),
        ),
        _ => ("80%".to_owned(), "80%".to_owned()),
    }
}

#[cfg(test)]
mod tests;
