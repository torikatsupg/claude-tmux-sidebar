//! What the preview shows for the cursor row, before it is wrapped to the sidebar's width.

use super::text::plain;
use crate::session::Status;
use crate::session::live::Claude;
use crate::session::transcript::Ask;
use crate::sidebar::rows::{self, Row, RowKey};
use crate::sidebar::snapshot::Snapshot;
use crate::tmux::pane::Pane;

/// What the cursor row stands for.
#[derive(Clone, Copy, Debug)]
pub enum Subject<'a> {
    /// A Claude session, previewed from its transcript.
    Claude(&'a Claude),
    /// A window without Claude, previewed from its pane's screen.
    Window(&'a Pane),
}

/// The subject of the selected row; none for a tmux session's placeholder.
pub fn subject<'a>(snapshot: &'a Snapshot, selected: Option<&RowKey>) -> Option<Subject<'a>> {
    let selected = selected?;
    snapshot
        .rows
        .iter()
        .filter(|r| rows::key(r).as_ref() == Some(selected))
        .find_map(|row| match row {
            Row::Session(claude) => Some(Subject::Claude(claude)),
            Row::Window(pane) => Some(Subject::Window(pane)),
            _ => None,
        })
}

/// What a session asks for while it waits, or else Claude's last reply.
pub fn said(claude: &Claude) -> Vec<String> {
    let said = claude.latest.as_ref().map(|l| &l.said);
    let waiting = claude.record.state == Status::Waiting;
    match said.and_then(|s| s.ask.as_ref()).filter(|_| waiting) {
        Some(ask) => asked(ask),
        None => tidy(said.map_or("", |s| s.reply.as_str())),
    }
}

/// `Tool: detail`, or the question itself for `AskUserQuestion`.
fn asked(ask: &Ask) -> Vec<String> {
    let detail = tidy(&ask.detail);
    if ask.tool == "AskUserQuestion" && !detail.is_empty() {
        return detail;
    }
    let mut lines = detail.into_iter();
    let first = lines
        .next()
        .map_or_else(|| ask.tool.clone(), |d| format!("{}: {d}", ask.tool));
    [first].into_iter().chain(lines).collect()
}

/// Markdown made plain enough to read: no code fences or heading marks, and at most one blank line in a row.
pub fn tidy(text: &str) -> Vec<String> {
    let lines = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("```"))
        .map(|line| plain(&heading(line)).trim_end().to_owned())
        .fold(Vec::<String>::new(), |mut lines, line| {
            let repeated = line.is_empty() && lines.last().is_none_or(String::is_empty);
            if !repeated {
                lines.push(line);
            }
            lines
        });
    let used = lines
        .iter()
        .rposition(|l| !l.is_empty())
        .map_or(0, |i| i + 1);
    lines.into_iter().take(used).collect()
}

/// `## Plan` becomes `Plan`; `#tag` stays.
fn heading(line: &str) -> String {
    let rest = line.trim_start().trim_start_matches('#');
    match rest.strip_prefix(' ') {
        Some(title) if line.trim_start().starts_with('#') => title.trim_start().to_owned(),
        _ => line.to_owned(),
    }
}
