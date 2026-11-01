//! How full a model's context window is: the size set for it in `@cts_context_windows`, otherwise a
//! guess from its name and the token count.

use std::collections::HashMap;

/// Space-separated `model=tokens` pairs, e.g. `claude-opus-5-5=1000000`.
pub const OPTION: &str = "@cts_context_windows";
const STANDARD_LIMIT: u64 = 200_000;
const LONG_LIMIT: u64 = 1_000_000;
/// Claude Code marks a 1M-context model with this suffix (`claude-opus-5-5[1m]`).
const LONG_SUFFIX: &str = "[1m]";
/// Model families that always run with the 1M context window.
const LONG_ONLY: [&str; 2] = ["fable", "mythos"];

/// The context window sizes the user set, by model id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ContextWindows(HashMap<String, u64>);

impl ContextWindows {
    /// Reads the value of [`OPTION`], skipping pairs without a model or a positive size.
    pub fn parse(option: &str) -> Self {
        Self(option.split_whitespace().filter_map(pair).collect())
    }

    /// `tokens` as a share of `model`'s context window, capped at 100.
    pub fn percent(&self, tokens: u64, model: &str) -> u8 {
        let window = self
            .0
            .get(model)
            .copied()
            .unwrap_or_else(|| guess(tokens, model));
        let share = tokens.saturating_mul(100) / window;
        u8::try_from(share.min(100)).unwrap_or(100)
    }
}

fn pair(text: &str) -> Option<(String, u64)> {
    let (model, size) = text.split_once('=')?;
    let size = size.parse::<u64>().ok().filter(|&s| s > 0)?;
    (!model.is_empty()).then(|| (model.to_owned(), size))
}

/// More tokens than the standard window means it must be the long one.
fn guess(tokens: u64, model: &str) -> u64 {
    let long = model.ends_with(LONG_SUFFIX)
        || LONG_ONLY.iter().any(|family| model.contains(family))
        || tokens > STANDARD_LIMIT;
    if long { LONG_LIMIT } else { STANDARD_LIMIT }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pairs_and_skips_malformed_ones() {
        let windows = ContextWindows::parse(
            " claude-opus-5-5=1000000  claude-sonnet-4-6=200000 bare =5 x=abc y=0 z=-1 w=",
        );
        let expected = [
            ("claude-opus-5-5", 1_000_000),
            ("claude-sonnet-4-6", 200_000),
        ]
        .map(|(m, s)| (m.to_owned(), s));
        assert_eq!(windows, ContextWindows(HashMap::from(expected)));
        assert_eq!(ContextWindows::parse(""), ContextWindows::default());
    }

    #[test]
    fn a_listed_model_uses_the_configured_size() {
        let windows = ContextWindows::parse("claude-opus-5-5=1000000 claude-tiny=100000");
        assert_eq!(windows.percent(100_000, "claude-opus-5-5"), 10);
        assert_eq!(windows.percent(300_000, "claude-tiny"), 100);
        assert_eq!(windows.percent(100_000, "claude-opus-5-5[1m]"), 10);
    }

    #[test]
    fn an_unlisted_model_falls_back_to_the_guess() {
        let windows = ContextWindows::parse("claude-other=50000");
        assert_eq!(windows.percent(100_000, "claude-opus-5-5"), 50);
        assert_eq!(windows.percent(100_000, "claude-opus-5-5[1m]"), 10);
        assert_eq!(windows.percent(100_000, "claude-fable-5-1"), 10);
        assert_eq!(windows.percent(100_000, "claude-mythos-1"), 10);
        assert_eq!(windows.percent(300_000, "claude-opus-5-5"), 30);
        assert_eq!(windows.percent(5_000_000, "claude-opus-5-5[1m]"), 100);
    }
}
