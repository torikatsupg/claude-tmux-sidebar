//! Which changes are notified, as hook events go through the state machine.

use super::*;
use crate::hook::event::Event;
use crate::hook::transition::{self, Outcome};
use crate::session::Record;

fn at(state: Status, updated_at: u64) -> Record {
    Record {
        state,
        updated_at,
        ..Record::default()
    }
}

#[test]
fn only_a_change_to_waiting_or_done_is_worth_a_notification() {
    let worth_ = |prev: Option<Status>, next: Status| {
        worth(prev.map(|p| at(p, 0)).as_ref(), &at(next, 100), false)
    };
    assert!(worth_(Some(Status::Running), Status::Waiting));
    assert!(worth_(Some(Status::Running), Status::Done));
    assert!(worth_(Some(Status::Waiting), Status::Done));
    assert!(worth_(None, Status::Waiting));
    assert!(!worth_(Some(Status::Waiting), Status::Waiting));
    assert!(!worth_(Some(Status::Done), Status::Done));
    assert!(!worth_(Some(Status::Waiting), Status::Running));
    assert!(!worth_(Some(Status::Done), Status::Idle));
}

/// The statuses notified while timed `events` go through the state machine, as the hook does.
fn notified(events: &[(u64, Event<'_>)]) -> Vec<Status> {
    let (_, sent) = events.iter().fold(
        (None::<Record>, Vec::new()),
        |(prev, sent), (now, event)| match transition::next(event, prev.as_ref()) {
            Outcome::Update(c) => {
                let record = Record {
                    state: c.status,
                    detail: c.detail,
                    background: c.background,
                    agent: c.agent,
                    answered: c.answered,
                    updated_at: *now,
                    ..prev.clone().unwrap_or_default()
                };
                let by_notification = matches!(event, Event::Notification { .. });
                let alert = worth(prev.as_ref(), &record, by_notification).then_some(c.status);
                (Some(record), [sent, alert.into_iter().collect()].concat())
            }
            Outcome::Ignore | Outcome::Remove => (prev, sent),
        },
    );
    sent
}

const PRE: Event<'static> = Event::PreToolUse {
    tool: "Bash",
    agent: "",
};
const ASK: Event<'static> = Event::PermissionRequest {
    tool: "Bash",
    agent: "",
};
const POST: Event<'static> = Event::PostToolUse {
    tool: "Bash",
    agent: "",
};
const PROMPT: Event<'static> = Event::Notification {
    kind: "permission_prompt",
    agent: "",
};
const STOP: Event<'static> = Event::Stop { busy: None };

#[test]
fn a_repeated_prompt_notification_is_not_notified_twice() {
    assert_eq!(
        notified(&[(0, Event::UserPromptSubmit), (1, ASK), (7, PROMPT)]),
        [Status::Waiting]
    );
    assert_eq!(
        notified(&[
            (0, ASK),
            (3, POST),
            (6, PROMPT),
            (8, STOP),
            (9, PROMPT),
            (9, STOP)
        ]),
        [Status::Waiting, Status::Done]
    );
}

#[test]
fn a_late_prompt_notification_after_the_next_tool_call_is_not_notified() {
    assert_eq!(
        notified(&[(0, PRE), (0, ASK), (5, POST), (5, PRE), (6, PROMPT)]),
        [Status::Waiting]
    );
    assert_eq!(
        notified(&[
            (0, ASK),
            (4, POST),
            (4, STOP),
            (5, Event::UserPromptSubmit),
            (6, PROMPT)
        ]),
        [Status::Waiting, Status::Done]
    );
}

#[test]
fn a_prompt_raised_without_permission_request_is_notified() {
    // A sandboxed command's network request: only the notification tells that it waits.
    assert_eq!(notified(&[(0, PRE), (6, PROMPT)]), [Status::Waiting]);
}
