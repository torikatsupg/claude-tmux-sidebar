//! `hook`: records a Claude Code session's status from a hook payload on stdin.

pub mod event;
pub mod transition;

use std::io;

use event::{Event, Payload};
use transition::{Change, Outcome};

use crate::context::{self, Context};
use crate::process;
use crate::session::{Record, store, valid_session_id};

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
    let prev = store::read(&ctx.state_dir, &payload.session_id);
    match transition::next(&Event::of(&payload), prev.as_ref()) {
        Outcome::Ignore => Ok(()),
        Outcome::Remove => {
            store::remove(&store::path(&ctx.state_dir, &payload.session_id));
            Ok(())
        }
        Outcome::Update(change) => {
            let origin = Origin::current(ctx, prev.as_ref());
            store::write(
                &ctx.state_dir,
                &compose(&payload, prev.as_ref(), change, &origin),
            )
        }
    }
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

/// The new record: `since` only moves when the status changes.
pub fn compose(
    payload: &Payload,
    prev: Option<&Record>,
    change: Change,
    origin: &Origin,
) -> Record {
    let Change { status, detail } = change;
    let since = prev
        .filter(|r| r.state == status)
        .and_then(|r| r.since)
        .unwrap_or(origin.now);
    let cwd = Some(payload.cwd.as_str())
        .filter(|c| !c.is_empty())
        .or_else(|| prev.map(|r| r.cwd.as_str()))
        .unwrap_or_default();
    Record {
        session_id: payload.session_id.clone(),
        state: status,
        detail,
        cwd: cwd.to_owned(),
        pane: origin.pane.clone(),
        socket: origin.socket.clone(),
        pid: origin.pid,
        since: Some(since),
        updated_at: origin.now,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Status;

    fn change(status: Status) -> Change {
        Change {
            status,
            detail: String::new(),
        }
    }

    fn origin(now: u64) -> Origin {
        Origin {
            pane: "%1".to_owned(),
            socket: "s".to_owned(),
            pid: Some(9),
            now,
        }
    }

    #[test]
    fn since_moves_only_on_a_status_change() {
        let payload = Payload {
            session_id: "a".to_owned(),
            cwd: "/x".to_owned(),
            ..Payload::default()
        };
        let first = compose(&payload, None, change(Status::Running), &origin(10));
        let same = compose(&payload, Some(&first), change(Status::Running), &origin(20));
        let changed = compose(&payload, Some(&same), change(Status::Done), &origin(30));
        assert_eq!(
            (first.since, same.since, changed.since),
            (Some(10), Some(10), Some(30))
        );
        assert_eq!(same.updated_at, 20);
    }

    #[test]
    fn keeps_the_previous_cwd_when_the_payload_has_none() {
        let prev = Record {
            cwd: "/x".to_owned(),
            ..Record::default()
        };
        let record = compose(
            &Payload::default(),
            Some(&prev),
            change(Status::Done),
            &origin(0),
        );
        assert_eq!(record.cwd, "/x");
    }
}
