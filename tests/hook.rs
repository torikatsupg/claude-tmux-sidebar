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
fn records_the_prompt_branch_and_context_usage() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let state_dir = dir.path().join("state");
    let transcript = dir.path().join("t.jsonl");
    std::fs::write(
        &transcript,
        r#"{"type":"assistant","gitBranch":"feat/x","requestedModel":"claude-opus-5-5[1m]","message":{"model":"claude-opus-5-5","usage":{"input_tokens":2,"cache_creation_input_tokens":8,"cache_read_input_tokens":249990}}}"#,
    )?;
    let payload = serde_json::json!({
        "hook_event_name": "UserPromptSubmit",
        "session_id": "ci",
        "prompt": "add a\ntest",
        "transcript_path": transcript,
    });
    hook(&state_dir, &payload.to_string())?;
    assert_eq!(field(&state_dir, "prompt").as_deref(), Some("add a test"));
    assert_eq!(field(&state_dir, "branch").as_deref(), Some("feat/x"));
    let bytes = std::fs::read(state_dir.join("ci.json"))?;
    let record: Value = serde_json::from_slice(&bytes)?;
    assert_eq!(record.get("context").and_then(Value::as_u64), Some(25));
    Ok(())
}
