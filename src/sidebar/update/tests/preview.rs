use super::*;
use crate::sidebar::preview::{Captured, Mode, Preview};

fn model(mode: Mode) -> Model {
    Model {
        height: 20,
        preview: Preview {
            mode,
            ..Preview::default()
        },
        ..Model::default()
    }
}

fn capture(id: u32, window: u32) -> Effect {
    Effect::Capture(pane(id, window).id)
}

/// The sidebar (pane 9) in window 0, which runs no Claude (pane 1), next to window 1 (pane 2).
fn windows(focused: bool) -> Snapshot {
    let groups = [Group {
        name: GroupName::Session("s".to_owned()),
        claudes: vec![],
    }];
    let sidebar = Pane {
        sidebar: true,
        active: focused,
        ..pane(9, 0)
    };
    let rows = rows::with_windows(&groups, &[pane(1, 0), pane(2, 1)]);
    let panes = [sidebar, pane(1, 0), pane(2, 1)];
    super::super::super::snapshot::from_parts(rows, &panes, Some(pane(9, 0).id), false)
}

#[test]
fn a_claude_row_is_previewed_from_its_transcript_without_a_capture() {
    let (model, loaded) = update(model(Mode::Pane), Msg::Loaded(snapshot(true)));
    assert_eq!(selected(&model), Some(&claude_key("a")));
    let (_, moved) = update(model, Msg::Action(Action::Move(1)));
    assert!(loaded.is_empty() && moved.is_empty());
}

#[test]
fn each_reload_captures_the_selected_window() {
    let (model, effects) = update(model(Mode::Pane), Msg::Loaded(windows(true)));
    assert_eq!(
        selected(&model),
        Some(&RowKey::Window(pane(1, 0).window_id))
    );
    assert_eq!(effects, [capture(1, 0)]);
}

#[test]
fn moving_the_cursor_captures_the_new_window() {
    let model = loaded(model(Mode::Pane), windows(true));
    let (_, effects) = update(model, Msg::Action(Action::Move(1)));
    assert_eq!(effects, [capture(2, 1)]);
}

#[test]
fn a_capture_is_kept_for_the_preview() {
    let model = loaded(model(Mode::Pane), windows(true));
    let captured = Captured {
        pane: pane(1, 0).id,
        lines: vec!["$ ls".to_owned()],
    };
    let (model, effects) = update(model, Msg::Captured(captured.clone()));
    assert_eq!(model.captured, Some(captured));
    assert!(effects.is_empty());
}

#[test]
fn the_popup_key_opens_the_selected_pane_in_a_popup() {
    let model = loaded(model(Mode::Popup), snapshot(true));
    let (model, moved) = update(model, Msg::Action(Action::Move(1)));
    assert!(moved.is_empty());
    let (_, effects) = update(model, Msg::Action(Action::Preview));
    assert_eq!(effects, [Effect::PreviewPopup(pane(2, 1).id)]);
}

#[test]
fn off_asks_for_nothing() {
    let (model, loaded) = update(model(Mode::Off), Msg::Loaded(windows(true)));
    let (model, moved) = update(model, Msg::Action(Action::Move(1)));
    let (_, key) = update(model, Msg::Action(Action::Preview));
    assert!(loaded.is_empty() && moved.is_empty() && key.is_empty());
}

#[test]
fn a_tmux_session_without_claude_previews_nothing() {
    let groups = [Group {
        name: GroupName::Session("empty".to_owned()),
        claudes: vec![],
    }];
    let snapshot = Snapshot {
        rows: rows::rows(&groups),
        focused: true,
        ..Snapshot::default()
    };
    let (model, effects) = update(model(Mode::Pane), Msg::Loaded(snapshot));
    assert_eq!(selected(&model), Some(&RowKey::Session("empty".to_owned())));
    assert!(effects.is_empty());
    let popup = Model {
        preview: Preview {
            mode: Mode::Popup,
            ..Preview::default()
        },
        ..model
    };
    assert!(update(popup, Msg::Action(Action::Preview)).1.is_empty());
}

#[test]
fn an_unfocused_sidebar_captures_nothing_until_it_gets_the_focus() {
    let (model, unfocused) = update(model(Mode::Pane), Msg::Loaded(windows(false)));
    assert!(unfocused.is_empty());
    let (_, moved) = update(model.clone(), Msg::Action(Action::Move(1)));
    assert!(moved.is_empty());
    let (_, focused) = update(model, Msg::Loaded(windows(true)));
    assert_eq!(focused, [capture(1, 0)]);
}

#[test]
fn no_capture_when_the_pane_is_too_short_for_the_preview() {
    let short = Model {
        height: 8,
        ..model(Mode::Pane)
    };
    assert!(update(short, Msg::Loaded(windows(true))).1.is_empty());
}
