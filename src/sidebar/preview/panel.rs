//! The status panel under the rule, by the selected session's status: what a waiting session asks for,
//! what a running one is doing, or how a finished one ended; then its tasks and the working tree's changes.

use super::body::{self, Subject};
use super::fit::{Section, fit};
use super::git::Changes;
use super::work::tasks;
use super::wrap::wrap;
use crate::session::live::Claude;
use crate::session::{Status, Task};
use crate::text;
use crate::tmux::pane::Pane;

/// A finished session's reply keeps at least this many lines, before the counts and the changes.
const MIN_REPLY_LINES: usize = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    /// The status icon and what the session does; empty text names the status.
    Lead {
        status: Status,
        text: String,
    },
    /// A wrapped line under the lead.
    More(String),
    Count {
        group: Group,
        count: usize,
    },
    Task {
        name: String,
        name_width: usize,
        text: String,
        age: u64,
    },
    /// `shared`: another session works in the same directory.
    Changes {
        changes: Changes,
        shared: bool,
    },
    Text(String),
    Muted(String),
    /// A directory, cut from its start so that its end shows.
    Path(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Agents,
    Background,
}

/// What the panel needs besides the subject.
pub struct Context<'a> {
    pub changes: Option<Changes>,
    pub branch: &'a str,
    pub shared: bool,
    pub now: u64,
    pub width: usize,
    /// `$HOME`, shown as `~`.
    pub home: &'a str,
}

/// The panel's lines, at most `room`.
pub fn items(subject: Subject<'_>, context: &Context<'_>, room: usize) -> Vec<Item> {
    let sections = match subject {
        Subject::Claude(claude) => match claude.record.state {
            Status::Waiting => waiting(claude, context, room),
            Status::Running => running(claude, context),
            Status::Done | Status::Idle => finished(claude, context),
        },
        Subject::Window(pane) => window(pane, context),
    };
    fit(&sections, room)
}

fn waiting(claude: &Claude, context: &Context<'_>, room: usize) -> Vec<Section> {
    let ask = body::ask(claude).unwrap_or_else(|| claude.record.detail.clone());
    let lines = wrap(&ask, context.width.saturating_sub(2));
    let mut lines = lines.into_iter().take(room.max(1));
    let lead = Item::Lead {
        status: Status::Waiting,
        text: lines.next().unwrap_or_default(),
    };
    let ask = Section {
        head: [lead].into_iter().chain(lines.map(Item::More)).collect(),
        ..Section::default()
    };
    [ask]
        .into_iter()
        .chain(tasks(&claude.record.tasks, context.now))
        .chain(changes(context))
        .collect()
}

fn running(claude: &Claude, context: &Context<'_>) -> Vec<Section> {
    let lead = Item::Lead {
        status: Status::Running,
        text: body::ask(claude).unwrap_or_else(|| claude.record.detail.clone()),
    };
    let lead = Section {
        head: vec![lead],
        ..Section::default()
    };
    [lead]
        .into_iter()
        .chain(tasks(&claude.record.tasks, context.now))
        .collect()
}

/// The end of the last reply, then the work still running and the changes on the last line. Monitors are
/// left out: they wait for events, and some never end.
fn finished(claude: &Claude, context: &Context<'_>) -> Vec<Section> {
    let reply = Section {
        body: body::reply(claude)
            .iter()
            .flat_map(|line| wrap(line, context.width))
            .map(Item::Text)
            .collect(),
        tail: true,
        min_body: MIN_REPLY_LINES,
        ..Section::default()
    };
    let working: Vec<Task> = claude
        .record
        .tasks
        .iter()
        .filter(|t| t.kind != "monitor")
        .cloned()
        .collect();
    let changes = changes(context).map(|c| Section { first: true, ..c });
    [reply]
        .into_iter()
        .chain(tasks(&working, context.now))
        .chain(changes)
        .collect()
}

fn window(pane: &Pane, context: &Context<'_>) -> Vec<Section> {
    let line = |item: Item| Section {
        head: vec![item],
        ..Section::default()
    };
    let texts = [
        Item::Text(text::printable(&pane.command)),
        Item::Path(home_relative(&text::printable(&pane.path), context.home)),
        Item::Muted(text::printable(context.branch)),
    ];
    texts
        .into_iter()
        .filter(
            |item| !matches!(item, Item::Text(t) | Item::Muted(t) | Item::Path(t) if t.is_empty()),
        )
        .map(line)
        .chain(changes(context))
        .collect()
}

/// `path` with `home` written as `~`.
fn home_relative(path: &str, home: &str) -> String {
    let rest = Some(home)
        .filter(|h| !h.is_empty())
        .and_then(|h| path.strip_prefix(h))
        .filter(|rest| rest.is_empty() || rest.starts_with('/'));
    rest.map_or_else(|| path.to_owned(), |rest| format!("~{rest}"))
}

fn changes(context: &Context<'_>) -> Option<Section> {
    context.changes.map(|changes| Section {
        head: vec![Item::Changes {
            changes,
            shared: context.shared,
        }],
        ..Section::default()
    })
}
