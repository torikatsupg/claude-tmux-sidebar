//! `@cts_notify` from the hook's side: with it off, the hook never calls tmux.

use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

const WAIT: &str =
    r#"{"hook_event_name":"PermissionRequest","session_id":"ci","tool_name":"Bash"}"#;

/// Runs the hook in pane `%1` with a `tmux` on `PATH` that only logs its arguments to `tmux.log`.
fn hook_in_pane(dir: &Path, payload: &str) -> io::Result<()> {
    let fake = dir.join("tmux");
    let log = dir.join("tmux.log");
    fs::write(
        &fake,
        format!("#!/bin/sh\necho \"$@\" >>'{}'\n", log.display()),
    )?;
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755))?;
    let path = format!(
        "{}:{}",
        dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_claude-tmux-sidebar"))
        .arg("hook")
        .env("CTS_STATE_DIR", dir.join("state"))
        .env("TMUX", "/tmp/fake-tmux,1,0")
        .env("TMUX_PANE", "%1")
        .env("PATH", path)
        .stdin(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("no stdin"))?
        .write_all(payload.as_bytes())?;
    assert!(child.wait()?.success());
    Ok(())
}

#[test]
fn off_or_unset_never_calls_tmux() -> io::Result<()> {
    let tmp = tempfile::tempdir()?;
    hook_in_pane(tmp.path(), WAIT)?;
    hook_in_pane(
        tmp.path(),
        r#"{"hook_event_name":"Stop","session_id":"ci"}"#,
    )?;
    fs::write(tmp.path().join("state").join("notify"), "off\n")?;
    hook_in_pane(tmp.path(), WAIT)?;
    assert!(!tmp.path().join("tmux.log").exists());
    Ok(())
}

#[test]
fn on_asks_tmux_where_the_pane_is() -> io::Result<()> {
    let tmp = tempfile::tempdir()?;
    fs::create_dir_all(tmp.path().join("state"))?;
    fs::write(tmp.path().join("state").join("notify"), "tmux\n")?;
    hook_in_pane(tmp.path(), WAIT)?;
    let log = fs::read_to_string(tmp.path().join("tmux.log"))?;
    assert!(log.starts_with("display -p -t %1"), "{log}");
    Ok(())
}
