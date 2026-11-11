//! `@cts_notify`: tells the user when a Claude session starts waiting for them or is done.

pub mod place;
pub mod send;

use crate::i18n::Messages;
use crate::session::Status;
use crate::status::{DONE_MARK, WAITING_MARK};
use place::Place;

pub const OPTION: &str = "@cts_notify";
const TITLE: &str = "Claude Code";

/// Where notifications go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Off,
    Tmux,
    Os,
    All,
}

impl Mode {
    /// The option's value; unset or unknown is `Off`.
    pub fn parse(value: Option<&str>) -> Self {
        match value.map(str::trim) {
            Some("tmux") => Self::Tmux,
            Some("os") => Self::Os,
            Some("all") => Self::All,
            _ => Self::Off,
        }
    }

    pub fn tmux(self) -> bool {
        matches!(self, Self::Tmux | Self::All)
    }

    pub fn os(self) -> bool {
        matches!(self, Self::Os | Self::All)
    }
}

/// Whether moving from `prev` to `next` asks for the user. A status the session already had is not news;
/// late prompt notifications never reach here, as the state machine ignores them.
pub fn worth(prev: Option<Status>, next: Status) -> bool {
    next.needs_attention() && prev != Some(next)
}

/// `◆ api:2 waiting for you`: the mark, the tmux session (with the window index when it has several) and
/// the status.
pub fn text(status: Status, place: &Place, messages: &Messages) -> String {
    let (mark, label) = match status {
        Status::Waiting => (WAITING_MARK, messages.waiting),
        _ => (DONE_MARK, messages.done),
    };
    let target = if place.windows > 1 {
        format!("{}:{}", place.session, place.window_index)
    } else {
        place.session.clone()
    };
    format!("{mark} {target} {label}")
}

/// `display-message` expands formats, so a `#` in a session name is doubled.
pub fn tmux_text(text: &str) -> String {
    text.replace('#', "##")
}

/// The command that shows `text` as an OS notification. The text is passed as an argument, never as code.
pub fn os_command(text: &str, macos: bool) -> (&'static str, Vec<String>) {
    if macos {
        let script = [
            "on run argv",
            "display notification (item 2 of argv) with title (item 1 of argv)",
            "end run",
        ];
        let args = script
            .iter()
            .flat_map(|line| ["-e", line])
            .chain([TITLE, text])
            .map(str::to_owned)
            .collect();
        ("osascript", args)
    } else {
        ("notify-send", vec![TITLE.to_owned(), text.to_owned()])
    }
}

#[cfg(test)]
mod tests;
