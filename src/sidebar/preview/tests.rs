use super::*;

fn options(pairs: &[(&str, &str)]) -> Preview {
    Preview::from_options(|name| {
        pairs
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| (*v).to_owned())
    })
}

fn key_option(value: &str) -> char {
    options(&[("@cts_preview_key", value)]).key
}

#[test]
fn the_option_defaults_to_pane_and_space() {
    assert_eq!(
        options(&[]),
        Preview {
            mode: Mode::Pane,
            ..Preview::default()
        }
    );
    assert_eq!(options(&[("@cts_preview", "popup")]).mode, Mode::Popup);
    assert_eq!(options(&[("@cts_preview", "off")]).mode, Mode::Off);
    assert_eq!(options(&[("@cts_preview", "bogus")]).mode, Mode::Pane);
}

#[test]
fn the_popup_key_is_space_or_one_character() {
    assert_eq!(key_option("v"), 'v');
    assert_eq!(key_option("space"), ' ');
    assert_eq!(key_option("C-v"), ' ');
    assert_eq!(options(&[("@cts_preview_key", "v")]).key_label(), "v");
    assert_eq!(Preview::default().key_label(), "␣");
}

#[test]
fn a_popup_key_taken_by_the_list_falls_back_to_space() {
    for taken in ["q", "j", "k", "g", "G", "n", "p", "l", "r"] {
        assert_eq!(key_option(taken), ' ', "{taken}");
    }
}

#[test]
fn the_popup_fits_the_pane_and_its_border_within_the_client() {
    let size = |s| popup_size(s);
    assert_eq!(size("165 50 200 52"), ("167".to_owned(), "52".to_owned()));
    assert_eq!(size("80 24 200 60"), ("82".to_owned(), "26".to_owned()));
    assert_eq!(size("200 60 200 60"), ("200".to_owned(), "60".to_owned()));
    assert_eq!(size(""), ("80%".to_owned(), "80%".to_owned()));
}

#[test]
fn wrapping_breaks_between_words() {
    assert_eq!(wrap("hello world foo", 11), ["hello world", "foo"]);
    assert_eq!(wrap("  - item one", 8), ["  - item", "one"]);
    assert_eq!(wrap("", 10), [""]);
}

#[test]
fn wrapping_breaks_between_wide_characters() {
    assert_eq!(wrap("日本語の文章です", 6), ["日本語", "の文章", "です"]);
    assert_eq!(wrap("abc 日本語 def", 7), ["abc 日", "本語", "def"]);
}

#[test]
fn a_word_longer_than_the_width_is_split() {
    assert_eq!(wrap("abcdefghij", 4), ["abcd", "efgh", "ij"]);
    assert_eq!(
        wrap("see /very/long/path/name ok", 10),
        ["see", "/very/long", "/path/name", "ok"]
    );
}

#[test]
fn a_reply_loses_code_fences_heading_marks_and_extra_blank_lines() {
    let reply = "## Plan\n\n\n```sh\ncargo test\n```\n\n#tag stays\n\n";
    assert_eq!(tidy(reply), ["Plan", "", "cargo test", "", "#tag stays"]);
}

#[test]
fn the_rule_color_follows_the_pane_border_style() {
    use ratatui::style::Color;
    assert_eq!(
        border_color(Some("fg=colour238")),
        Some(Color::Indexed(238))
    );
    assert_eq!(
        border_color(Some("bg=red,fg=#336699")),
        Some(Color::Rgb(0x33, 0x66, 0x99))
    );
    assert_eq!(border_color(Some("default")), Some(Color::Reset));
    assert_eq!(border_color(None), Some(Color::Reset));
    assert_eq!(border_color(Some("fg=bogus")), None);
}
