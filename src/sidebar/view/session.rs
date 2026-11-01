//! One Claude session: status icon, directory, branch, and the tool and age on the right; below it the
//! last prompt and context usage.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::session::Status;
use crate::session::live::Claude;
use crate::sidebar::theme::Theme;
use crate::sidebar::update::Model;
use crate::text::{self, age, truncate};

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
/// The name column keeps at least this many cells when the status text is long.
const MIN_NAME_WIDTH: usize = 12;
/// A branch squeezed below this many cells is dropped rather than cut to a stub.
const MIN_BRANCH_WIDTH: usize = 4;
/// Context usage from this percent on is a warning, and from the next one on an alarm.
const CONTEXT_WARN: u8 = 50;
const CONTEXT_ALARM: u8 = 80;

pub fn look(theme: &Theme, status: Status) -> (&'static str, Color) {
    match status {
        Status::Waiting => ("◆", theme.waiting),
        Status::Running => ("●", theme.running),
        Status::Done => ("✓", theme.done),
        Status::Idle => ("○", theme.dim),
    }
}

/// `▌● 2:api main                        Bash 3s `; the bar marks the window you are in.
pub fn line(model: &Model, claude: &Claude, width: usize) -> Line<'static> {
    let (_, color) = look(&model.theme, claude.record.state);
    let right = truncate(
        &status_text(model, claude),
        width.saturating_sub(MIN_NAME_WIDTH),
    );
    let room = width.saturating_sub(5 + text::width(&right));
    let name = truncate(&name(claude), room);
    let branch = branch(claude.branch(), room.saturating_sub(text::width(&name)));
    let here = here(model, claude);
    let used = text::width(&name) + text::width(&branch) + text::width(&right);
    Line::from(vec![
        marker(model, here),
        icon(model, claude.record.state),
        Span::raw(" "),
        Span::styled(name, bold_if(here)),
        Span::styled(branch, model.theme.muted),
        Span::raw(" ".repeat(width.saturating_sub(4 + used))),
        Span::styled(right, color),
        Span::raw(" "),
    ])
}

/// `▌└ fix the failing test…               42% `: the prompt lines up with the name above it.
pub fn detail(model: &Model, claude: &Claude, width: usize) -> Line<'static> {
    let record = &claude.record;
    let theme = &model.theme;
    let windows = &model.snapshot.context_windows;
    let (context, color) = claude
        .context(windows)
        .map_or((String::new(), theme.muted), |c| {
            (format!("{c}%"), context_color(theme, c))
        });
    let reserved = if context.is_empty() {
        0
    } else {
        text::width(&context) + 1
    };
    let prompt = truncate(&record.prompt, width.saturating_sub(4 + reserved));
    let gap = width.saturating_sub(4 + text::width(&prompt) + text::width(&context));
    Line::from(vec![
        marker(model, here(model, claude)),
        Span::styled("└ ", theme.dim),
        Span::styled(prompt, theme.muted),
        Span::raw(" ".repeat(gap)),
        Span::styled(context, color),
        Span::raw(" "),
    ])
}

fn context_color(theme: &Theme, percent: u8) -> Color {
    if percent >= CONTEXT_ALARM {
        theme.critical
    } else if percent >= CONTEXT_WARN {
        theme.warning
    } else {
        theme.muted
    }
}

/// The session runs in the window the sidebar belongs to.
fn here(model: &Model, claude: &Claude) -> bool {
    claude
        .pane
        .as_ref()
        .is_some_and(|p| Some(p.window_id) == model.snapshot.window)
}

/// ` main` after the name, cut to `room` cells; empty when there is no branch or no room for it.
fn branch(branch: &str, room: usize) -> String {
    if branch.is_empty() || room < MIN_BRANCH_WIDTH + 1 {
        return String::new();
    }
    format!(" {}", truncate(branch, room - 1))
}

/// The bar on the left edge for the window you are in.
fn marker(model: &Model, here: bool) -> Span<'static> {
    if here {
        Span::styled("▌", model.theme.accent)
    } else {
        Span::raw(" ")
    }
}

/// The status icon; a running session spins.
fn icon(model: &Model, status: Status) -> Span<'static> {
    let (icon, color) = look(&model.theme, status);
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

/// `window:directory` when the tmux session has several windows, otherwise just the directory.
fn name(claude: &Claude) -> String {
    let dir = std::path::Path::new(&claude.record.cwd)
        .file_name()
        .map_or_else(|| "?".to_owned(), |d| d.to_string_lossy().into_owned());
    claude
        .pane
        .as_ref()
        .filter(|p| p.session_windows > 1)
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
