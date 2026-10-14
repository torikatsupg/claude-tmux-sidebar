//! Questions about other processes: whether one is alive, and which Claude process started this hook.

use std::iter;
use std::process::{Command, Stdio};

use rustix::io::Errno;
use rustix::process::{Pid, test_kill_process};

/// How far up the process tree to look for Claude (hook runner, shells, then Claude itself).
const MAX_ANCESTORS: usize = 6;

/// Whether a process exists. A process owned by someone else counts as alive.
pub fn alive(pid: u32) -> bool {
    i32::try_from(pid)
        .ok()
        .and_then(Pid::from_raw)
        .is_none_or(|pid| test_kill_process(pid).err() != Some(Errno::SRCH))
}

/// The pid of the Claude Code process among this process's ancestors.
pub fn claude_ancestor() -> Option<u32> {
    iter::successors(Some(std::os::unix::process::parent_id()), |&pid| {
        parent_of(pid)
    })
    .take(MAX_ANCESTORS)
    .take_while(|&pid| pid > 1)
    .find(|&pid| is_claude(&ps(pid, "comm="), || ps(pid, "args=")))
}

/// Claude runs as `claude`, or as `node .../@anthropic-ai/claude-code/cli.js` when installed with npm.
pub fn is_claude(command: &str, args: impl FnOnce() -> String) -> bool {
    match command.rsplit('/').next() {
        Some("claude") => true,
        Some("node") => args().contains("claude"),
        _ => false,
    }
}

fn parent_of(pid: u32) -> Option<u32> {
    ps(pid, "ppid=").parse().ok()
}

fn ps(pid: u32, field: &str) -> String {
    Command::new("ps")
        .args(["-o", field, "-p", &pid.to_string()])
        .stderr(Stdio::null())
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_process_is_alive() {
        assert!(alive(std::process::id()));
    }

    #[test]
    fn recognises_claude_commands() {
        assert!(is_claude("/usr/local/bin/claude", String::new));
        assert!(is_claude("node", || {
            "node /x/@anthropic-ai/claude-code/cli.js".to_owned()
        }));
        assert!(!is_claude("node", || "node server.js".to_owned()));
        assert!(!is_claude("zsh", String::new));
    }
}
