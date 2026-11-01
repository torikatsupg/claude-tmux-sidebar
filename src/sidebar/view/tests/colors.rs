use super::*;

#[test]
fn the_title_and_status_use_the_configured_colors() {
    let theme = Theme {
        accent: Color::Rgb(0xd9, 0x77, 0x57),
        done: Color::Indexed(82),
        ..Theme::default()
    };
    let model = Model {
        theme,
        ..model(Status::Done, "", false)
    };
    let lines = lines(&model, &EN, WIDTH, model.height);
    let colors = |i: usize| {
        lines
            .get(i)
            .map(|l| l.spans.iter().map(|s| s.style.fg).collect())
    };
    let title: Option<Vec<_>> = colors(0);
    assert!(title.is_some_and(|c| c.contains(&Some(theme.accent))));
    let row: Option<Vec<_>> = colors(3);
    assert!(row.is_some_and(|c| c.contains(&Some(theme.done))));
}
