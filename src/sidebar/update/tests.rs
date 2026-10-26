use super::*;
use crate::listing::{Group, GroupName};
use crate::session::live::Claude;
use crate::session::{Record, Status};
use crate::tmux::pane::{Pane, fixture::pane};

fn claude(id: &str, pane: Pane, state: Status) -> Claude {
    let record = Record {
        session_id: id.to_owned(),
        pane: pane.id.to_string(),
        state,
        ..Record::default()
    };
    Claude {
        record,
        pane: Some(pane),
        latest: None,
    }
}

/// Window 0 holds the sidebar (pane 9) and the user's pane 1 running Claude `a`; window 1 holds Claude `b`.
fn snapshot(sidebar_active: bool) -> Snapshot {
    let claudes = vec![
        claude("a", pane(1, 0), Status::Running),
        claude("b", pane(2, 1), Status::Waiting),
    ];
    let groups = [Group {
        name: GroupName::Session("s".to_owned()),
        claudes,
    }];
    let sidebar = Pane {
        sidebar: true,
        active: sidebar_active,
        ..pane(9, 0)
    };
    let user = Pane {
        active: !sidebar_active,
        last: sidebar_active,
        ..pane(1, 0)
    };
    let rows = rows::rows(&groups);
    super::super::snapshot::from_parts(
        rows,
        &[sidebar, user, pane(2, 1)],
        Some(pane(9, 0).id),
        false,
    )
}

fn loaded(model: Model, snapshot: Snapshot) -> Model {
    update(model, Msg::Loaded(snapshot)).0
}

fn selected(model: &Model) -> Option<&RowKey> {
    model.selection.selected.as_ref()
}

fn claude_key(id: &str) -> RowKey {
    RowKey::Claude(id.to_owned())
}

#[test]
fn selection_mirrors_the_focused_pane_while_the_sidebar_is_unfocused() {
    let model = loaded(
        Model {
            height: 20,
            ..Model::default()
        },
        snapshot(false),
    );
    assert_eq!(selected(&model), Some(&claude_key("a")));
    let moved = update(model, Msg::Action(Action::Move(1))).0;
    assert_eq!(selected(&moved), Some(&claude_key("b")));
    assert_eq!(
        selected(&loaded(moved, snapshot(false))),
        Some(&claude_key("a"))
    );
}

#[test]
fn the_user_owns_the_selection_while_the_sidebar_is_focused() {
    let model = loaded(
        Model {
            height: 20,
            ..Model::default()
        },
        snapshot(true),
    );
    let moved = update(model, Msg::Action(Action::Move(1))).0;
    assert_eq!(
        selected(&loaded(moved, snapshot(true))),
        Some(&claude_key("b"))
    );
}

#[test]
fn jumping_to_another_window_leaves_this_one() {
    let model = loaded(
        Model {
            height: 20,
            ..Model::default()
        },
        snapshot(true),
    );
    let moved = update(model, Msg::Action(Action::Move(1))).0;
    let (_, effects) = update(moved, Msg::Action(Action::Activate));
    assert_eq!(
        effects,
        [Effect::Goto {
            target: Target::Pane(pane(2, 1).id),
            leaves_window: true
        }]
    );
}

#[test]
fn clicking_a_row_selects_and_jumps() {
    let model = loaded(
        Model {
            height: 20,
            ..Model::default()
        },
        snapshot(false),
    );
    let (model, effects) = update(
        model,
        Msg::Clicked {
            y: rows::FIRST_ROW + 2,
        },
    );
    assert_eq!(selected(&model), Some(&claude_key("b")));
    assert!(matches!(
        effects.as_slice(),
        [Effect::Goto {
            target: Target::Pane(_),
            ..
        }]
    ));
}

#[test]
fn clicking_a_heading_does_nothing() {
    let model = loaded(
        Model {
            height: 20,
            ..Model::default()
        },
        snapshot(false),
    );
    let (model, effects) = update(model, Msg::Clicked { y: rows::FIRST_ROW });
    assert_eq!(selected(&model), Some(&claude_key("a")));
    assert!(effects.is_empty());
}

#[test]
fn moving_past_the_last_row_wraps_to_the_first() {
    let model = loaded(
        Model {
            height: 20,
            ..Model::default()
        },
        snapshot(true),
    );
    let last = update(model, Msg::Action(Action::Last)).0;
    let wrapped = update(last, Msg::Action(Action::Move(1))).0;
    assert_eq!(selected(&wrapped), Some(&claude_key("a")));
    let back = update(wrapped, Msg::Action(Action::Move(-1))).0;
    assert_eq!(selected(&back), Some(&claude_key("b")));
}

#[test]
fn quitting_closes_every_sidebar_but_only_the_popup() {
    let (_, sidebar) = update(Model::default(), Msg::Action(Action::Quit));
    let (_, popup) = update(
        Model {
            popup: true,
            ..Model::default()
        },
        Msg::Action(Action::Quit),
    );
    assert_eq!(sidebar, [Effect::CloseAll]);
    assert_eq!(popup, [Effect::Exit]);
}

#[test]
fn a_redundant_sidebar_removes_itself() {
    let redundant = Snapshot {
        redundant: true,
        ..Snapshot::default()
    };
    assert_eq!(
        update(Model::default(), Msg::Loaded(redundant)).1,
        [Effect::KillSelf, Effect::Exit]
    );
}
