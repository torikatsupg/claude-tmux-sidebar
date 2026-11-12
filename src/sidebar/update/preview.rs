//! What the preview asks of tmux as the cursor moves.

use super::{Effect, Model};
use crate::sidebar::preview::{self, GitAsk, Mode, git};
use crate::sidebar::rows::Row;

/// A read of the preview's directory with `git`, at most every [`git::INTERVAL`] seconds and right away
/// for a new directory, only while the preview is drawn.
pub fn git(model: Model) -> (Model, Vec<Effect>) {
    let selected = model.selection.selected.as_ref();
    let dir = preview::directory(&model.snapshot, selected)
        .filter(|_| model.preview.mode == Mode::Pane)
        .filter(|_| model.preview_height(model.height) > 0)
        .filter(|dir| git::due(model.git.as_ref(), dir, model.now));
    match dir {
        Some(dir) => {
            let asked = git::asked(model.git.clone(), &dir, model.now);
            let others = others(&model, &dir);
            let model = Model {
                git: Some(asked),
                ..model
            };
            (model, vec![Effect::Git(GitAsk { dir, others })])
        }
        None => (model, vec![]),
    }
}

/// The other sessions' directories whose repository is not known yet, to tell whether their changes
/// overlap.
fn others(model: &Model, dir: &str) -> Vec<String> {
    let dirs = model.snapshot.rows.iter().filter_map(|row| match row {
        Row::Session(c) => Some(c.record.cwd.clone()),
        _ => None,
    });
    let mut others: Vec<String> = dirs
        .filter(|d| !d.is_empty() && d != dir && !model.tops.contains_key(d))
        .collect();
    others.sort();
    others.dedup();
    others
}

/// The cursor row's pane in a tmux popup.
pub fn popup(model: &Model) -> Vec<Effect> {
    let selected = model.selection.selected.as_ref();
    preview::target(&model.snapshot, selected)
        .filter(|_| model.preview.mode == Mode::Popup)
        .map(Effect::PreviewPopup)
        .into_iter()
        .collect()
}
