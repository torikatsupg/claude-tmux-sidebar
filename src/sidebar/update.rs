//! The sidebar as a pure state machine: `update(model, msg)` returns the next model and the effects to run.

use super::rows::{self, Row, RowKey};
use super::selection::{self, Selection};
use super::snapshot::Snapshot;
use crate::jump::Direction;
use crate::tmux::pane::{PaneId, WindowId};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Model {
    pub popup: bool,
    pub snapshot: Snapshot,
    pub selection: Selection,
    /// Spinner animation frame.
    pub frame: usize,
    /// Seconds since the epoch, for ages.
    pub now: u64,
    pub height: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Msg {
    Loaded(Snapshot),
    Tick { now: u64, frame: usize },
    Resized { height: u16 },
    Clicked { y: u16 },
    Action(Action),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Move(isize),
    First,
    Last,
    Attention(Direction),
    Activate,
    Quit,
    Reload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Go to a pane or session. `leaves_window`: the target is in another window, so this window's
    /// focus goes back to the user's pane (if the sidebar has it) instead of staying on the sidebar.
    Goto {
        target: Target,
        leaves_window: bool,
    },
    /// Close the sidebar in every window, this one included.
    CloseAll,
    /// Remove this redundant sidebar pane.
    KillSelf,
    Reload,
    Exit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Pane(PaneId),
    Session(String),
}

pub fn update(model: Model, msg: Msg) -> (Model, Vec<Effect>) {
    match msg {
        Msg::Loaded(snapshot) if snapshot.redundant => {
            (model, vec![Effect::KillSelf, Effect::Exit])
        }
        Msg::Loaded(snapshot) => {
            let selection = selection::follow(model.selection.clone(), &snapshot, model.popup);
            (
                Model {
                    snapshot,
                    selection,
                    ..model
                },
                vec![],
            )
        }
        Msg::Tick { now, frame } => (
            Model {
                frame,
                now,
                ..model
            },
            vec![],
        ),
        Msg::Resized { height } => (Model { height, ..model }, vec![]),
        Msg::Clicked { y } => match rows::at(&model.snapshot.rows, model.height, y) {
            Some(key) => activate(select(model, Some(key))),
            None => (model, vec![]),
        },
        Msg::Action(action) => act(model, action),
    }
}

fn act(model: Model, action: Action) -> (Model, Vec<Effect>) {
    match action {
        Action::Activate => activate(model),
        Action::Quit if model.popup => (model, vec![Effect::Exit]),
        // No Exit: `off` kills this pane after restoring the layout; exiting first would remove the pane
        // before `off` sees it.
        Action::Quit => (model, vec![Effect::CloseAll]),
        Action::Reload => (model, vec![Effect::Reload]),
        Action::Move(_) | Action::First | Action::Last | Action::Attention(_) => {
            let key = moved_to(&model, action);
            (select(model, key), vec![])
        }
    }
}

fn moved_to(model: &Model, action: Action) -> Option<RowKey> {
    let keys = rows::keys(&model.snapshot.rows);
    let selected = model.selection.selected.as_ref();
    match action {
        Action::Move(delta) => selection::moved(&keys, selected, delta),
        Action::First => keys.first().cloned(),
        Action::Last => keys.last().cloned(),
        Action::Attention(direction) => selection::attention(
            &keys,
            &attention_keys(&model.snapshot.rows),
            selected,
            direction,
        ),
        Action::Activate | Action::Quit | Action::Reload => None,
    }
}

fn select(model: Model, key: Option<RowKey>) -> Model {
    let Model { selection, .. } = &model;
    let selection = Selection {
        selected: key.or_else(|| selection.selected.clone()),
        synced: selection.synced.clone(),
    };
    Model { selection, ..model }
}

fn attention_keys(rows: &[Row]) -> Vec<RowKey> {
    rows.iter()
        .filter(|r| matches!(r, Row::Session(c) if c.record.state.needs_attention()))
        .filter_map(rows::key)
        .collect()
}

/// Jumps to the selected row; a popup closes after a successful jump.
fn activate(model: Model) -> (Model, Vec<Effect>) {
    let goto = model
        .selection
        .selected
        .as_ref()
        .and_then(|key| destination(&model.snapshot, key));
    let leaves = |window: Option<WindowId>| window.is_some() && window != model.snapshot.window;
    let effects = goto
        .into_iter()
        .flat_map(|(target, window)| {
            let exit = model.popup.then_some(Effect::Exit);
            [Effect::Goto {
                target,
                leaves_window: leaves(window),
            }]
            .into_iter()
            .chain(exit)
        })
        .collect();
    (model, effects)
}

fn destination(snapshot: &Snapshot, key: &RowKey) -> Option<(Target, Option<WindowId>)> {
    match key {
        RowKey::Session(name) => Some((
            Target::Session(name.clone()),
            snapshot.session_windows.get(name).copied(),
        )),
        RowKey::Claude(id) => snapshot.rows.iter().find_map(|r| match r {
            Row::Session(c) if c.record.session_id == *id => c
                .pane
                .as_ref()
                .map(|p| (Target::Pane(p.id), Some(p.window_id))),
            _ => None,
        }),
    }
}

#[cfg(test)]
mod tests;
