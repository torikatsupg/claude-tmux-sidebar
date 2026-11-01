//! What a session's transcript tells about it: the git branch and how many tokens fill the context
//! window (unknown right after a compaction).

pub mod cache;

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use serde::Deserialize;

/// Only the end of the transcript is read; the last assistant entry is nearly always in it.
const TAIL_BYTES: u64 = 256 * 1024;

/// The latest state of a session, as of its last assistant entry or compaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Latest {
    pub branch: String,
    /// `None` after a compaction until the next assistant entry.
    pub usage: Option<Usage>,
}

/// The tokens in the context window and the model they were sent to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Usage {
    pub tokens: u64,
    pub model: String,
}

#[derive(Deserialize)]
struct Entry {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    subtype: String,
    #[serde(default, rename = "isSidechain")]
    sidechain: bool,
    #[serde(default, rename = "gitBranch")]
    branch: String,
    #[serde(default, rename = "requestedModel")]
    requested_model: String,
    message: Option<Message>,
}

#[derive(Deserialize)]
struct Message {
    #[serde(default)]
    model: String,
    usage: Option<TokenCounts>,
}

#[derive(Deserialize)]
#[expect(
    clippy::struct_field_names,
    reason = "the names of the transcript format"
)]
struct TokenCounts {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
}

/// Reads the end of the transcript at `path`; `None` when it is missing or has no assistant entry or compaction yet.
pub fn read(path: &Path) -> Option<Latest> {
    tail(path).ok().as_deref().and_then(latest)
}

fn tail(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    file.seek(SeekFrom::Start(len.saturating_sub(TAIL_BYTES)))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// The last main-thread assistant entry with token usage or compaction boundary in `text` (JSON lines).
/// A boundary has no usage, so the context is unknown until the next assistant entry.
pub fn latest(text: &str) -> Option<Latest> {
    text.lines()
        .rev()
        .filter(|line| line.contains("\"usage\"") || line.contains("\"compact_boundary\""))
        .filter_map(|line| serde_json::from_str::<Entry>(line).ok())
        .filter(|e| !e.sidechain)
        .find_map(|entry| match entry.kind.as_str() {
            "system" if entry.subtype == "compact_boundary" => Some(Latest {
                branch: entry.branch,
                usage: None,
            }),
            "assistant" => {
                let message = entry.message?;
                let usage = message.usage?;
                let model = Some(entry.requested_model)
                    .filter(|m| !m.is_empty())
                    .unwrap_or(message.model);
                let tokens = usage.input_tokens
                    + usage.cache_creation_input_tokens
                    + usage.cache_read_input_tokens;
                Some(Latest {
                    branch: entry.branch,
                    usage: Some(Usage { tokens, model }),
                })
            }
            _ => None,
        })
}

#[cfg(test)]
mod tests;
