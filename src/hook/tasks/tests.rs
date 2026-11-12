use super::*;

/// The tasks after the hook payload `json`.
fn step(json: &str, prev: &[Task], now: u64) -> Vec<Task> {
    let payload: Payload = serde_json::from_str(json).unwrap_or_default();
    next(&payload, json, prev, now)
}

/// `(id, kind)`, `(agent, text)` and `since`, running in the background.
fn task((id, kind): (&str, &str), (agent, text): (&str, &str), since: u64) -> Task {
    Task {
        id: id.to_owned(),
        kind: kind.to_owned(),
        agent: agent.to_owned(),
        text: text.to_owned(),
        since,
        foreground: false,
    }
}

/// A subagent the main thread waits on, under its tool call id.
fn foreground(id: &str, agent: &str, since: u64) -> Task {
    Task {
        foreground: true,
        ..task((id, "subagent"), (agent, "call"), since)
    }
}

const LAUNCH: &str = r#"{"hook_event_name":"PreToolUse","session_id":"s","tool_name":"Agent","tool_use_id":"t1",
    "tool_input":{"description":"Survey the docs","prompt":"…","subagent_type":"Explore"}}"#;

#[test]
fn an_agent_call_on_the_main_thread_starts_a_foreground_subagent() {
    let started = step(LAUNCH, &[], 100);
    assert_eq!(
        started,
        [Task {
            foreground: true,
            ..task(("t1", "subagent"), ("Explore", "Survey the docs"), 100)
        }]
    );
    let inside = r#"{"hook_event_name":"PreToolUse","session_id":"s","tool_name":"Agent","tool_use_id":"t2",
        "agent_id":"a1","tool_input":{"description":"nested"}}"#;
    assert_eq!(step(inside, &started, 120), started);
}

#[test]
fn a_finished_agent_call_ends_its_subagent() {
    let started = step(LAUNCH, &[], 100);
    let done = r#"{"hook_event_name":"PostToolUse","session_id":"s","tool_name":"Agent","tool_use_id":"t1",
        "tool_response":{"status":"completed","content":[]}}"#;
    assert!(step(done, &started, 160).is_empty());
}

#[test]
fn a_background_agent_goes_by_its_agent_id_until_it_stops() {
    let started = step(LAUNCH, &[], 100);
    let launched = r#"{"hook_event_name":"PostToolUse","session_id":"s","tool_name":"Agent","tool_use_id":"t1",
        "tool_response":{"status":"async_launched","agentId":"a9","description":"Survey the docs"}}"#;
    let running = step(launched, &started, 101);
    assert_eq!(
        running,
        [task(
            ("a9", "subagent"),
            ("Explore", "Survey the docs"),
            100
        )]
    );
    let stopped = r#"{"hook_event_name":"SubagentStop","session_id":"s","agent_id":"a9"}"#;
    assert!(step(stopped, &running, 200).is_empty());
}

const STOP: &str = r#"{"hook_event_name":"Stop","session_id":"s","background_tasks":[
    {"id":"a9","type":"subagent","status":"running","agent_type":"Explore","description":"Survey the docs"},
    {"id":"b1","type":"shell","status":"running","description":"build","command":"cargo build\n  --release"},
    {"id":"b2","type":"shell","status":"completed","command":"ls"},
    {"id":"m1","type":"monitor","status":"running","description":"watch CI","server":"gh"}]}"#;

#[test]
fn stop_replaces_the_list_with_the_running_background_tasks() {
    let prev = [
        task(("a9", "subagent"), ("Explore", "Survey the docs"), 100),
        foreground("t5", "Plan", 90),
    ];
    assert_eq!(
        step(STOP, &prev, 300),
        [
            task(("a9", "subagent"), ("Explore", "Survey the docs"), 100),
            task(("b1", "shell"), ("", "cargo build --release"), 300),
            task(("m1", "monitor"), ("", "watch CI"), 300),
        ]
    );
}

#[test]
fn a_subagent_stop_removes_only_its_own_entry() {
    let prev = [
        task(("a9", "subagent"), ("Explore", "x"), 100),
        foreground("t5", "Plan", 90),
        foreground("t6", "review", 95),
    ];
    let stop = STOP.replace("\"Stop\"", "\"SubagentStop\"").replace(
        "\"background_tasks\"",
        "\"agent_id\":\"a9\",\"background_tasks\"",
    );
    assert_eq!(
        step(&stop, &prev, 300),
        [foreground("t5", "Plan", 90), foreground("t6", "review", 95)]
    );
}

#[test]
fn a_new_prompt_or_a_failed_stop_drops_only_the_foreground_subagents() {
    let background = task(("a9", "subagent"), ("Explore", "x"), 100);
    let prev = [background.clone(), foreground("t5", "Plan", 90)];
    let prompt = r#"{"hook_event_name":"UserPromptSubmit","session_id":"s","prompt":"hi"}"#;
    assert_eq!(step(prompt, &prev, 300), std::slice::from_ref(&background));
    let failure = r#"{"hook_event_name":"StopFailure","session_id":"s"}"#;
    assert_eq!(step(failure, &prev, 300), std::slice::from_ref(&background));
    let old_stop = r#"{"hook_event_name":"Stop","session_id":"s"}"#;
    assert_eq!(step(old_stop, &prev, 300), [background]);
}

#[test]
fn a_new_process_starts_with_no_tasks() {
    let prev = [task(("a9", "subagent"), ("Explore", "x"), 100)];
    let start = r#"{"hook_event_name":"SessionStart","session_id":"s","source":"resume"}"#;
    assert!(step(start, &prev, 300).is_empty());
    let other = r#"{"hook_event_name":"PostToolUse","session_id":"s","tool_name":"Bash"}"#;
    assert_eq!(step(other, &prev, 300), prev);
}

#[test]
fn texts_lose_control_and_format_characters() {
    let launch = LAUNCH.replace(
        "Survey the docs",
        "Survey\\u202e the\\u0007 docs\\udb40\\udc41",
    );
    let started = step(&launch, &[], 100);
    assert_eq!(
        started.first().map(|t| t.text.as_str()),
        Some("Survey the docs")
    );
}
