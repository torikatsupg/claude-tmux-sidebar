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
        background: false,
        agent: String::new(),
    })
}

const DONE: Event<'static> = Event::Stop { busy: None };

/// Feeds `events` through the state machine from no record, returning the last record.
fn run(events: &[Event<'_>]) -> Record {
    events.iter().fold(Record::default(), |prev, event| {
        match next(event, Some(&prev)) {
            Outcome::Update(c) => Record {
                state: c.status,
                detail: c.detail,
                background: c.background,
                agent: c.agent,
                ..prev
            },
            _ => prev,
        }
    })
}

#[test]
fn tool_use_runs_and_questions_wait() {
    assert_eq!(
        next(
            &Event::PreToolUse {
                tool: "Bash",
                agent: ""
            },
            None
        ),
        update(Status::Running, "Bash")
    );
    assert_eq!(
        next(
            &Event::PreToolUse {
                tool: ASK_TOOL,
                agent: ""
            },
            None
        ),
        update(Status::Waiting, ASK_DETAIL)
    );
    assert_eq!(next(&DONE, None), update(Status::Done, ""));
}

#[test]
fn session_start_keeps_the_previous_status() {
    let start = Event::SessionStart { source: "clear" };
    assert_eq!(next(&start, None), update(Status::Idle, ""));
    assert_eq!(
        next(&start, Some(&prev(Status::Done, ""))),
        update(Status::Done, "")
    );
}

#[test]
fn permission_prompts_keep_the_tool_name() {
    let running = prev(Status::Running, "Edit");
    let outcome = next(
        &Event::Notification {
            kind: "permission_prompt",
            agent: "",
        },
        Some(&running),
    );
    assert_eq!(outcome, update(Status::Waiting, "Edit"));
}

#[test]
fn idle_prompt_only_ends_a_running_turn() {
    let idle = Event::Notification {
        kind: "idle_prompt",
        agent: "",
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
fn background_tasks_keep_a_stopped_session_running() {
    let busy = |kind| Event::Stop { busy: Some(kind) };
    let working = run(&[
        Event::UserPromptSubmit,
        busy("subagent"),
        Event::Notification {
            kind: "idle_prompt",
            agent: "",
        },
    ]);
    assert_eq!(
        (working.state, working.detail.as_str(), working.background),
        (Status::Running, "Agent", true)
    );
    // A subagent hands back, which wakes the main thread; its shell is still running.
    let shell = run(&[busy("subagent"), Event::UserPromptSubmit, busy("shell")]);
    assert_eq!(
        (shell.state, shell.detail.as_str()),
        (Status::Running, "Shell")
    );
    let done = run(&[busy("subagent"), Event::UserPromptSubmit, DONE]);
    assert_eq!((done.state, done.background), (Status::Done, false));
}

#[test]
fn a_new_process_has_no_background_tasks() {
    let record = run(&[
        Event::Stop {
            busy: Some("shell"),
        },
        Event::SessionStart { source: "resume" },
        Event::Notification {
            kind: "idle_prompt",
            agent: "",
        },
    ]);
    assert!(!record.background);
}

#[test]
fn unknown_events_are_ignored() {
    assert_eq!(
        next(
            &Event::Notification {
                kind: "other",
                agent: ""
            },
            None
        ),
        Outcome::Ignore
    );
    assert_eq!(next(&Event::Other, None), Outcome::Ignore);
    assert_eq!(next(&Event::SessionEnd, None), Outcome::Remove);
}

mod agents;
