use super::*;

/// The sidebar (pane 9) sits in window 0 next to Claude `a`; window 1 runs no Claude (pane 2), and
/// window 2 runs Claude `b`. Both Claudes wait for the user.
fn snapshot() -> Snapshot {
    let groups = [Group {
        name: GroupName::Session("s".to_owned()),
        claudes: vec![
            claude("a", pane(1, 0), Status::Waiting),
            claude("b", pane(3, 2), Status::Waiting),
        ],
    }];
    let sidebar = Pane {
        sidebar: true,
        active: true,
        ..pane(9, 0)
    };
    let rows = rows::with_windows(&groups, &[pane(2, 1)]);
    let panes = [sidebar, pane(1, 0), pane(2, 1), pane(3, 2)];
    super::super::super::snapshot::from_parts(rows, &panes, Some(pane(9, 0).id), false)
}

fn model() -> Model {
    loaded(
        Model {
            height: 20,
            ..Model::default()
        },
        snapshot(),
    )
}

fn window_key() -> RowKey {
    RowKey::Window(pane(2, 1).window_id)
}

fn goto_window() -> Vec<Effect> {
    vec![Effect::Goto {
        target: Target::Pane(pane(2, 1).id),
        leaves_window: true,
    }]
}

#[test]
fn the_cursor_stops_on_a_window_but_attention_skips_it() {
    let model = model();
    assert_eq!(selected(&model), Some(&claude_key("a")));
    let down = update(model.clone(), Msg::Action(Action::Move(1))).0;
    assert_eq!(selected(&down), Some(&window_key()));
    let next = update(model, Msg::Action(Action::Attention(Direction::Next))).0;
    assert_eq!(selected(&next), Some(&claude_key("b")));
    let back = update(next, Msg::Action(Action::Attention(Direction::Prev))).0;
    assert_eq!(selected(&back), Some(&claude_key("a")));
}

#[test]
fn enter_on_a_window_goes_there() {
    let down = update(model(), Msg::Action(Action::Move(1))).0;
    assert_eq!(update(down, Msg::Action(Action::Activate)).1, goto_window());
}

#[test]
fn clicking_a_window_goes_there() {
    let y = rows::FIRST_ROW + 3;
    let (model, effects) = update(model(), Msg::Clicked { y });
    assert_eq!(selected(&model), Some(&window_key()));
    assert_eq!(effects, goto_window());
}
