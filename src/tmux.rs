//! Side effects on the tmux server. Everything else in the crate works on the values these return.

pub mod pane;

use std::process::{Command, Stdio};

use pane::{Pane, PaneId, WindowId};

/// Runs `tmux ARGS` and returns its stdout, or an empty string when tmux fails.
pub fn output(args: &[&str]) -> String {
    Command::new("tmux")
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default()
}

/// Runs `tmux ARGS`, reporting whether it succeeded.
pub fn run(args: &[&str]) -> bool {
    Command::new("tmux")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// A tmux format expanded against a target, trimmed.
pub fn display(target: &str, format: &str) -> String {
    output(&["display", "-p", "-t", target, format])
        .trim()
        .to_owned()
}

/// The current window of the client running this command.
pub fn current_window() -> Option<WindowId> {
    WindowId::parse(output(&["display", "-p", "#{window_id}"]).trim())
}

/// A global option, `None` when unset or empty.
pub fn global_option(name: &str) -> Option<String> {
    non_empty(&output(&["show", "-gqv", name]))
}

/// A window option, `None` when unset or empty.
pub fn window_option(window: WindowId, name: &str) -> Option<String> {
    non_empty(&output(&["show", "-wqv", "-t", &window.to_string(), name]))
}

fn non_empty(s: &str) -> Option<String> {
    Some(s.trim().to_owned()).filter(|s| !s.is_empty())
}

/// Every pane of the server.
pub fn panes() -> Vec<Pane> {
    pane::parse(&output(&["list-panes", "-a", "-F", pane::FORMAT]))
}

/// The panes of one window.
pub fn window_panes(window: WindowId) -> Vec<Pane> {
    pane::parse(&output(&[
        "list-panes",
        "-t",
        &window.to_string(),
        "-F",
        pane::FORMAT,
    ]))
}

/// Session names in tmux's own order.
pub fn sessions() -> Vec<String> {
    output(&["list-sessions", "-F", "#{session_name}"])
        .lines()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Brings `pane` to the front: its session on the client, its window, then the pane itself.
pub fn goto(pane: PaneId, client: Option<&str>) {
    let target = pane.to_string();
    let switch: Vec<&str> = ["switch-client"]
        .into_iter()
        .chain(client.into_iter().flat_map(|c| ["-c", c]))
        .chain(["-t", &target])
        .collect();
    run(&switch);
    run(&["select-window", "-t", &target]);
    run(&["select-pane", "-t", &target]);
}

/// Shows a message on the client (or the current one).
pub fn message(client: Option<&str>, text: &str) {
    let args: Vec<&str> = ["display-message"]
        .into_iter()
        .chain(client.into_iter().flat_map(|c| ["-c", c]))
        .chain([text])
        .collect();
    run(&args);
}
