//! `hook`: records a Claude Code session's status from a hook payload on stdin.

pub mod compose;
pub mod event;
pub mod transition;

use std::io;
use std::path::Path;

use compose::compose;
use event::{Event, Payload};
use transition::Outcome;

use crate::context::{self, Context};
use crate::process;
use crate::session::transcript::{self, Latest};
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
            let origin = Origin::current(ctx, prev.as_ref(), &payload);
            store::write(
                &ctx.state_dir,
                &compose(&payload, prev.as_ref(), change, &origin),
            )
        }
    }
}

/// Where and when the hook runs, and what the transcript says at that moment.
pub struct Origin {
    pub pane: String,
    pub socket: String,
    pub pid: Option<u32>,
    pub now: u64,
    pub transcript: Option<Latest>,
}

impl Origin {
    fn current(ctx: &Context, prev: Option<&Record>, payload: &Payload) -> Self {
        Self {
            pane: ctx.pane.map(|p| p.to_string()).unwrap_or_default(),
            socket: ctx.socket.clone(),
            // The Claude pid lets the sidebar drop sessions whose process died without SessionEnd.
            pid: prev.and_then(|r| r.pid).or_else(process::claude_ancestor),
            now: context::now(),
            transcript: Some(payload.transcript_path.as_str())
                .filter(|p| !p.is_empty())
                .and_then(|p| transcript::read(Path::new(p))),
        }
    }
}
