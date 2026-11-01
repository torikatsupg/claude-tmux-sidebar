use super::*;
use crate::sidebar::label::Label;

fn window_model(name: &str) -> Model {
    let mut claude = busy_claude(1, None);
    claude.pane = claude.pane.map(|p| Pane {
        window_name: name.to_owned(),
        ..p
    });
    Model {
        label: Label::Window,
        ..model_of(claude, false)
    }
}

#[test]
fn the_window_label_replaces_the_directory_and_branch() {
    let screen = screen(&window_model("editor"), &EN);
    assert_eq!(
        screen.get(3).map(String::as_str),
        Some(" ✓ 2:editor                    2s ")
    );
    assert_eq!(
        screen.get(4).map(String::as_str),
        Some(" └ fix the failing sidebar test … ")
    );
}

#[test]
fn a_narrow_row_cuts_the_window_label() {
    let model = window_model("a long window name");
    let row = lines(&model, &EN, 16, model.height).get(3).map(text);
    assert_eq!(row.as_deref(), Some(" ✓ 2:a long… 2s "));
}
