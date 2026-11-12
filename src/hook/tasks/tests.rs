use super::*;

fn payload(json: &str) -> Payload {
    serde_json::from_str(json).unwrap_or_default()
}

/// `(id, kind)`, `(agent, text)` and `since`.
fn task((id, kind): (&str, &str), (agent, text): (&str, &str), since: u64) -> Task {
    Task {
        id: id.to_owned(),
        kind: kind.to_owned(),
        agent: agent.to_owned(),
        text: text.to_owned(),
        since,
    }
}

const LAUNCH: &str = r#"{"hook_event_name":"PreToolUse","session_id":"s","tool_name":"Agent","tool_use_id":"t1",
    "tool_input":{"description":"Survey the docs","prompt":"…","subagent_type":"Explore"}}"#;

#[test]
fn an_agent_call_on_the_main_thread_starts_a_subagent() {
    let started = next(&payload(LAUNCH), &[], 100);
    assert_eq!(
        started,
        [task(
            ("t1", "subagent"),
            ("Explore", "Survey the docs"),
            100
        )]
    );
    let inside = payload(
        r#"{"hook_event_name":"PreToolUse","session_id":"s","tool_name":"Agent","tool_use_id":"t2",
            "agent_id":"a1","tool_input":{"description":"nested"}}"#,
    );
    assert_eq!(next(&inside, &started, 120), started);
}

#[test]
fn a_finished_agent_call_ends_its_subagent() {
    let started = next(&payload(LAUNCH), &[], 100);
    let done = payload(
        r#"{"hook_event_name":"PostToolUse","session_id":"s","tool_name":"Agent","tool_use_id":"t1",
            "tool_response":{"status":"completed","content":[]}}"#,
    );
    assert!(next(&done, &started, 160).is_empty());
}

#[test]
fn a_background_agent_goes_by_its_agent_id_until_it_stops() {
    let started = next(&payload(LAUNCH), &[], 100);
    let launched = payload(
        r#"{"hook_event_name":"PostToolUse","session_id":"s","tool_name":"Agent","tool_use_id":"t1",
            "tool_response":{"status":"async_launched","agentId":"a9","description":"Survey the docs"}}"#,
    );
    let running = next(&launched, &started, 101);
    assert_eq!(
        running,
        [task(
            ("a9", "subagent"),
            ("Explore", "Survey the docs"),
            100
        )]
    );
    let stopped = payload(r#"{"hook_event_name":"SubagentStop","session_id":"s","agent_id":"a9"}"#);
    assert!(next(&stopped, &running, 200).is_empty());
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
        task(("t5", "subagent"), ("Plan", "lost foreground call"), 90),
    ];
    assert_eq!(
        next(&payload(STOP), &prev, 300),
        [
            task(("a9", "subagent"), ("Explore", "Survey the docs"), 100),
            task(("b1", "shell"), ("", "cargo build --release"), 300),
            task(("m1", "monitor"), ("", "watch CI"), 300),
        ]
    );
}

#[test]
fn a_subagent_stop_with_the_session_tasks_refreshes_the_list_too() {
    let prev = [task(("a9", "subagent"), ("Explore", "x"), 100)];
    let stop = STOP.replace("\"Stop\"", "\"SubagentStop\"").replace(
        "\"background_tasks\"",
        "\"agent_id\":\"a9\",\"background_tasks\"",
    );
    assert_eq!(next(&payload(&stop), &prev, 300).len(), 3);
}

#[test]
fn a_stop_without_tasks_or_a_new_process_clears_the_list() {
    let prev = [task(("a9", "subagent"), ("Explore", "x"), 100)];
    let bare = payload(r#"{"hook_event_name":"StopFailure","session_id":"s"}"#);
    assert!(next(&bare, &prev, 300).is_empty());
    let start = payload(r#"{"hook_event_name":"SessionStart","session_id":"s","source":"resume"}"#);
    assert!(next(&start, &prev, 300).is_empty());
    let other = payload(r#"{"hook_event_name":"UserPromptSubmit","session_id":"s","prompt":"hi"}"#);
    assert_eq!(next(&other, &prev, 300), prev);
}

#[test]
fn texts_lose_control_and_format_characters() {
    let launch = LAUNCH.replace("Survey the docs", "Survey\\u202e the\\u0007 docs");
    let started = next(&payload(&launch), &[], 100);
    assert_eq!(
        started.first().map(|t| t.text.as_str()),
        Some("Survey the docs")
    );
}
