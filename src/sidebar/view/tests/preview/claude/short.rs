//! A panel too short for everything.

use super::*;

#[test]
fn a_short_running_preview_keeps_the_counts() {
    let model = claude_model(Status::Running, asking("cargo build"), 15);
    assert_eq!(
        panel(&model, 3),
        ["⠋ Bash: cargo build", "⠋ agents 2", "⠋ bg 1"]
    );
}

#[test]
fn a_short_finished_preview_keeps_the_end_of_the_reply_and_the_changes() {
    let model = claude_model(Status::Idle, replied("one\ntwo\nthree\nfour"), 18);
    assert_eq!(
        panel(&model, 4),
        ["three", "four", "⠋ agents 2", "± +128 -14 (3 files)"]
    );
    let shorter = claude_model(Status::Idle, replied("one\ntwo\nthree\nfour"), 15);
    assert_eq!(
        panel(&shorter, 3),
        ["three", "four", "± +128 -14 (3 files)"]
    );
}
