//! Jumping to the selected row.

use super::{Effect, Model, Target};
use crate::sidebar::rows::{self, RowKey};
use crate::sidebar::snapshot::Snapshot;
use crate::tmux::pane::WindowId;

/// Jumps to the selected row; a popup closes after a successful jump.
pub fn activate(model: Model) -> (Model, Vec<Effect>) {
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
        RowKey::Claude(_) | RowKey::Window(_) => snapshot
            .rows
            .iter()
            .filter(|r| rows::key(r).as_ref() == Some(key))
            .find_map(rows::pane)
            .map(|p| (Target::Pane(p.id), Some(p.window_id))),
    }
}
