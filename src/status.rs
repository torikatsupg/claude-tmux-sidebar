//! `status`: how many Claude sessions wait, run and are done, as one line for tmux's `status-right`.

use std::io::{self, Write};

use crate::context::Context;
use crate::session::{Status, live};
use crate::tmux;

pub const WAITING_MARK: &str = "◆";
pub const RUNNING_MARK: &str = "⠹";
pub const DONE_MARK: &str = "✓";

/// The number of sessions in each status the line shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub waiting: usize,
    pub running: usize,
    pub done: usize,
}

impl Counts {
    pub fn of(states: impl IntoIterator<Item = Status>) -> Self {
        states
            .into_iter()
            .fold(Self::default(), |counts, state| match state {
                Status::Waiting => Self {
                    waiting: counts.waiting + 1,
                    ..counts
                },
                Status::Running => Self {
                    running: counts.running + 1,
                    ..counts
                },
                Status::Done => Self {
                    done: counts.done + 1,
                    ..counts
                },
                Status::Idle => counts,
            })
    }
}

/// tmux colors for each count, from the sidebar's `@cts_color_*` options.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Colors {
    pub waiting: String,
    pub running: String,
    pub done: String,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            waiting: "yellow".to_owned(),
            running: "green".to_owned(),
            done: "cyan".to_owned(),
        }
    }
}

impl Colors {
    /// The options looked up with `option`; an unset one, or one that would break the style, keeps its default.
    pub fn from_options(option: impl Fn(&str) -> Option<String>) -> Self {
        let pick = |role: &str, default: String| {
            option(&format!("@cts_color_{role}"))
                .map(|v| v.trim().to_owned())
                .filter(|v| {
                    !v.is_empty() && v.chars().all(|c| c.is_ascii_alphanumeric() || c == '#')
                })
                .unwrap_or(default)
        };
        let d = Self::default();
        Self {
            waiting: pick("waiting", d.waiting),
            running: pick("running", d.running),
            done: pick("done", d.done),
        }
    }
}

/// `◆2 ⠹1 ✓3`, leaving out the statuses no session is in; empty when there are none.
pub fn line(counts: Counts, colors: Option<&Colors>) -> String {
    let color = |pick: fn(&Colors) -> &String| colors.map(pick);
    [
        (WAITING_MARK, counts.waiting, color(|c| &c.waiting)),
        (RUNNING_MARK, counts.running, color(|c| &c.running)),
        (DONE_MARK, counts.done, color(|c| &c.done)),
    ]
    .into_iter()
    .filter(|&(_, n, _)| n > 0)
    .map(|(mark, n, color)| match color {
        Some(color) => format!("#[fg={color}]{mark}{n}#[fg=default]"),
        None => format!("{mark}{n}"),
    })
    .collect::<Vec<_>>()
    .join(" ")
}

/// Prints the line for the live sessions of this tmux server, colored when `color` is set.
pub fn run(ctx: &Context, color: bool) {
    let claudes = live::load(&ctx.state_dir, &ctx.socket, &tmux::panes());
    let counts = Counts::of(claudes.iter().map(|c| c.record.state));
    let colors = color.then(|| Colors::from_options(tmux::global_option));
    // A status line that cannot be written has no one to report to.
    let _ = io::stdout().write_all(line(counts, colors.as_ref()).as_bytes());
}

#[cfg(test)]
mod tests;
