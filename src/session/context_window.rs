//! How full a model's context window is: the size set for it in `@cts_context_windows`, otherwise a
//! guess from its name and the token count. A size the status line reported goes before both.

use std::collections::HashMap;

/// Space-separated `model=tokens` pairs, e.g. `claude-opus-5-5=1000000`.
pub const OPTION: &str = "@cts_context_windows";
const STANDARD_LIMIT: u64 = 200_000;
const LONG_LIMIT: u64 = 1_000_000;
/// Claude Code marks a 1M-context model with this suffix (`claude-opus-5-5[1m]`).
const LONG_SUFFIX: &str = "[1m]";
const MAX_VERSION_DIGITS: usize = 2;
/// Model families that always run with the 1M context window.
const LONG_ONLY: [&str; 2] = ["fable", "mythos"];
/// Model families and the `(major, minor)` version from which they run with it without the suffix.
const LONG_FROM: [(&str, (u32, u32)); 3] =
    [("opus", (4, 7)), ("sonnet", (5, 0)), ("haiku", (5, 5))];

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
        share(tokens, window)
    }
}

/// `tokens` as a share of a `window` of known, positive size, capped at 100.
pub fn share(tokens: u64, window: u64) -> u8 {
    let share = tokens.saturating_mul(100) / window;
    u8::try_from(share.min(100)).unwrap_or(100)
}

fn pair(text: &str) -> Option<(String, u64)> {
    let (model, size) = text.split_once('=')?;
    let size = size.parse::<u64>().ok().filter(|&s| s > 0)?;
    (!model.is_empty()).then(|| (model.to_owned(), size))
}

/// The window Claude Code documents for the model. More tokens than the standard window means it must
/// be the long one.
fn guess(tokens: u64, model: &str) -> u64 {
    let long = model.ends_with(LONG_SUFFIX) || long_by_default(model) || tokens > STANDARD_LIMIT;
    if long { LONG_LIMIT } else { STANDARD_LIMIT }
}

fn long_by_default(model: &str) -> bool {
    LONG_ONLY.iter().any(|family| model.contains(family))
        || LONG_FROM
            .iter()
            .any(|(family, from)| version(model, family).is_some_and(|v| v >= *from))
}

/// The version after the family in a model id: `claude-opus-4-7-20260101` is `(4, 7)` for `opus`. A date
/// is longer than a version number, so `claude-3-5-sonnet-20241022` has none for `sonnet`.
fn version(model: &str, family: &str) -> Option<(u32, u32)> {
    let (_, rest) = model.split_once(&format!("{family}-"))?;
    let parts: Vec<&str> = rest.split(|c: char| !c.is_ascii_alphanumeric()).collect();
    let number = |i: usize| {
        parts
            .get(i)
            .filter(|part| part.len() <= MAX_VERSION_DIGITS)
            .and_then(|part| part.parse().ok())
    };
    Some((number(0)?, number(1).unwrap_or(0)))
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
        assert_eq!(windows.percent(100_000, "claude-opus-4-6"), 50);
        assert_eq!(windows.percent(100_000, "claude-opus-4-6[1m]"), 10);
        assert_eq!(windows.percent(100_000, "claude-fable-5-1"), 10);
        assert_eq!(windows.percent(100_000, "claude-mythos-1"), 10);
        assert_eq!(windows.percent(300_000, "claude-opus-4-6"), 30);
        assert_eq!(windows.percent(5_000_000, "claude-opus-4-6[1m]"), 100);
    }

    #[test]
    fn documented_models_run_with_the_long_window_without_the_suffix() {
        let long = [
            "claude-opus-4-7",
            "claude-opus-4-7-20260101",
            "claude-opus-5",
            "claude-opus-5-5",
            "claude-sonnet-5",
            "claude-sonnet-5-20260101",
            "claude-sonnet-5-5",
            "claude-haiku-5-5",
            "claude-haiku-6",
            "claude-fable-5-1",
            "anthropic.claude-opus-4-7-v1:0",
            "claude-opus-4-7@20260101",
        ];
        assert!(long.into_iter().all(long_by_default), "{long:?}");
    }

    #[test]
    fn older_models_need_the_suffix() {
        let standard = [
            "claude-opus-4-6",
            "claude-opus-4-20250514",
            "claude-opus-4-5-20251101",
            "claude-sonnet-4-6",
            "claude-haiku-4-5-20251001",
            "claude-haiku-5",
            "claude-3-5-sonnet-20241022",
            "claude-3-opus-20240229",
            "unknown",
            "",
        ];
        assert!(!standard.into_iter().any(long_by_default), "{standard:?}");
    }
}
