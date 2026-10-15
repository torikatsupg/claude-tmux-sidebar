//! One Claude session row: status icon, `window:directory`, and the tool and age on the right.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use super::{ACCENT, DIM};
use crate::session::Status;
use crate::session::live::Claude;
use crate::sidebar::update::Model;
use crate::text::{self, age, truncate};

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
/// The name column keeps at least this many cells when the status text is long.
const MIN_NAME_WIDTH: usize = 12;

pub fn look(status: Status) -> (&'static str, Color) {
    match status {
        Status::Waiting => ("◆", Color::Yellow),
        Status::Running => ("●", Color::Green),
        Status::Done => ("✓", Color::Cyan),
        Status::Idle => ("○", DIM),
    }
}

/// `▌● 2:api                             Bash 3s `; the bar marks the window you are in.
pub fn line(model: &Model, claude: &Claude, width: usize) -> Line<'static> {
    let (_, color) = look(claude.record.state);
    let right = truncate(
        &status_text(model, claude),
        width.saturating_sub(MIN_NAME_WIDTH),
    );
    let name = truncate(&name(claude), width.saturating_sub(5 + text::width(&right)));
    let here = claude
        .pane
        .as_ref()
        .is_some_and(|p| Some(p.window_id) == model.snapshot.window);
    let gap = width.saturating_sub(4 + text::width(&name) + text::width(&right));
    Line::from(vec![
        marker(here),
        icon(model, claude.record.state),
        Span::raw(" "),
        Span::styled(name, bold_if(here)),
        Span::raw(" ".repeat(gap)),
        Span::styled(right, color),
        Span::raw(" "),
    ])
}

/// The bar on the left edge for the window you are in.
fn marker(here: bool) -> Span<'static> {
    if here {
        Span::styled("▌", ACCENT)
    } else {
        Span::raw(" ")
    }
}

/// The status icon; a running session spins.
fn icon(model: &Model, status: Status) -> Span<'static> {
    let (icon, color) = look(status);
    let spinner = SPINNER
        .get(model.frame % SPINNER.len())
        .filter(|_| status == Status::Running);
    let style = Style::new()
        .fg(color)
        .add_modifier(bold_if(status == Status::Waiting));
    Span::styled(*spinner.unwrap_or(&icon), style)
}

fn bold_if(on: bool) -> Modifier {
    if on {
        Modifier::BOLD
    } else {
        Modifier::empty()
    }
}

/// `window:directory`, or just the directory outside tmux.
fn name(claude: &Claude) -> String {
    let dir = std::path::Path::new(&claude.record.cwd)
        .file_name()
        .map_or_else(|| "?".to_owned(), |d| d.to_string_lossy().into_owned());
    claude
        .pane
        .as_ref()
        .map_or_else(|| dir.clone(), |p| format!("{}:{dir}", p.window_index))
}

/// `[detail ]age`; the icon already shows the status, and the detail (tool name) is shown only while
/// running or waiting.
fn status_text(model: &Model, claude: &Claude) -> String {
    let record = &claude.record;
    let since = record.since.unwrap_or(record.updated_at);
    let base = age(model.now.saturating_sub(since));
    let busy =
        matches!(record.state, Status::Running | Status::Waiting) && !record.detail.is_empty();
    if busy {
        format!("{} {base}", record.detail)
    } else {
        base
    }
}
