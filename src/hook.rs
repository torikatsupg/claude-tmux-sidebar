//! `hook`: records a Claude Code session's status from a hook payload on stdin.

pub mod compose;
pub mod event;
pub mod tasks;
pub mod transition;

use std::io;

use compose::compose;
use event::{Event, Payload};
use transition::Outcome;

use crate::context::{self, Context};
use crate::notify;
use crate::process;
use crate::session::{Record, Task, lock, store, valid_session_id};

const DEBUG_LOG: &str = "debug.log";

/// Handles one payload. A hook must never fail Claude, so errors are swallowed.
pub fn run(ctx: &Context, input: &str) {
    let _ = handle(ctx, input);
}

fn handle(ctx: &Context, input: &str) -> io::Result<()> {
    store::ensure_dir(&ctx.state_dir)?;
    if ctx.debug {
        store::append(&ctx.state_dir.join(DEBUG_LOG), input)?;
    }
    let payload: Payload = serde_json::from_str(input).unwrap_or_default();
    if !valid_session_id(&payload.session_id) {
        return Ok(());
    }
    let event = Event::of(&payload);
    let lock = lock::acquire(&ctx.state_dir, &payload.session_id);
    let written = record(ctx, &payload, input, &event)?;
    drop(lock);
    if let Some((prev, record)) = written {
        let by_notification = matches!(event, Event::Notification { .. });
        notify::send::run(ctx, prev.as_ref(), &record, by_notification);
    }
    Ok(())
}

/// Reads, updates and writes the state file under the session's lock; the records to notify about, if a
/// status was written.
fn record(
    ctx: &Context,
    payload: &Payload,
    input: &str,
    event: &Event<'_>,
) -> io::Result<Option<(Option<Record>, Record)>> {
    let prev = store::read(&ctx.state_dir, &payload.session_id);
    let now = context::now();
    let tasks = tasks::next(payload, input, prev.as_ref().map_or(&[], |r| &r.tasks), now);
    match transition::next(event, prev.as_ref()) {
        Outcome::Ignore => prev.map_or(Ok(None), |prev| retask(ctx, prev, tasks).map(|()| None)),
        Outcome::Remove => {
            store::remove_session(&store::path(&ctx.state_dir, &payload.session_id));
            Ok(None)
        }
        Outcome::Update(change) => {
            let origin = Origin::current(ctx, prev.as_ref());
            let record = Record {
                tasks,
                ..compose(payload, prev.as_ref(), change, &origin)
            };
            store::write(&ctx.state_dir, &record)?;
            Ok(Some((prev, record)))
        }
    }
}

/// An event that leaves the status alone may still end a subagent; only the tasks change then.
fn retask(ctx: &Context, prev: Record, tasks: Vec<Task>) -> io::Result<()> {
    if tasks == prev.tasks {
        return Ok(());
    }
    store::write(&ctx.state_dir, &Record { tasks, ..prev })
}

/// Where and when the hook runs.
pub struct Origin {
    pub pane: String,
    pub socket: String,
    pub pid: Option<u32>,
    pub now: u64,
}

impl Origin {
    fn current(ctx: &Context, prev: Option<&Record>) -> Self {
        Self {
            pane: ctx.pane.map(|p| p.to_string()).unwrap_or_default(),
            socket: ctx.socket.clone(),
            // The Claude pid lets the sidebar drop sessions whose process died without SessionEnd.
            pid: prev.and_then(|r| r.pid).or_else(process::claude_ancestor),
            now: context::now(),
        }
    }
}
