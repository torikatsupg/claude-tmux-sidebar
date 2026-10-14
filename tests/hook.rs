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
