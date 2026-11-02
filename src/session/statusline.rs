//! `statusline`: records the context window size Claude Code gives a status line command on stdin.

use std::io;

use serde::Deserialize;

use super::{store, valid_session_id};
use crate::context::Context;

#[derive(Debug, Default, Deserialize)]
struct Input {
    #[serde(default)]
    session_id: String,
    #[serde(default)]
    context_window: Window,
}

#[derive(Debug, Default, Deserialize)]
struct Window {
    #[serde(default)]
    context_window_size: u64,
}

/// The session and its window size, when `input` has both in a usable form.
fn parse(input: &str) -> Option<(String, u64)> {
    let input: Input = serde_json::from_str(input).ok()?;
    let size = input.context_window.context_window_size;
    (valid_session_id(&input.session_id) && size > 0).then_some((input.session_id, size))
}

/// Handles one status line update. Errors are swallowed so that a status line script keeps working.
pub fn run(ctx: &Context, input: &str) {
    let _ = parse(input).map_or(Ok(()), |(id, size)| record(ctx, &id, size));
}

/// Writes the size of a session the hooks know, only when it changed: status lines update often, and
/// every write makes the sidebars reload.
fn record(ctx: &Context, session_id: &str, size: u64) -> io::Result<()> {
    let known = store::read(&ctx.state_dir, session_id).is_some();
    if known && store::read_window(&ctx.state_dir, session_id) != Some(size) {
        store::write_window(&ctx.state_dir, session_id, size)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_session_and_its_size() {
        let input = r#"{"session_id":"s1","context_window":{"context_window_size":1000000,"used_percentage":8}}"#;
        assert_eq!(parse(input), Some(("s1".to_owned(), 1_000_000)));
    }

    #[test]
    fn ignores_input_it_cannot_use() {
        let inputs = [
            "",
            "not json",
            "{}",
            r#"{"session_id":"s1"}"#,
            r#"{"session_id":"s1","context_window":{"context_window_size":0}}"#,
            r#"{"session_id":"s1","context_window":{"context_window_size":"big"}}"#,
            r#"{"session_id":"s1","context_window":{"context_window_size":null}}"#,
            r#"{"session_id":"../x","context_window":{"context_window_size":200000}}"#,
        ];
        assert_eq!(inputs.map(parse), inputs.map(|_| None));
    }
}
