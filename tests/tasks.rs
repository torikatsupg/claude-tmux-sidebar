//! The subagents and background tasks the `hook` command records.

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

/// The `tasks` of the `ci` session's state file: `(id, kind)` pairs.
fn tasks(state_dir: &Path) -> Vec<(String, String)> {
    let record: Value = std::fs::read(state_dir.join("ci.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default();
    let pick = |t: &Value, name: &str| {
        t.get(name)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    record
        .get("tasks")
        .and_then(Value::as_array)
        .map(|list| {
            list.iter()
                .map(|t| (pick(t, "id"), pick(t, "kind")))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn records_the_subagents_until_they_stop() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let state_dir = dir.path().join("state");
    hook(
        &state_dir,
        r#"{"hook_event_name":"PreToolUse","session_id":"ci","tool_name":"Agent","tool_use_id":"t1",
            "tool_input":{"description":"Survey","subagent_type":"Explore"}}"#,
    )?;
    hook(
        &state_dir,
        r#"{"hook_event_name":"PostToolUse","session_id":"ci","tool_name":"Agent","tool_use_id":"t1",
            "tool_response":{"status":"async_launched","agentId":"a9"}}"#,
    )?;
    assert_eq!(
        tasks(&state_dir),
        [("a9".to_owned(), "subagent".to_owned())]
    );
    hook(
        &state_dir,
        r#"{"hook_event_name":"SubagentStop","session_id":"ci","agent_id":"a9"}"#,
    )?;
    assert!(tasks(&state_dir).is_empty());
    assert_eq!(field(&state_dir, "state").as_deref(), Some("running"));
    Ok(())
}
