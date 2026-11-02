//! The `statusline` command end to end: its input on stdin, the size file on disk.

use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};

fn run(state_dir: &Path, command: &str, input: &str) -> io::Result<String> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_claude-tmux-sidebar"))
        .arg(command)
        .env("CTS_STATE_DIR", state_dir)
        .env_remove("TMUX")
        .env_remove("TMUX_PANE")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("no stdin"))?
        .write_all(input.as_bytes())?;
    let output = child.wait_with_output()?;
    assert!(
        output.status.success(),
        "must never fail: {}",
        output.status
    );
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

const STATUS: &str = r#"{"session_id":"ci","context_window":{"context_window_size":1000000}}"#;
const START: &str = r#"{"hook_event_name":"SessionStart","session_id":"ci","cwd":"/x"}"#;
const END: &str = r#"{"hook_event_name":"SessionEnd","session_id":"ci"}"#;

#[test]
fn records_the_size_of_a_known_session_and_prints_nothing() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    run(dir.path(), "hook", START)?;
    assert_eq!(run(dir.path(), "statusline", STATUS)?, "");
    assert_eq!(
        std::fs::read_to_string(dir.path().join("ci.window"))?,
        "1000000"
    );
    Ok(())
}

#[test]
fn leaves_nothing_for_a_session_the_hooks_do_not_know() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    std::fs::create_dir_all(dir.path())?;
    run(dir.path(), "statusline", STATUS)?;
    assert!(!dir.path().join("ci.window").exists());
    Ok(())
}

#[test]
fn survives_garbage() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    assert_eq!(run(dir.path(), "statusline", "not json")?, "");
    Ok(())
}

#[test]
fn the_size_goes_with_the_session() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    run(dir.path(), "hook", START)?;
    run(dir.path(), "statusline", STATUS)?;
    run(dir.path(), "hook", END)?;
    assert!(!dir.path().join("ci.json").exists());
    assert!(!dir.path().join("ci.window").exists());
    Ok(())
}
