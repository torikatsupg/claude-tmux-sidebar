use super::*;

#[test]
fn names_map_to_the_ansi_colors() {
    assert_eq!(color("black"), Some(Color::Black));
    assert_eq!(color("magenta"), Some(Color::Magenta));
    assert_eq!(color("white"), Some(Color::Gray));
    assert_eq!(color("default"), Some(Color::Reset));
}

#[test]
fn bright_names_map_to_the_light_colors() {
    assert_eq!(color("brightblack"), Some(Color::DarkGray));
    assert_eq!(color("brightred"), Some(Color::LightRed));
    assert_eq!(color("brightcyan"), Some(Color::LightCyan));
    assert_eq!(color("brightwhite"), Some(Color::White));
}

#[test]
fn colour_and_color_take_an_index_up_to_255() {
    assert_eq!(color("colour0"), Some(Color::Indexed(0)));
    assert_eq!(color("colour237"), Some(Color::Indexed(237)));
    assert_eq!(color("color255"), Some(Color::Indexed(255)));
    assert_eq!(color("colour256"), None);
    assert_eq!(color("colour"), None);
    assert_eq!(color("colour+5"), None);
    assert_eq!(color("colour-1"), None);
}

#[test]
fn hex_takes_exactly_six_digits() {
    assert_eq!(color("#d97757"), Some(Color::Rgb(0xd9, 0x77, 0x57)));
    assert_eq!(color("#000000"), Some(Color::Rgb(0, 0, 0)));
    assert_eq!(color("#fff"), None);
    assert_eq!(color("#12345g"), None);
    assert_eq!(color("#+12345"), None);
    assert_eq!(color("#1234567"), None);
}

#[test]
fn case_does_not_matter() {
    assert_eq!(color("Magenta"), Some(Color::Magenta));
    assert_eq!(color("BrightBlue"), Some(Color::LightBlue));
    assert_eq!(color("COLOUR12"), Some(Color::Indexed(12)));
    assert_eq!(color("#D97757"), Some(Color::Rgb(0xd9, 0x77, 0x57)));
}

#[test]
fn garbage_is_rejected() {
    assert_eq!(color(""), None);
    assert_eq!(color("purple"), None);
    assert_eq!(color("bright"), None);
    assert_eq!(color("12"), None);
}

#[test]
fn no_options_give_the_default_theme() {
    assert_eq!(Theme::from_options(|_| None), Theme::default());
}

#[test]
fn set_options_override_their_role_and_invalid_ones_keep_the_default() {
    let theme = Theme::from_options(|name| {
        let value = match name {
            "@cts_color_accent" => "#D97757",
            "@cts_color_running" => "colour82",
            "@cts_color_done" => "brightblue",
            "@cts_color_cursor" => "nope",
            "@cts_color_critical" => "colour300",
            "@cts_color_muted" => "",
            _ => return None,
        };
        Some(value.to_owned())
    });
    let default = Theme::default();
    let expected = Theme {
        accent: Color::Rgb(0xd9, 0x77, 0x57),
        running: Color::Indexed(82),
        done: Color::LightBlue,
        ..default
    };
    assert_eq!(theme, expected);
}
