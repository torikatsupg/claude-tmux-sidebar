//! The work a session still runs: its subagents and background tasks, as panel sections.

use super::fit::Section;
use super::panel::{Group, Item};
use crate::session::Task;
use crate::text;

/// A subagent's type is padded to the longest of them, up to this many cells.
const MAX_NAME_WIDTH: usize = 12;

/// The subagents, then the other background tasks: a count each, over one row per task.
pub fn tasks(tasks: &[Task], now: u64) -> Vec<Section> {
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
        ..Section::default()
    }
}
