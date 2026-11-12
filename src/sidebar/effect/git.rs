//! Reading a directory's branch and change summary with `git`, off the drawing loop.

use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};

use crate::sidebar::preview::{self, GitAsk, GitRead};
use crate::sidebar::update::Msg;

/// A `git` that takes longer is stopped; the preview then shows no branch or changes.
const TIMEOUT: Duration = Duration::from_secs(3);
const POLL: Duration = Duration::from_millis(10);

/// Starts the one thread that runs git, and returns where to send it requests. It reads only the newest
/// request waiting, so holding a movement key does not pile up git processes.
pub fn worker(answers: Sender<Msg>) -> Sender<GitAsk> {
    let (requests, waiting) = mpsc::channel::<GitAsk>();
    thread::spawn(move || {
        while let Ok(first) = waiting.recv() {
            let ask = newest(first, waiting.try_iter().collect());
            if answers.send(read(&ask)).is_err() {
                return;
            }
        }
    });
    requests
}

fn newest(first: GitAsk, rest: Vec<GitAsk>) -> GitAsk {
    rest.into_iter().last().unwrap_or(first)
}

fn read(ask: &GitAsk) -> Msg {
    let git = |dir: &str, command: &[&str]| run("git", &args(dir, command), TIMEOUT);
    let dir = ask.dir.as_str();
    let tops = [dir]
        .into_iter()
        .chain(ask.others.iter().map(String::as_str))
        .map(|d| {
            let top = git(d, &["rev-parse", "--show-toplevel"]).unwrap_or_default();
            (d.to_owned(), top)
        })
        .collect();
    Msg::Git(GitRead {
        tops,
        ..answer(
            dir,
            git(dir, &["rev-parse", "--abbrev-ref", "HEAD"]),
            git(dir, &["diff", "--shortstat", "HEAD"]).as_deref(),
        )
    })
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
fn answer(dir: &str, branch: Option<String>, stat: Option<&str>) -> GitRead {
    GitRead {
        dir: dir.to_owned(),
        branch: branch.unwrap_or_default(),
        changes: stat.and_then(preview::shortstat),
        tops: Vec::new(),
    }
}

#[cfg(test)]
mod tests;
