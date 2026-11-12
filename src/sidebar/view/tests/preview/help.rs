//! The key help in each preview mode.

use super::*;

#[test]
fn the_help_shows_the_popup_key_only_for_popup() {
    let help = |mode, messages| {
        screen(&windows_model(2, mode, 30), messages)
            .last()
            .cloned()
    };
    assert_eq!(
        help(Mode::Popup, &EN).as_deref(),
        Some("⏎ jump  j/k move  q close  ␣ peek")
    );
    assert_eq!(
        help(Mode::Popup, &JA).as_deref(),
        Some("⏎ 移動  j/k 選択  q 閉じる  ␣ 中身")
    );
    assert_eq!(
        help(Mode::Pane, &EN).as_deref(),
        Some("⏎ jump  j/k move  q close")
    );
    let other_key = Model {
        preview: Preview {
            mode: Mode::Popup,
            key: 'v',
            ..Preview::default()
        },
        ..windows_model(2, Mode::Popup, 30)
    };
    assert!(
        screen(&other_key, &EN)
            .last()
            .is_some_and(|l| l.ends_with("v peek"))
    );
}
