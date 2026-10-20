//! Prompts answered before their notification arrives, and MCP elicitation.

use super::*;

const MCP_TOOL: &str = "mcp__jira__create_issue";

fn ask() -> Event<'static> {
    Event::PermissionRequest {
        tool: "Bash",
        agent: "",
    }
}

fn post() -> Event<'static> {
    Event::PostToolUse {
        tool: "Bash",
        agent: "",
    }
}

fn notify(kind: &'static str) -> Event<'static> {
    Event::Notification { kind, agent: "" }
}

#[test]
fn a_late_permission_prompt_keeps_an_answered_session_running() {
    let ran = run(&[ask(), post(), notify("permission_prompt")]);
    assert_eq!(
        (ran.state, ran.detail.as_str(), ran.answered),
        (Status::Running, "Bash", true)
    );
    assert_eq!(
        run(&[ask(), post(), DONE, notify("permission_prompt")]).state,
        Status::Done
    );
}

#[test]
fn a_new_prompt_is_shown_again() {
    let asked = run(&[ask(), post(), ask()]);
    assert_eq!((asked.state, asked.answered), (Status::Waiting, false));
    let next_turn = run(&[
        ask(),
        post(),
        DONE,
        Event::UserPromptSubmit,
        notify("permission_prompt"),
    ]);
    assert_eq!(next_turn.state, Status::Waiting);
    // A sandboxed command's network request raises no `PermissionRequest`.
    let next_tool = Event::PreToolUse {
        tool: "Bash",
        agent: "",
    };
    let sandboxed = run(&[ask(), post(), next_tool, notify("permission_prompt")]);
    assert_eq!(sandboxed.state, Status::Waiting);
}

#[test]
fn an_elicitation_waits_until_its_result() {
    let mcp = Event::PreToolUse {
        tool: MCP_TOOL,
        agent: "",
    };
    let elicit = || Event::Elicitation { agent: "" };
    let result = Event::ElicitationResult { agent: "" };
    let waiting = run(&[mcp, elicit()]);
    assert_eq!(
        (waiting.state, waiting.detail.as_str()),
        (Status::Waiting, MCP_TOOL)
    );
    let mcp = Event::PreToolUse {
        tool: MCP_TOOL,
        agent: "",
    };
    let answered = run(&[mcp, elicit(), result, notify("elicitation_dialog")]);
    assert_eq!(
        (answered.state, answered.detail.as_str()),
        (Status::Running, MCP_TOOL)
    );
}

#[test]
fn elicitation_notifications_wait() {
    for kind in ["elicitation_dialog", "elicitation_url_dialog"] {
        let waiting = run(&[Event::UserPromptSubmit, notify(kind)]);
        assert_eq!(waiting.state, Status::Waiting, "{kind}");
    }
}

#[test]
fn another_agents_elicitation_result_keeps_the_wait() {
    let waiting = run(&[
        Event::Elicitation { agent: "a1" },
        Event::ElicitationResult { agent: "a2" },
    ]);
    assert_eq!(
        (waiting.state, waiting.agent.as_str()),
        (Status::Waiting, "a1")
    );
    let ran = run(&[
        Event::Elicitation { agent: "a1" },
        Event::ElicitationResult { agent: "a1" },
    ]);
    assert_eq!(ran.state, Status::Running);
}
