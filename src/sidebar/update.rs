//! The sidebar as a pure state machine: `update(model, msg)` returns the next model and the effects to run.

mod activate;
mod layout;
mod preview;

use std::collections::BTreeMap;

use super::preview::{Git, GitAsk, GitRead, Preview};
use super::rows::{self, Row, RowKey};
use super::selection::{self, Selection};
use super::snapshot::Snapshot;
use crate::jump::Direction;
use crate::tmux::pane::PaneId;
use activate::activate;

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
    /// The key help is shown on the last line.
    pub help: bool,
    pub label: super::label::Label,
    pub indent: super::indent::Indent,
    pub theme: super::theme::Theme,
    pub preview: Preview,
    /// What `git` said about the preview's directory; shown only while it is still that directory.
    pub git: Option<Git>,
    /// The repository top directory of each session directory git was asked about; empty outside one.
    pub tops: BTreeMap<String, String>,
    /// `$HOME`, written as `~` in the preview.
    pub home: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Msg {
    Loaded(Snapshot),
    Tick { now: u64, frame: usize },
    Resized { height: u16 },
    Clicked { y: u16 },
    Action(Action),
    Git(GitRead),
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
    /// Open the cursor row's pane in a popup (`@cts_preview popup`).
    Preview,
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
    /// Read the branch and change summary of a directory; answered with [`Msg::Git`].
    Git(GitAsk),
    /// Show the tail of a pane in a tmux popup.
    PreviewPopup(PaneId),
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
        Msg::Loaded(snapshot) => loaded(model, snapshot),
        Msg::Tick { now, frame } => (
            Model {
                frame,
                now,
                ..model
            },
            vec![],
        ),
        Msg::Resized { height } => (Model { height, ..model }, vec![]),
        Msg::Clicked { y } => clicked(model, y),
        Msg::Action(action) => act(model, action),
        Msg::Git(read) => {
            let tops = model
                .tops
                .clone()
                .into_iter()
                .chain(read.tops.clone())
                .collect();
            let git = super::preview::git::answered(model.git.clone(), read);
            (Model { git, tops, ..model }, vec![])
        }
    }
}

fn loaded(model: Model, snapshot: Snapshot) -> (Model, Vec<Effect>) {
    let selection = selection::follow(model.selection.clone(), &snapshot, model.popup);
    preview::git(Model {
        snapshot,
        selection,
        ..model
    })
}

fn clicked(model: Model, y: u16) -> (Model, Vec<Effect>) {
    let end = model.list_end(model.height);
    let selected = model.selection.selected.as_ref();
    match rows::at(&model.snapshot.rows, selected, end, y) {
        Some(key) => activate(select(model, Some(key))),
        None => (model, vec![]),
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
        Action::Preview => {
            let effects = preview::popup(&model);
            (model, effects)
        }
        Action::Move(_) | Action::First | Action::Last | Action::Attention(_) => {
            let key = moved_to(&model, action);
            preview::git(select(model, key))
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
        Action::Activate | Action::Quit | Action::Reload | Action::Preview => None,
    }
}

fn select(model: Model, key: Option<RowKey>) -> Model {
    let selection = Selection {
        selected: key.or_else(|| model.selection.selected.clone()),
        synced: model.selection.synced.clone(),
    };
    Model { selection, ..model }
}

fn attention_keys(rows: &[Row]) -> Vec<RowKey> {
    rows.iter()
        .filter(|r| matches!(r, Row::Session(c) if c.record.state.needs_attention()))
        .filter_map(rows::key)
        .collect()
}

#[cfg(test)]
mod tests;
