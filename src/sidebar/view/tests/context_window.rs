use super::*;
use crate::session::context_window::ContextWindows;

#[test]
fn the_context_uses_the_configured_window_size() {
    let mut model = model_of(busy_claude(1, Some(84_000)), false);
    model.snapshot.context_windows = ContextWindows::parse("claude-opus-5-5=1000000");
    assert_eq!(
        screen(&model, &EN).get(4).map(String::as_str),
        Some(" └ fix the failing sidebar te… 8% ")
    );
}
