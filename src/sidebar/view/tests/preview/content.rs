//! The rule above the preview.

use super::*;

#[test]
fn the_rule_names_the_row_and_takes_the_border_color() {
    let model = Model {
        preview: Preview {
            mode: Mode::Pane,
            rule: Some(Color::Indexed(238)),
            ..Preview::default()
        },
        ..windows_model(2, Mode::Pane, 30)
    };
    let rule = lines_of(&model, 20);
    assert!(
        rule.spans
            .iter()
            .all(|s| s.style.fg == Some(Color::Indexed(238)))
    );
    let dim = lines_of(&windows_model(2, Mode::Pane, 30), 20);
    assert!(
        dim.spans
            .iter()
            .all(|s| s.style.fg == Some(Theme::DEFAULT.dim))
    );
    let narrow = lines(&windows_model(30, Mode::Pane, 30), &EN, 10, 30);
    assert_eq!(narrow.get(20).map(text), Some("─ s/29:… ─".to_owned()));
}

fn lines_of(model: &Model, y: usize) -> Line<'static> {
    lines(model, &EN, WIDTH, 30)
        .get(y)
        .cloned()
        .unwrap_or_default()
}
