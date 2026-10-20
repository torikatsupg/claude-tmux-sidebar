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
        ..latest(payload, prev, origin)
    }
}

/// The prompt, branch and context usage: new values when this event brings them, else the previous ones.
fn latest(payload: &Payload, prev: Option<&Record>, origin: &Origin) -> Record {
    let seen = origin.transcript.as_ref();
    Record {
        prompt: Some(headline(&payload.prompt))
            .filter(|p| !p.is_empty())
            .or_else(|| prev.map(|r| r.prompt.clone()))
            .unwrap_or_default(),
        branch: seen
            .map(|t| t.branch.clone())
            .filter(|b| !b.is_empty())
            .or_else(|| prev.map(|r| r.branch.clone()))
            .unwrap_or_default(),
        context: seen
            .map(|t| t.context)
            .or_else(|| prev.and_then(|r| r.context)),
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
mod tests {
    use super::*;
    use crate::session::Status;
    use crate::session::transcript::Latest;

    fn change(status: Status) -> Change {
        Change {
            status,
            detail: String::new(),
            background: false,
            agent: String::new(),
            answered: false,
        }
    }

    fn origin(now: u64) -> Origin {
        Origin {
            pane: "%1".to_owned(),
            socket: "s".to_owned(),
            pid: Some(9),
            now,
            transcript: None,
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

    #[test]
    fn a_subagent_does_not_move_the_cwd() {
        let prev = Record {
            cwd: "/x".to_owned(),
            ..Record::default()
        };
        let payload = Payload {
            cwd: "/x/worktree".to_owned(),
            agent_id: "a1".to_owned(),
            ..Payload::default()
        };
        let record = compose(&payload, Some(&prev), change(Status::Running), &origin(0));
        assert_eq!(record.cwd, "/x");
    }

    #[test]
    fn keeps_the_prompt_branch_and_context_until_new_ones_arrive() {
        let submit = Payload {
            session_id: "a".to_owned(),
            prompt: "fix\n  the   bug".to_owned(),
            ..Payload::default()
        };
        let seen = Origin {
            transcript: Some(Latest {
                branch: "main".to_owned(),
                context: 42,
            }),
            ..origin(0)
        };
        let first = compose(&submit, None, change(Status::Running), &seen);
        assert_eq!(
            (first.prompt.as_str(), first.branch.as_str(), first.context),
            ("fix the bug", "main", Some(42))
        );
        let later = compose(
            &Payload::default(),
            Some(&first),
            change(Status::Done),
            &origin(1),
        );
        assert_eq!(
            (later.prompt.as_str(), later.branch.as_str(), later.context),
            ("fix the bug", "main", Some(42))
        );
    }
}
