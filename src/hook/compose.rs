//! The record a hook writes: the new status, plus what the session carries over from before.

use super::Origin;
use super::event::Payload;
use super::transition::Change;
use crate::session::Record;
use crate::text::truncate;

/// The prompt is kept only as long as a sidebar row could show it.
const PROMPT_WIDTH: usize = 200;

/// The new record: `since` only moves when the status changes.
pub fn compose(
    payload: &Payload,
    prev: Option<&Record>,
    change: Change,
    origin: &Origin,
) -> Record {
    let since = prev
        .filter(|r| r.state == change.status)
        .and_then(|r| r.since)
        .unwrap_or(origin.now);
    let cwd = Some(payload.cwd.as_str()).filter(|c| !c.is_empty());
    // A subagent may work elsewhere, such as in its own worktree.
    let cwd = cwd
        .filter(|_| payload.agent_id.is_empty())
        .or_else(|| prev.map(|r| r.cwd.as_str()).filter(|c| !c.is_empty()))
        .or(cwd)
        .unwrap_or_default();
    Record {
        session_id: payload.session_id.clone(),
        state: change.status,
        detail: change.detail,
        cwd: cwd.to_owned(),
        pane: origin.pane.clone(),
        socket: origin.socket.clone(),
        pid: origin.pid,
        since: Some(since),
        updated_at: origin.now,
        background: change.background,
        agent: change.agent,
        answered: change.answered,
        ..latest(payload, prev)
    }
}

/// The prompt and transcript path: new values when this event brings them, else the previous ones.
fn latest(payload: &Payload, prev: Option<&Record>) -> Record {
    Record {
        prompt: Some(headline(&payload.prompt))
            .filter(|p| !p.is_empty())
            .or_else(|| prev.map(|r| r.prompt.clone()))
            .unwrap_or_default(),
        transcript: Some(payload.transcript_path.clone())
            .filter(|t| !t.is_empty())
            .or_else(|| prev.map(|r| r.transcript.clone()))
            .unwrap_or_default(),
        ..Record::default()
    }
}

/// The prompt on one line, cut to what a row can show.
fn headline(prompt: &str) -> String {
    truncate(
        &prompt.split_whitespace().collect::<Vec<_>>().join(" "),
        PROMPT_WIDTH,
    )
}

#[cfg(test)]
mod tests;
