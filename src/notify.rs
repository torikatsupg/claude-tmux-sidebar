//! `@cts_notify`: tells the user when a Claude session starts waiting for them or is done.

pub mod place;
pub mod send;

use std::fs;
use std::path::Path;

use crate::i18n::Messages;
use crate::session::{Record, Status};
use crate::status::{DONE_MARK, WAITING_MARK};
use place::Place;

/// Where `sidebar.tmux` copies `@cts_notify`, in the state directory: the hooks read it there so that
/// they never wait on tmux.
const MODE_FILE: &str = "notify";
const TITLE: &str = "Claude Code";
/// Claude Code sends a prompt notification only after the prompt has waited about six seconds. One that
/// comes sooner after the previous hook is for a prompt already answered; whole seconds round it down to 5.
const PROMPT_WAIT: u64 = 5;

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

    /// The mode `sidebar.tmux` recorded in `dir`; `Off` when there is none.
    pub fn read(dir: &Path) -> Self {
        Self::parse(fs::read_to_string(dir.join(MODE_FILE)).ok().as_deref())
    }

    pub fn tmux(self) -> bool {
        matches!(self, Self::Tmux | Self::All)
    }

    pub fn os(self) -> bool {
        matches!(self, Self::Os | Self::All)
    }
}

/// Whether the change from `prev` to `next` asks for the user. A status the session already had is not
/// news. The state machine ignores a late prompt notification only until the next tool call or prompt, so
/// one that `by_notification` brings too soon after the previous hook is not news either.
pub fn worth(prev: Option<&Record>, next: &Record, by_notification: bool) -> bool {
    let late = by_notification
        && prev.is_some_and(|p| next.updated_at.saturating_sub(p.updated_at) < PROMPT_WAIT);
    next.state.needs_attention() && prev.map(|p| p.state) != Some(next.state) && !late
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
        // notify-send reads the body as Pango markup.
        let body = text
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;");
        ("notify-send", vec![TITLE.to_owned(), body])
    }
}

#[cfg(test)]
mod tests;
