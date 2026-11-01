//! A window without Claude (`@cts_all_windows`): a quiet `#I:#W` row in the name column.

use ratatui::text::{Line, Span};

use super::session::marker;
use crate::sidebar::update::Model;
use crate::text::truncate;
use crate::tmux::pane::Pane;

/// `▌· 2:zsh`; the bar marks the window you are in.
pub fn line(model: &Model, pane: &Pane, width: usize) -> Line<'static> {
    let here = Some(pane.window_id) == model.snapshot.window;
    Line::from(vec![
        marker(model, here),
        Span::styled("·", model.theme.dim),
        Span::raw(" "),
        Span::styled(
            truncate(&label(pane), width.saturating_sub(4)),
            model.theme.muted,
        ),
    ])
}

/// `#I:#W`, as tmux names the window.
pub(super) fn label(pane: &Pane) -> String {
    format!("{}:{}", pane.window_index, pane.window_name)
}
