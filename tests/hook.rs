//! The `hook` command end to end: payloads on stdin, state files on disk.

use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::Value;

fn hook(state_dir: &Path, payload: &str) -> io::Result<()> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_claude-tmux-sidebar"))
        .arg("hook")
        .env("CTS_STATE_DIR", state_dir)
        .env_remove("TMUX")
        .env_remove("TMUX_PANE")
        .stdin(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("no stdin"))?
        .write_all(payload.as_bytes())?;
    let status = child.wait()?;
    assert!(status.success(), "the hook must never fail: {status}");
    Ok(())
}

/// A string field of the `ci` session's state file.
fn field(state_dir: &Path, name: &str) -> Option<String> {
    let bytes = std::fs::read(state_dir.join("ci.json")).ok()?;
    let record: Value = serde_json::from_slice(&bytes).ok()?;
    record.get(name)?.as_str().map(str::to_owned)
}

#[test]
fn records_a_session_from_start_to_end() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let state_dir = dir.path().join("state");
    hook(
        &state_dir,
        r#"{"hook_event_name":"PreToolUse","session_id":"ci","tool_name":"Bash","cwd":"/tmp"}"#,
    )?;
    assert_eq!(field(&state_dir, "state").as_deref(), Some("running"));
    assert_eq!(field(&state_dir, "detail").as_deref(), Some("Bash"));
    hook(
        &state_dir,
        r#"{"hook_event_name":"PreToolUse","session_id":"ci","tool_name":"AskUserQuestion"}"#,
    )?;
    assert_eq!(field(&state_dir, "state").as_deref(), Some("waiting"));
    assert_eq!(field(&state_dir, "detail").as_deref(), Some("Question"));
    hook(
        &state_dir,
        r#"{"hook_event_name":"Stop","session_id":"ci"}"#,
    )?;
    assert_eq!(field(&state_dir, "state").as_deref(), Some("done"));
    assert_eq!(field(&state_dir, "cwd").as_deref(), Some("/tmp"));
    hook(
        &state_dir,
        r#"{"hook_event_name":"SessionEnd","session_id":"ci"}"#,
    )?;
    assert!(!state_dir.join("ci.json").exists());
    Ok(())
}

#[test]
fn background_tasks_keep_a_stopped_session_running() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let state_dir = dir.path().join("state");
    hook(
        &state_dir,
        r#"{"hook_event_name":"Stop","session_id":"ci",
            "background_tasks":[{"id":"a1","type":"subagent","status":"running"}]}"#,
    )?;
    assert_eq!(field(&state_dir, "state").as_deref(), Some("running"));
    assert_eq!(field(&state_dir, "detail").as_deref(), Some("Agent"));
    hook(
        &state_dir,
        r#"{"hook_event_name":"Stop","session_id":"ci","background_tasks":[]}"#,
    )?;
    assert_eq!(field(&state_dir, "state").as_deref(), Some("done"));
    Ok(())
}

#[test]
fn ignores_garbage_and_unsafe_session_ids() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let state_dir = dir.path().join("state");
    hook(&state_dir, "not json")?;
    hook(
        &state_dir,
        r#"{"hook_event_name":"Stop","session_id":"../escape"}"#,
    )?;
    assert_eq!(std::fs::read_dir(&state_dir)?.count(), 0);
    assert!(!dir.path().join("escape.json").exists());
    Ok(())
}

#[test]
fn records_the_prompt_and_the_transcript_path() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let state_dir = dir.path().join("state");
    let payload = serde_json::json!({
        "hook_event_name": "UserPromptSubmit",
        "session_id": "ci",
        "prompt": "add a\ntest",
        "transcript_path": "/t/ci.jsonl",
    });
    hook(&state_dir, &payload.to_string())?;
    assert_eq!(field(&state_dir, "prompt").as_deref(), Some("add a test"));
    assert_eq!(
        field(&state_dir, "transcript").as_deref(),
        Some("/t/ci.jsonl")
    );
    Ok(())
}

#[test]
fn only_the_waiting_agent_ends_its_prompt() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let state_dir = dir.path().join("state");
    hook(
        &state_dir,
        r#"{"hook_event_name":"UserPromptSubmit","session_id":"ci","cwd":"/repo"}"#,
    )?;
    hook(
        &state_dir,
        r#"{"hook_event_name":"PermissionRequest","session_id":"ci","tool_name":"Edit",
            "agent_id":"a1","agent_type":"general-purpose","cwd":"/repo/wt"}"#,
    )?;
    hook(
        &state_dir,
        r#"{"hook_event_name":"PreToolUse","session_id":"ci","tool_name":"Bash",
            "agent_id":"a2","agent_type":"Explore","cwd":"/repo/wt2"}"#,
    )?;
    hook(
        &state_dir,
        r#"{"hook_event_name":"Stop","session_id":"ci","cwd":"/repo",
            "background_tasks":[{"id":"a1","type":"subagent","status":"running"}]}"#,
    )?;
    assert_eq!(field(&state_dir, "state").as_deref(), Some("waiting"));
    assert_eq!(field(&state_dir, "cwd").as_deref(), Some("/repo"));
    hook(
        &state_dir,
        r#"{"hook_event_name":"PostToolUse","session_id":"ci","tool_name":"Edit",
            "agent_id":"a1","agent_type":"general-purpose","cwd":"/repo/wt"}"#,
    )?;
    assert_eq!(field(&state_dir, "state").as_deref(), Some("running"));
    Ok(())
}

#[test]
fn a_turn_that_ends_without_stop_settles() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let state_dir = dir.path().join("state");
    let ask = r#"{"hook_event_name":"PermissionRequest","session_id":"ci","tool_name":"Bash"}"#;
    hook(&state_dir, ask)?;
    hook(
        &state_dir,
        r#"{"hook_event_name":"PostToolUseFailure","session_id":"ci","tool_name":"Bash",
            "error":"Exit code 1","is_interrupt":false}"#,
    )?;
    assert_eq!(field(&state_dir, "state").as_deref(), Some("running"));
    hook(
        &state_dir,
        r#"{"hook_event_name":"StopFailure","session_id":"ci","error":"rate_limit"}"#,
    )?;
    assert_eq!(field(&state_dir, "state").as_deref(), Some("done"));
    hook(&state_dir, ask)?;
    hook(
        &state_dir,
        r#"{"hook_event_name":"Notification","session_id":"ci","notification_type":"idle_prompt"}"#,
    )?;
    assert_eq!(field(&state_dir, "state").as_deref(), Some("idle"));
    Ok(())
}

#[test]
fn a_late_permission_prompt_and_mcp_elicitation() -> io::Result<()> {
    let tmp = tempfile::tempdir()?;
    let dir = tmp.path();
    let payloads = [
        r#"{"hook_event_name":"PermissionRequest","session_id":"ci","tool_name":"Bash"}"#,
        r#"{"hook_event_name":"PostToolUse","session_id":"ci","tool_name":"Bash"}"#,
        r#"{"hook_event_name":"Notification","session_id":"ci","notification_type":"permission_prompt"}"#,
        r#"{"hook_event_name":"Elicitation","session_id":"ci","mcp_server_name":"m","mode":"url"}"#,
        r#"{"hook_event_name":"ElicitationResult","session_id":"ci","mcp_server_name":"m","action":"accept"}"#,
    ];
    let wants = ["waiting", "running", "running", "waiting", "running"];
    for (p, want) in payloads.iter().zip(wants) {
        hook(dir, p)?;
        assert_eq!(field(dir, "state").as_deref(), Some(want), "{p}");
    }
    Ok(())
}
