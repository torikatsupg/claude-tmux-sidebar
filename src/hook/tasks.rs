//! The subagents and background tasks a session is running, kept up to date from hook payloads.
//!
//! An `Agent` call on the main thread starts a subagent, keyed by its tool call; a background launch rekeys
//! it to its agent id. `SubagentStop` and a finished call end it. `Stop` (and `SubagentStop`) report the
//! session's background tasks, which replace the list, so an entry whose end was missed does not stay.

use serde_json::Value;

use super::event::{BackgroundTask, Payload};
use crate::session::Task;
use crate::text::{printable, truncate};

const AGENT_TOOL: &str = "Agent";
/// Texts are kept only as long as a preview line could show them.
const TEXT_WIDTH: usize = 200;

/// The tasks after `payload`, given the previous ones; `now` dates the tasks it starts.
pub fn next(payload: &Payload, prev: &[Task], now: u64) -> Vec<Task> {
    let main = payload.agent_id.is_empty();
    let agent_call = main && payload.tool_name == AGENT_TOOL;
    match payload.hook_event_name.as_str() {
        "PreToolUse" if agent_call => started(payload, prev, now),
        "PostToolUse" | "PostToolUseFailure" if agent_call => returned(payload, prev),
        "SubagentStop" => match &payload.background_tasks {
            Some(tasks) => reported(tasks, prev, now),
            None => without(prev, &payload.agent_id),
        },
        "Stop" | "StopFailure" => reported(
            payload.background_tasks.as_deref().unwrap_or_default(),
            prev,
            now,
        ),
        "SessionStart" if matches!(payload.source.as_str(), "startup" | "resume") => Vec::new(),
        _ => prev.to_vec(),
    }
}

fn started(payload: &Payload, prev: &[Task], now: u64) -> Vec<Task> {
    let input = |name: &str| {
        payload
            .tool_input
            .get(name)
            .and_then(Value::as_str)
            .unwrap_or_default()
    };
    let task = Task {
        id: payload.tool_use_id.clone(),
        kind: "subagent".to_owned(),
        agent: short(input("subagent_type")),
        text: short(input("description")),
        since: now,
    };
    without(prev, &payload.tool_use_id)
        .into_iter()
        .chain([task])
        .collect()
}

/// A background launch keeps the subagent under its agent id; any other return ends it.
fn returned(payload: &Payload, prev: &[Task]) -> Vec<Task> {
    let response = |name: &str| payload.tool_response.get(name).and_then(Value::as_str);
    let agent_id = response("agentId").filter(|_| response("status") == Some("async_launched"));
    match agent_id {
        Some(agent_id) => prev
            .iter()
            .map(|t| {
                let id = if t.id == payload.tool_use_id {
                    agent_id
                } else {
                    &t.id
                };
                Task {
                    id: id.to_owned(),
                    ..t.clone()
                }
            })
            .collect(),
        None => without(prev, &payload.tool_use_id),
    }
}

fn without(prev: &[Task], id: &str) -> Vec<Task> {
    prev.iter().filter(|t| t.id != id).cloned().collect()
}

/// The running tasks Claude Code reports, keeping the start of the ones already known.
fn reported(tasks: &[BackgroundTask], prev: &[Task], now: u64) -> Vec<Task> {
    tasks
        .iter()
        .filter(|t| t.status == "running")
        .map(|t| Task {
            id: t.id.clone(),
            kind: t.kind.clone(),
            agent: short(&t.agent_type),
            text: short(
                Some(&t.command)
                    .filter(|c| !c.is_empty())
                    .unwrap_or(&t.description),
            ),
            since: prev.iter().find(|p| p.id == t.id).map_or(now, |p| p.since),
        })
        .collect()
}

fn short(text: &str) -> String {
    truncate(&printable(text), TEXT_WIDTH)
}

#[cfg(test)]
mod tests;
