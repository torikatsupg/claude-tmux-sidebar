//! Claude Code sessions: the state file each hook writes, and the live view the sidebar builds from them.

pub mod live;
pub mod store;
pub mod transcript;

use serde::{Deserialize, Serialize};

use crate::tmux::pane::PaneId;

/// What a Claude session is doing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Waiting,
    Running,
    Done,
    #[default]
    Idle,
}

impl Status {
    /// Statuses that `next` / `prev` visit: the ones asking for the user.
    pub fn needs_attention(self) -> bool {
        matches!(self, Self::Waiting | Self::Done)
    }
}

/// One state file. Field names are the on-disk format shared with older versions.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub session_id: String,
    #[serde(default)]
    pub state: Status,
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub pane: String,
    #[serde(default)]
    pub socket: String,
    #[serde(default)]
    pub pid: Option<u32>,
    #[serde(default)]
    pub since: Option<u64>,
    #[serde(default)]
    pub updated_at: u64,
    /// The head of the last prompt the user sent.
    #[serde(default)]
    pub prompt: String,
    /// The session's transcript, where the sidebar reads the branch and context usage.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub transcript: String,
    /// Whether background tasks were still running when the main thread last stopped.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub background: bool,
    /// While waiting: the subagent whose prompt is pending, or empty for the main thread.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub agent: String,
    /// The last prompt was answered: a notification for it that arrives late is ignored.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub answered: bool,
    /// The branch a hook before 0.7.4 wrote; used only while there is no `transcript`.
    #[serde(default, skip_serializing)]
    pub branch: String,
    /// The context usage a hook before 0.7.4 wrote; used only while there is no `transcript`.
    #[serde(default, skip_serializing)]
    pub context: Option<u8>,
}

impl Record {
    /// The tmux pane Claude runs in; `None` outside tmux.
    pub fn pane_id(&self) -> Option<PaneId> {
        PaneId::parse(&self.pane)
    }
}

/// Session ids become file names, so only plain ids are accepted.
pub fn valid_session_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_shared_format() {
        let json = r#"{"session_id":"a","state":"waiting","detail":"Bash","cwd":"/x","pane":"%3",
                       "socket":"/tmp/s","pid":null,"since":5,"updated_at":6,"branch":"main","context":42}"#;
        let record: Record = serde_json::from_str(json).unwrap_or_default();
        assert_eq!(record.state, Status::Waiting);
        assert_eq!(record.pane_id(), PaneId::parse("%3"));
        assert_eq!(record.pid, None);
        assert_eq!(record.branch, "main");
        assert_eq!(record.context, Some(42));
    }

    #[test]
    fn does_not_write_the_old_branch_and_context() {
        let record = Record {
            branch: "main".to_owned(),
            context: Some(42),
            ..Record::default()
        };
        let json = serde_json::to_string(&record).unwrap_or_default();
        assert!(!json.contains("branch") && !json.contains("context"));
    }

    #[test]
    fn rejects_path_like_session_ids() {
        assert!(valid_session_id("0b6f-41_a"));
        assert!(!valid_session_id("../x"));
        assert!(!valid_session_id(""));
    }
}
