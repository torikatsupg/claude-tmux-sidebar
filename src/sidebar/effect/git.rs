//! Reading a directory's branch and change summary with `git`, off the drawing loop.

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use crate::sidebar::preview::{self, GitRead};
use crate::sidebar::update::Msg;

/// A `git` that takes longer is stopped; the preview then shows no branch or changes.
const TIMEOUT: Duration = Duration::from_secs(3);
const POLL: Duration = Duration::from_millis(10);

/// Reads `dir` on its own thread and sends the answer.
pub fn spawn(dir: String, answers: Sender<Msg>) {
    thread::spawn(move || {
        let branch = run(
            "git",
            &args(&dir, &["rev-parse", "--abbrev-ref", "HEAD"]),
            TIMEOUT,
        );
        let stat = run(
            "git",
            &args(&dir, &["diff", "--shortstat", "HEAD"]),
            TIMEOUT,
        );
        let _ = answers.send(answer(&dir, branch, stat.as_deref()));
    });
}

/// `git` in `dir` without optional locks (it never blocks the user's own git), without the file system
/// monitor, and without diff drivers that would run other programs.
fn args(dir: &str, command: &[&str]) -> Vec<String> {
    let safe = [
        "--no-optional-locks",
        "-c",
        "core.fsmonitor=false",
        "-C",
        dir,
    ];
    let diff = ["--no-textconv", "--no-ext-diff"];
    let extra = if command.first() == Some(&"diff") {
        diff.as_slice()
    } else {
        &[]
    };
    safe.iter()
        .chain(command)
        .chain(extra)
        .map(|&a| a.to_owned())
        .collect()
}

/// The trimmed stdout of `program ARGS`; `None` when it fails or outlasts `timeout`.
fn run(program: &str, args: &[String], timeout: Duration) -> Option<String> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + timeout;
    let status = std::iter::repeat(())
        .take_while(|()| Instant::now() < deadline)
        .find_map(|()| finished(&mut child));
    let Some(status) = status else {
        let _ = child.kill();
        let _ = child.wait();
        return None;
    };
    let mut out = String::new();
    child.stdout.take()?.read_to_string(&mut out).ok()?;
    status.then(|| out.trim().to_owned())
}

/// Whether the child succeeded, once it has exited.
fn finished(child: &mut Child) -> Option<bool> {
    match child.try_wait() {
        Ok(Some(status)) => Some(status.success()),
        Ok(None) => {
            thread::sleep(POLL);
            None
        }
        Err(_) => Some(false),
    }
}

/// What git answered about `dir`; nothing outside a repository.
fn answer(dir: &str, branch: Option<String>, stat: Option<&str>) -> Msg {
    Msg::Git(GitRead {
        dir: dir.to_owned(),
        branch: branch.unwrap_or_default(),
        changes: stat.and_then(preview::shortstat),
    })
}

#[cfg(test)]
mod tests;
