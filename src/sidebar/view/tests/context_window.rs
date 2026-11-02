use super::*;
use crate::session::context_window::ContextWindows;

#[test]
fn the_context_uses_the_configured_window_size() {
    let mut model = model_of(busy_claude(1, Some(84_000)), false);
    model.snapshot.context_windows = ContextWindows::parse("claude-opus-4-6=1000000");
    assert_eq!(
        screen(&model, &EN).get(4).map(String::as_str),
        Some("└ fix the failing sidebar tes… 8% ")
    );
}

#[test]
fn a_session_before_its_first_reply_has_a_detail_line_at_zero() {
    let claude = Claude {
        record: Record {
            transcript: "/t".to_owned(),
            ..record(Status::Idle, "")
        },
        pane: None,
        latest: None,
    };
    let screen = screen(&model_of(claude, false), &EN);
    let detail = screen.get(4).cloned().unwrap_or_default();
    assert!(
        detail.starts_with("└ ") && detail.ends_with(" 0% "),
        "{detail}"
    );
}
