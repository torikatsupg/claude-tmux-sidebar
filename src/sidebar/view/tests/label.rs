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

fn auto_model(named: bool) -> Model {
    let mut claude = busy_claude(1, None);
    claude.pane = claude.pane.map(|p| Pane {
        window_name: "2.1.316".to_owned(),
        window_named: named,
        ..p
    });
    Model {
        label: Label::Auto,
        ..model_of(claude, false)
    }
}

#[test]
fn the_auto_label_names_only_a_window_named_by_hand() {
    let named = screen(&auto_model(true), &EN);
    assert!(named.get(3).is_some_and(|row| row.contains("2:2.1.316")));
    let automatic = screen(&auto_model(false), &EN);
    let same_as_dir = screen(
        &Model {
            label: Label::Dir,
            ..auto_model(false)
        },
        &EN,
    );
    assert_eq!(automatic, same_as_dir);
}

#[test]
fn only_auto_and_window_are_read_from_the_option() {
    assert_eq!(Label::parse(Some("auto")), Label::Auto);
    assert_eq!(Label::parse(Some("window")), Label::Window);
    assert_eq!(Label::parse(Some("dir")), Label::Dir);
    assert_eq!(Label::parse(Some("other")), Label::Dir);
    assert_eq!(Label::parse(None), Label::Dir);
}
