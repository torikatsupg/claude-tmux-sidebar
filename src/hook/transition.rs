//! The status state machine: how each hook event changes a session's status.

use super::event::Event;
use crate::session::{Record, Status};

/// `AskUserQuestion` blocks on the user like an approval prompt.
const ASK_TOOL: &str = "AskUserQuestion";
const ASK_DETAIL: &str = "Question";

/// What the hook does with the state file.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Ignore,
    Remove,
    Update(Change),
}

/// The status a session moves to, what it is busy with, whether background tasks keep it busy,
/// and which agent it waits on.
#[derive(Debug, PartialEq, Eq)]
pub struct Change {
    pub status: Status,
    pub detail: String,
    pub background: bool,
    pub agent: String,
}

/// The next status for `event`, given the previous record (if any).
pub fn next(event: &Event<'_>, prev: Option<&Record>) -> Outcome {
    let prev_status = prev.map(|r| r.state);
    let background = match *event {
        Event::Stop { busy } => busy.is_some(),
        // A new process has no background tasks yet.
        Event::SessionStart {
            source: "startup" | "resume",
        } => false,
        _ => prev.is_some_and(|r| r.background),
    };
    let step = pending(event, prev).unwrap_or_else(|| step(event, prev_status, background));
    match step {
        Step::Ignore => Outcome::Ignore,
        Step::Remove => Outcome::Remove,
        Step::To(status, detail) => Outcome::Update(Change {
            status,
            detail: carry_detail(status, detail, prev),
            background,
            agent: waiting_agent(status, event, prev),
        }),
    }
}

enum Step<'a> {
    Ignore,
    Remove,
    To(Status, &'a str),
}

fn step<'a>(event: &Event<'a>, prev: Option<Status>, background: bool) -> Step<'a> {
    match *event {
        Event::SessionStart { .. } => Step::To(prev.unwrap_or_default(), ""),
        Event::SessionEnd => Step::Remove,
        Event::UserPromptSubmit => Step::To(Status::Running, ""),
        Event::PreToolUse { tool, .. } if tool == ASK_TOOL => Step::To(Status::Waiting, ASK_DETAIL),
        Event::PreToolUse { tool, .. } | Event::PostToolUse { tool, .. } => {
            Step::To(Status::Running, tool)
        }
        Event::PermissionRequest { tool, .. } => Step::To(Status::Waiting, tool),
        Event::Notification { kind, .. } => notification(kind, prev, background),
        // Background tasks keep working after the turn; their completion wakes Claude, which stops again.
        Event::Stop { busy: Some(kind) } => Step::To(Status::Running, busy_detail(kind)),
        Event::Stop { busy: None } => Step::To(Status::Done, ""),
        Event::SubagentStop { .. } | Event::Other => Step::Ignore,
    }
}

/// While a prompt is pending, only the agent that raised it moves the session on.
fn pending<'a>(event: &Event<'a>, prev: Option<&Record>) -> Option<Step<'a>> {
    let owner = prev.filter(|r| r.state == Status::Waiting)?.agent.as_str();
    let keep = Some(Step::To(Status::Waiting, ""));
    match *event {
        Event::PreToolUse { tool, agent } if tool != ASK_TOOL && agent != owner => keep,
        Event::PostToolUse { agent, .. } if agent != owner => keep,
        // The main thread stops while a background subagent still waits for the user.
        Event::Stop { busy: Some(_) } if !owner.is_empty() => keep,
        // The waiting subagent ended without finishing its tool.
        Event::SubagentStop { agent } if !agent.is_empty() && agent == owner => {
            Some(Step::To(Status::Running, busy_detail("subagent")))
        }
        _ => None,
    }
}

/// The agent a waiting session waits on: the one raising a prompt now, else the one that already had.
fn waiting_agent(status: Status, event: &Event<'_>, prev: Option<&Record>) -> String {
    if status != Status::Waiting {
        return String::new();
    }
    let raised = match *event {
        Event::PermissionRequest { agent, .. } => Some(agent),
        Event::PreToolUse { tool, agent } if tool == ASK_TOOL => Some(agent),
        // A `Notification` may not name its agent; the `PermissionRequest` before it did.
        Event::Notification { agent, .. } if !agent.is_empty() => Some(agent),
        _ => None,
    };
    raised
        .or_else(|| {
            prev.filter(|r| r.state == Status::Waiting)
                .map(|r| r.agent.as_str())
        })
        .unwrap_or_default()
        .to_owned()
}

fn notification(kind: &str, prev: Option<Status>, background: bool) -> Step<'static> {
    match kind {
        "permission_prompt" | "elicitation_dialog" => Step::To(Status::Waiting, ""),
        // The main thread is quiet, but its background tasks are still working.
        "idle_prompt" if background => Step::Ignore,
        // Fires after ~60s of inactivity: also recovers from an interrupted (Esc) turn, which skips Stop.
        "idle_prompt" if prev == Some(Status::Running) => Step::To(Status::Idle, ""),
        "idle_prompt" => Step::To(prev.unwrap_or_default(), ""),
        _ => Step::Ignore,
    }
}

/// What a session whose turn ended is still busy with.
fn busy_detail(kind: &str) -> &'static str {
    match kind {
        "subagent" => "Agent",
        "shell" => "Shell",
        _ => "Task",
    }
}

/// Keeps the tool name visible while waiting for approval (`Notification` carries no tool name).
fn carry_detail(status: Status, detail: &str, prev: Option<&Record>) -> String {
    prev.filter(|_| status == Status::Waiting && detail.is_empty())
        .filter(|r| matches!(r.state, Status::Waiting | Status::Running))
        .map_or(detail, |r| r.detail.as_str())
        .to_owned()
}

#[cfg(test)]
mod tests;
