use super::*;
use crate::sidebar::preview::{Changes, GitAsk, GitRead, Mode, Preview};

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

fn git(dir: &str) -> Effect {
    asking(dir, &[])
}

fn asking(dir: &str, others: &[&str]) -> Effect {
    Effect::Git(GitAsk {
        dir: dir.to_owned(),
        others: others.iter().map(|&o| o.to_owned()).collect(),
    })
}

/// A pane working in `/w<id>`.
fn at_work(id: u32, window: u32) -> Pane {
    Pane {
        path: format!("/w{id}"),
        ..pane(id, window)
    }
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
    let rows = rows::with_windows(&groups, &[at_work(1, 0), at_work(2, 1)]);
    let panes = [sidebar, at_work(1, 0), at_work(2, 1)];
    super::super::super::snapshot::from_parts(rows, &panes, Some(pane(9, 0).id), false)
}

/// The Claude rows of [`snapshot`], working in `/a` and `/b`.
fn in_dirs(sidebar_active: bool) -> Snapshot {
    let snapshot = snapshot(sidebar_active);
    let rows = snapshot
        .rows
        .iter()
        .map(|row| match row {
            Row::Session(c) => Row::Session(Claude {
                record: Record {
                    cwd: format!("/{}", c.record.session_id),
                    ..c.record.clone()
                },
                ..c.clone()
            }),
            other => other.clone(),
        })
        .collect();
    Snapshot { rows, ..snapshot }
}

#[test]
fn the_preview_reads_the_selected_sessions_directory_with_git_but_no_screen() {
    let (model, loaded) = update(model(Mode::Pane), Msg::Loaded(in_dirs(true)));
    assert_eq!(selected(&model), Some(&claude_key("a")));
    assert_eq!(loaded, [asking("/a", &["/b"])]);
    let (_, moved) = update(model, Msg::Action(Action::Move(1)));
    assert_eq!(moved, [asking("/b", &["/a"])]);
}

#[test]
fn each_reload_reads_the_selected_windows_directory() {
    let (model, effects) = update(model(Mode::Pane), Msg::Loaded(windows(true)));
    assert_eq!(
        selected(&model),
        Some(&RowKey::Window(pane(1, 0).window_id))
    );
    assert_eq!(effects, [git("/w1")]);
    let (_, moved) = update(model, Msg::Action(Action::Move(1)));
    assert_eq!(moved, [git("/w2")]);
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
fn an_unfocused_sidebar_runs_no_git_until_it_gets_the_focus() {
    let (model, unfocused) = update(model(Mode::Pane), Msg::Loaded(windows(false)));
    assert!(unfocused.is_empty());
    let (_, moved) = update(model.clone(), Msg::Action(Action::Move(1)));
    assert!(moved.is_empty());
    let (_, focused) = update(model, Msg::Loaded(windows(true)));
    assert_eq!(focused, [git("/w1")]);
}

#[test]
fn no_git_when_the_pane_is_too_short_for_the_preview() {
    let short = Model {
        height: 8,
        ..model(Mode::Pane)
    };
    assert!(update(short, Msg::Loaded(windows(true))).1.is_empty());
}

mod git;
