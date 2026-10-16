//! What a session's transcript tells about it: the git branch and how full the context window is.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use serde::Deserialize;

/// Only the end of the transcript is read; the last assistant entry is nearly always in it.
const TAIL_BYTES: u64 = 256 * 1024;
const STANDARD_LIMIT: u64 = 200_000;
const LONG_LIMIT: u64 = 1_000_000;
/// Claude Code marks a 1M-context model with this suffix (`claude-opus-5-5[1m]`).
const LONG_SUFFIX: &str = "[1m]";
/// Model families that always run with the 1M context window.
const LONG_ONLY: [&str; 2] = ["fable", "mythos"];

/// The latest state of a session, as of its last assistant entry.
#[derive(Debug, PartialEq, Eq)]
pub struct Latest {
    pub branch: String,
    /// Context usage in percent.
    pub context: u8,
}

#[derive(Deserialize)]
struct Entry {
    #[serde(rename = "type")]
    kind: String,
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
    usage: Option<Usage>,
}

#[derive(Deserialize)]
#[expect(
    clippy::struct_field_names,
    reason = "the names of the transcript format"
)]
struct Usage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
}

/// Reads the end of the transcript at `path`; `None` when it is missing or has no assistant entry yet.
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

/// The last main-thread assistant entry with token usage in `text` (JSON lines).
pub fn latest(text: &str) -> Option<Latest> {
    text.lines()
        .rev()
        .filter(|line| line.contains("\"usage\""))
        .filter_map(|line| serde_json::from_str::<Entry>(line).ok())
        .filter(|e| e.kind == "assistant" && !e.sidechain)
        .find_map(|entry| {
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
                context: percent(tokens, &model),
            })
        })
}

/// `tokens` as a share of the model's context window, capped at 100.
pub fn percent(tokens: u64, model: &str) -> u8 {
    let share = tokens.saturating_mul(100) / limit(tokens, model);
    u8::try_from(share.min(100)).unwrap_or(100)
}

/// The context window of `model`. More tokens than the standard window means it must be the long one.
fn limit(tokens: u64, model: &str) -> u64 {
    let long = model.ends_with(LONG_SUFFIX)
        || LONG_ONLY.iter().any(|family| model.contains(family))
        || tokens > STANDARD_LIMIT;
    if long { LONG_LIMIT } else { STANDARD_LIMIT }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assistant(model: &str, branch: &str, tokens: u64, sidechain: bool) -> String {
        format!(
            r#"{{"type":"assistant","isSidechain":{sidechain},"gitBranch":"{branch}","requestedModel":"{model}","message":{{"model":"x","usage":{{"input_tokens":{tokens},"cache_creation_input_tokens":0,"cache_read_input_tokens":0}}}}}}"#
        )
    }

    #[test]
    fn picks_the_last_main_thread_assistant_entry() {
        let text = [
            assistant("claude-opus-5-5", "old", 20_000, false),
            assistant("claude-opus-5-5", "main", 100_000, false),
            assistant("claude-haiku-5-5", "side", 10_000, true),
            r#"{"type":"user","message":{"content":"hi"}}"#.to_owned(),
            "partial line".to_owned(),
        ]
        .join("\n");
        assert_eq!(
            latest(&text),
            Some(Latest {
                branch: "main".to_owned(),
                context: 50
            })
        );
        assert_eq!(latest("nothing"), None);
    }

    #[test]
    fn the_long_window_comes_from_the_suffix_the_family_or_the_size() {
        assert_eq!(percent(100_000, "claude-opus-5-5"), 50);
        assert_eq!(percent(100_000, "claude-opus-5-5[1m]"), 10);
        assert_eq!(percent(100_000, "claude-fable-5-1"), 10);
        assert_eq!(percent(300_000, "claude-opus-5-5"), 30);
        assert_eq!(percent(5_000_000, "claude-opus-5-5[1m]"), 100);
    }
}
