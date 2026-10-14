//! One Claude session row: status icon, `window:directory`, and the status text on the right.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use super::DIM;
use crate::i18n::Messages;
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

fn label(status: Status, messages: &Messages) -> &'static str {
    match status {
        Status::Waiting => messages.waiting,
        Status::Running => messages.running,
        Status::Done => messages.done,
        Status::Idle => messages.idle,
    }
}

/// ` ● 2:api                     Bash running 3s `
pub fn line(model: &Model, messages: &Messages, claude: &Claude, width: usize) -> Line<'static> {
    let (_, color) = look(claude.record.state);
    let right = truncate(
        &status_text(model, messages, claude),
        width.saturating_sub(MIN_NAME_WIDTH),
    );
    let name = truncate(&name(claude), width.saturating_sub(5 + text::width(&right)));
    let here = claude
        .pane
        .as_ref()
        .is_some_and(|p| Some(p.window_id) == model.snapshot.window);
    let gap = width.saturating_sub(4 + text::width(&name) + text::width(&right));
    Line::from(vec![
        Span::raw(" "),
        icon(model, claude.record.state),
        Span::raw(" "),
        Span::styled(name, bold_if(here)),
        Span::raw(" ".repeat(gap)),
        Span::styled(right, color),
        Span::raw(" "),
    ])
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

/// `[detail ]label age`; the detail (tool name) only while running or waiting.
fn status_text(model: &Model, messages: &Messages, claude: &Claude) -> String {
    let record = &claude.record;
    let since = record.since.unwrap_or(record.updated_at);
    let base = format!(
        "{} {}",
        label(record.state, messages),
        age(model.now.saturating_sub(since))
    );
    let busy =
        matches!(record.state, Status::Running | Status::Waiting) && !record.detail.is_empty();
    if busy {
        format!("{} {base}", record.detail)
    } else {
        base
    }
}
