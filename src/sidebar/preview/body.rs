//! What the preview shows for the cursor row, before it is wrapped to the sidebar's width.

use crate::session::live::Claude;
use crate::sidebar::rows::{self, Row, RowKey};
use crate::sidebar::snapshot::Snapshot;
use crate::text::{printable, visible};
use crate::tmux::pane::Pane;

/// The hooks report `AskUserQuestion` as this detail.
const ASK_DETAIL: &str = "Question";
const ASK_TOOL: &str = "AskUserQuestion";

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

/// What the session is waiting on or working with, from the transcript: `Tool: detail` on one line, or
/// the question of `AskUserQuestion`. It is the first call without a result of the tool the hooks reported,
/// and nothing while a subagent asks, since its calls are not in the main transcript.
pub fn ask(claude: &Claude) -> Option<String> {
    let record = &claude.record;
    let tool = match record.detail.as_str() {
        ASK_DETAIL => ASK_TOOL,
        tool => tool,
    };
    let ask = claude
        .latest
        .as_ref()?
        .said
        .pending
        .iter()
        .filter(|_| record.agent.is_empty())
        .find(|a| a.tool == tool)?;
    let detail = printable(&ask.detail);
    Some(match (ask.tool.as_str(), detail.is_empty()) {
        (_, true) => printable(&ask.tool),
        (ASK_TOOL, false) => detail,
        (tool, false) => format!("{}: {detail}", printable(tool)),
    })
}

/// Claude's last reply, made plain.
pub fn reply(claude: &Claude) -> Vec<String> {
    tidy(claude.latest.as_ref().map_or("", |l| l.said.reply.as_str()))
}

/// Markdown made plain enough to read: no code fences or heading marks, and at most one blank line in a row.
pub fn tidy(text: &str) -> Vec<String> {
    let lines = text
        .lines()
        .filter(|line| !line.trim_start().starts_with("```"))
        .map(|line| visible(&heading(line)).trim_end().to_owned())
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
