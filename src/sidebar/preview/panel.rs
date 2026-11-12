//! The status panel under the rule, by the selected session's status: what a waiting session asks for,
//! what a running one is doing, or how a finished one ended; then its tasks and the working tree's changes.

use super::body::{self, Subject};
use super::fit::{Section, fit};
use super::git::Changes;
use super::wrap::wrap;
use crate::session::live::Claude;
use crate::session::{Status, Task};
use crate::text;
use crate::tmux::pane::Pane;

/// A subagent's type is padded to the longest of them, up to this many cells.
const MAX_NAME_WIDTH: usize = 12;

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

/// The end of the last reply, then the tasks still running and the changes on the last line.
fn finished(claude: &Claude, context: &Context<'_>) -> Vec<Section> {
    let reply = Section {
        body: body::reply(claude)
            .iter()
            .flat_map(|line| wrap(line, context.width))
            .map(Item::Text)
            .collect(),
        tail: true,
        ..Section::default()
    };
    [reply]
        .into_iter()
        .chain(tasks(&claude.record.tasks, context.now))
        .chain(changes(context))
        .collect()
}

fn window(pane: &Pane, context: &Context<'_>) -> Vec<Section> {
    let line = |item: Item| Section {
        head: vec![item],
        ..Section::default()
    };
    let texts = [
        Item::Text(pane.command.clone()),
        Item::Muted(pane.path.clone()),
        Item::Muted(context.branch.to_owned()),
    ];
    texts
        .into_iter()
        .filter(|item| !matches!(item, Item::Text(t) | Item::Muted(t) if t.is_empty()))
        .map(line)
        .chain(changes(context))
        .collect()
}

/// The subagents, then the other background tasks: a count each, over one row per task.
fn tasks(tasks: &[Task], now: u64) -> Vec<Section> {
    let (agents, others): (Vec<&Task>, Vec<&Task>) =
        tasks.iter().partition(|t| t.kind == "subagent");
    [(Group::Agents, agents), (Group::Background, others)]
        .into_iter()
        .filter(|(_, list)| !list.is_empty())
        .map(|(group, list)| group_section(group, &list, now))
        .collect()
}

/// A count over one row per task: a subagent's type, or a task's kind unless it is a shell.
fn group_section(group: Group, list: &[&Task], now: u64) -> Section {
    let name = |t: &Task| match group {
        Group::Agents => t.agent.clone(),
        Group::Background if t.kind == "shell" => String::new(),
        Group::Background => t.kind.clone(),
    };
    let name_width = list
        .iter()
        .map(|t| text::width(&name(t)))
        .max()
        .unwrap_or(0)
        .min(MAX_NAME_WIDTH);
    let row = |t: &&Task| Item::Task {
        name: name(t),
        name_width,
        text: text::printable(&t.text),
        age: now.saturating_sub(t.since),
    };
    Section {
        head: vec![Item::Count {
            group,
            count: list.len(),
        }],
        body: list.iter().map(row).collect(),
        tail: false,
    }
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
