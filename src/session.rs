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
    /// The git branch Claude last worked on.
    #[serde(default)]
    pub branch: String,
    /// How full the context window is, in percent.
    #[serde(default)]
    pub context: Option<u8>,
    /// Whether background tasks were still running when the main thread last stopped.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub background: bool,
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
                       "socket":"/tmp/s","pid":null,"since":5,"updated_at":6}"#;
        let record: Record = serde_json::from_str(json).unwrap_or_default();
        assert_eq!(record.state, Status::Waiting);
        assert_eq!(record.pane_id(), PaneId::parse("%3"));
        assert_eq!(record.pid, None);
    }

    #[test]
    fn rejects_path_like_session_ids() {
        assert!(valid_session_id("0b6f-41_a"));
        assert!(!valid_session_id("../x"));
        assert!(!valid_session_id(""));
    }
}
