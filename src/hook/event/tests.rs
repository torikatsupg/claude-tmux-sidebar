use super::*;

#[test]
fn reads_the_event_and_its_data() {
    let payload: Payload = serde_json::from_str(
        r#"{"hook_event_name":"PreToolUse","session_id":"s","tool_name":"Bash","extra":1}"#,
    )
    .unwrap_or_default();
    assert_eq!(
        Event::of(&payload),
        Event::PreToolUse {
            tool: "Bash",
            agent: ""
        }
    );
}

#[test]
fn reads_the_subagent_a_hook_fired_in() {
    let payload: Payload = serde_json::from_str(
        r#"{"hook_event_name":"PostToolUseFailure","session_id":"s","tool_name":"Bash",
            "agent_id":"a1","agent_type":"Explore"}"#,
    )
    .unwrap_or_default();
    assert_eq!(
        Event::of(&payload),
        Event::PostToolUse {
            tool: "Bash",
            agent: "a1"
        }
    );
}

#[test]
fn stop_reads_the_running_background_task() {
    let payload: Payload = serde_json::from_str(
        r#"{"hook_event_name":"Stop","session_id":"s","background_tasks":[
            {"id":"a","type":"subagent","status":"completed"},
            {"id":"b","type":"shell","status":"running","command":"sleep 40"}]}"#,
    )
    .unwrap_or_default();
    assert_eq!(
        Event::of(&payload),
        Event::Stop {
            busy: Some("shell")
        }
    );
}

#[test]
fn stop_failure_ends_the_turn() {
    let payload: Payload = serde_json::from_str(
        r#"{"hook_event_name":"StopFailure","session_id":"s","error":"rate_limit"}"#,
    )
    .unwrap_or_default();
    assert_eq!(Event::of(&payload), Event::Stop { busy: None });
}

#[test]
fn stop_does_not_count_a_running_monitor() {
    let payload: Payload = serde_json::from_str(
        r#"{"hook_event_name":"Stop","session_id":"s","background_tasks":[
            {"id":"a","type":"monitor","status":"running"}]}"#,
    )
    .unwrap_or_default();
    assert_eq!(Event::of(&payload), Event::Stop { busy: None });
}
