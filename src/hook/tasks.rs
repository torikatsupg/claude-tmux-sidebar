//! The subagents and background tasks a session is running, kept up to date from hook payloads.
//!
//! An `Agent` call on the main thread starts a foreground subagent under its tool call id; a background
//! launch moves it to its agent id. The call's return or `SubagentStop` ends it. A new prompt, `Stop` and
//! `StopFailure` end the turn and with it the foreground ones. `Stop` reports the session's background
//! tasks, which then replace the background ones.

use serde::Deserialize;

use super::event::{BackgroundTask, Payload};
use crate::session::Task;
use crate::text::{printable, truncate};

const AGENT_TOOL: &str = "Agent";
/// Texts are kept only as long as a preview line could show them.
const TEXT_WIDTH: usize = 200;

/// The parts of an `Agent` call's payload that the tasks need; read only for such calls.
#[derive(Default, Deserialize)]
struct AgentCall {
    #[serde(default)]
    tool_input: AgentInput,
    #[serde(default)]
    tool_response: AgentResponse,
}

#[derive(Default, Deserialize)]
struct AgentInput {
    #[serde(default)]
    description: String,
    #[serde(default)]
    subagent_type: String,
}

#[derive(Default, Deserialize)]
struct AgentResponse {
    #[serde(default)]
    status: String,
    #[serde(default, rename = "agentId")]
    agent_id: String,
}

/// The tasks after `payload` (`input` is its JSON), given the previous ones; `now` dates new tasks.
pub fn next(payload: &Payload, input: &str, prev: &[Task], now: u64) -> Vec<Task> {
    let agent_call = payload.agent_id.is_empty() && payload.tool_name == AGENT_TOOL;
    let call = || serde_json::from_str::<AgentCall>(input).unwrap_or_default();
    match payload.hook_event_name.as_str() {
        "PreToolUse" if agent_call => started(payload, &call(), prev, now),
        "PostToolUse" | "PostToolUseFailure" if agent_call => returned(payload, &call(), prev),
        "SubagentStop" => without(prev, &payload.agent_id),
        "Stop" => match &payload.background_tasks {
            Some(tasks) => reported(tasks, prev, now),
            None => background(prev),
        },
        "UserPromptSubmit" | "StopFailure" => background(prev),
        "SessionStart" if matches!(payload.source.as_str(), "startup" | "resume") => Vec::new(),
        _ => prev.to_vec(),
    }
}

fn started(payload: &Payload, call: &AgentCall, prev: &[Task], now: u64) -> Vec<Task> {
    let task = Task {
        id: payload.tool_use_id.clone(),
        kind: "subagent".to_owned(),
        agent: short(&call.tool_input.subagent_type),
        text: short(&call.tool_input.description),
        since: now,
        foreground: true,
    };
    without(prev, &payload.tool_use_id)
        .into_iter()
        .chain([task])
        .collect()
}

/// A background launch moves the subagent to its agent id; any other return ends it.
fn returned(payload: &Payload, call: &AgentCall, prev: &[Task]) -> Vec<Task> {
    let response = &call.tool_response;
    if response.status != "async_launched" || response.agent_id.is_empty() {
        return without(prev, &payload.tool_use_id);
    }
    prev.iter()
        .map(|t| {
            if t.id == payload.tool_use_id {
                Task {
                    id: response.agent_id.clone(),
                    foreground: false,
                    ..t.clone()
                }
            } else {
                t.clone()
            }
        })
        .collect()
}

fn without(prev: &[Task], id: &str) -> Vec<Task> {
    prev.iter().filter(|t| t.id != id).cloned().collect()
}

/// The tasks that outlive the turn.
fn background(prev: &[Task]) -> Vec<Task> {
    prev.iter().filter(|t| !t.foreground).cloned().collect()
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
            foreground: false,
        })
        .collect()
}

fn short(text: &str) -> String {
    truncate(&printable(text), TEXT_WIDTH)
}

#[cfg(test)]
mod tests;
