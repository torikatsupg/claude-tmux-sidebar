//! The `status` command end to end: state files on disk, the counts on stdout.

use std::io;
use std::path::Path;
use std::process::Command;

fn status(state_dir: &Path) -> io::Result<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_claude-tmux-sidebar"))
        .arg("status")
        .env("CTS_STATE_DIR", state_dir)
        .env_remove("TMUX")
        .env_remove("TMUX_PANE")
        .output()?;
    assert!(output.status.success(), "{}", output.status);
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn place(state_dir: &Path, id: &str, state: &str) -> io::Result<()> {
    let record = serde_json::json!({
        "session_id": id,
        "state": state,
        "pid": null,
        "updated_at": 1,
    });
    std::fs::write(state_dir.join(format!("{id}.json")), record.to_string())
}

#[test]
fn prints_the_count_of_each_status() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    let states = [
        ("a", "waiting"),
        ("b", "waiting"),
        ("c", "running"),
        ("d", "done"),
        ("e", "done"),
        ("f", "done"),
        ("g", "idle"),
    ];
    for (id, state) in states {
        place(dir.path(), id, state)?;
    }
    assert_eq!(status(dir.path())?, "◆2 ⠹1 ✓3");
    Ok(())
}

#[test]
fn leaves_out_empty_statuses_and_prints_nothing_without_sessions() -> io::Result<()> {
    let dir = tempfile::tempdir()?;
    assert_eq!(status(dir.path())?, "");
    place(dir.path(), "a", "done")?;
    place(dir.path(), "b", "idle")?;
    assert_eq!(status(dir.path())?, "✓1");
    Ok(())
}
