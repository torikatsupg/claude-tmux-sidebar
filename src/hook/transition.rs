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

/// The status a session moves to, and what it is busy with.
#[derive(Debug, PartialEq, Eq)]
pub struct Change {
    pub status: Status,
    pub detail: String,
}

/// The next status for `event`, given the previous record (if any).
pub fn next(event: &Event<'_>, prev: Option<&Record>) -> Outcome {
    let prev_status = prev.map(|r| r.state);
    match step(event, prev_status) {
        Step::Ignore => Outcome::Ignore,
        Step::Remove => Outcome::Remove,
        Step::To(status, detail) => Outcome::Update(Change {
            status,
            detail: carry_detail(status, detail, prev),
        }),
    }
}

enum Step<'a> {
    Ignore,
    Remove,
    To(Status, &'a str),
}

fn step<'a>(event: &Event<'a>, prev: Option<Status>) -> Step<'a> {
    match *event {
        Event::SessionStart => Step::To(prev.unwrap_or_default(), ""),
        Event::SessionEnd => Step::Remove,
        Event::UserPromptSubmit => Step::To(Status::Running, ""),
        Event::PreToolUse { tool } if tool == ASK_TOOL => Step::To(Status::Waiting, ASK_DETAIL),
        Event::PreToolUse { tool } | Event::PostToolUse { tool } => Step::To(Status::Running, tool),
        Event::PermissionRequest { tool } => Step::To(Status::Waiting, tool),
        Event::Notification { kind } => notification(kind, prev),
        Event::Stop => Step::To(Status::Done, ""),
        Event::Other => Step::Ignore,
    }
}

fn notification(kind: &str, prev: Option<Status>) -> Step<'static> {
    match kind {
        "permission_prompt" | "elicitation_dialog" => Step::To(Status::Waiting, ""),
        // Fires after ~60s of inactivity: also recovers from an interrupted (Esc) turn, which skips Stop.
        "idle_prompt" if prev == Some(Status::Running) => Step::To(Status::Idle, ""),
        "idle_prompt" => Step::To(prev.unwrap_or_default(), ""),
        _ => Step::Ignore,
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
mod tests {
    use super::*;

    fn prev(state: Status, detail: &str) -> Record {
        Record {
            state,
            detail: detail.to_owned(),
            ..Record::default()
        }
    }

    fn update(status: Status, detail: &str) -> Outcome {
        Outcome::Update(Change {
            status,
            detail: detail.to_owned(),
        })
    }

    #[test]
    fn tool_use_runs_and_questions_wait() {
        assert_eq!(
            next(&Event::PreToolUse { tool: "Bash" }, None),
            update(Status::Running, "Bash")
        );
        assert_eq!(
            next(&Event::PreToolUse { tool: ASK_TOOL }, None),
            update(Status::Waiting, ASK_DETAIL)
        );
        assert_eq!(next(&Event::Stop, None), update(Status::Done, ""));
    }

    #[test]
    fn session_start_keeps_the_previous_status() {
        assert_eq!(next(&Event::SessionStart, None), update(Status::Idle, ""));
        assert_eq!(
            next(&Event::SessionStart, Some(&prev(Status::Done, ""))),
            update(Status::Done, "")
        );
    }

    #[test]
    fn permission_prompts_keep_the_tool_name() {
        let running = prev(Status::Running, "Edit");
        let outcome = next(
            &Event::Notification {
                kind: "permission_prompt",
            },
            Some(&running),
        );
        assert_eq!(outcome, update(Status::Waiting, "Edit"));
    }

    #[test]
    fn idle_prompt_only_ends_a_running_turn() {
        let idle = Event::Notification {
            kind: "idle_prompt",
        };
        assert_eq!(
            next(&idle, Some(&prev(Status::Running, "Bash"))),
            update(Status::Idle, "")
        );
        assert_eq!(
            next(&idle, Some(&prev(Status::Done, ""))),
            update(Status::Done, "")
        );
    }

    #[test]
    fn unknown_events_are_ignored() {
        assert_eq!(
            next(&Event::Notification { kind: "other" }, None),
            Outcome::Ignore
        );
        assert_eq!(next(&Event::Other, None), Outcome::Ignore);
        assert_eq!(next(&Event::SessionEnd, None), Outcome::Remove);
    }
}
