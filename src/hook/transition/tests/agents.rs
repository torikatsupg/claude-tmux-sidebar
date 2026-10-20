//! Which agent a pending prompt belongs to.

use super::*;

fn pre(tool: &'static str, agent: &'static str) -> Event<'static> {
    Event::PreToolUse { tool, agent }
}

fn post(tool: &'static str, agent: &'static str) -> Event<'static> {
    Event::PostToolUse { tool, agent }
}

fn ask(tool: &'static str, agent: &'static str) -> Event<'static> {
    Event::PermissionRequest { tool, agent }
}

#[test]
fn another_agents_tool_events_keep_a_pending_prompt() {
    let waiting = run(&[pre("Edit", "a1"), ask("Edit", "a1"), pre("Bash", "a2")]);
    assert_eq!(
        (
            waiting.state,
            waiting.detail.as_str(),
            waiting.agent.as_str()
        ),
        (Status::Waiting, "Edit", "a1")
    );
    let waiting = run(&[ask("Edit", "a1"), post("Bash", "a2"), post("Read", "")]);
    assert_eq!(waiting.state, Status::Waiting);
    let main = run(&[ask("Bash", ""), pre("Read", "a1")]);
    assert_eq!(main.state, Status::Waiting);
}

#[test]
fn the_waiting_agents_own_tool_events_end_the_wait() {
    let ran = run(&[ask("Edit", "a1"), pre("Bash", "a2"), post("Edit", "a1")]);
    assert_eq!(
        (ran.state, ran.detail.as_str(), ran.agent.as_str()),
        (Status::Running, "Edit", "")
    );
    let asked = run(&[pre(ASK_TOOL, "a1"), post(ASK_TOOL, "")]);
    assert_eq!(asked.state, Status::Waiting);
    let answered = run(&[pre(ASK_TOOL, "a1"), post(ASK_TOOL, "a1")]);
    assert_eq!(answered.state, Status::Running);
}

#[test]
fn stop_keeps_a_background_subagents_prompt() {
    let busy = Event::Stop {
        busy: Some("subagent"),
    };
    let waiting = run(&[ask("Bash", "a1"), busy]);
    assert_eq!(
        (waiting.state, waiting.detail.as_str(), waiting.background),
        (Status::Waiting, "Bash", true)
    );
    let done = run(&[ask("Bash", "a1"), DONE]);
    assert_eq!(done.state, Status::Done);
}

#[test]
fn subagent_stop_ends_only_its_own_wait() {
    let stop = |agent| Event::SubagentStop { agent };
    let ended = run(&[ask("Bash", "a1"), stop("a1")]);
    assert_eq!(
        (ended.state, ended.detail.as_str()),
        (Status::Running, "Agent")
    );
    assert_eq!(run(&[ask("Bash", "a1"), stop("a2")]).state, Status::Waiting);
    assert_eq!(next(&stop("a1"), None), Outcome::Ignore);
}

#[test]
fn a_notification_keeps_the_agent_its_permission_request_named() {
    let prompt = Event::Notification {
        kind: "permission_prompt",
        agent: "",
    };
    let waiting = run(&[ask("Bash", "a1"), prompt]);
    assert_eq!(waiting.agent, "a1");
}
