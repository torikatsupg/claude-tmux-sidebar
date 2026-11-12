//! What the preview asks of tmux as the cursor moves.

use super::{Effect, Model};
use crate::sidebar::preview::{self, Mode};

/// A fresh capture of a window row's pane, only while the preview below the list is drawn. Claude rows
/// are previewed from their transcript, which the list reload reads.
pub fn capture(model: &Model) -> Vec<Effect> {
    let selected = model.selection.selected.as_ref();
    preview::window_target(&model.snapshot, selected)
        .filter(|_| model.preview.mode == Mode::Pane)
        .filter(|_| model.preview_height(model.height) > 0)
        .map(Effect::Capture)
        .into_iter()
        .collect()
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
