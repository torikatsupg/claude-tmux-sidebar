//! Window rows: a window without Claude (`@cts_all_windows`), and the heading of a window in the tree.

use ratatui::text::{Line, Span};

use super::session::marker;
use crate::sidebar::update::Model;
use crate::text::truncate;
use crate::tmux::pane::Pane;

/// A window without Claude: `▌· 2:zsh` among flat rows, or `▌  2:zsh` faint at the window level of the
/// tree; the bar marks the window you are in.
pub fn line(model: &Model, pane: &Pane, width: usize) -> Line<'static> {
    let here = Some(pane.window_id) == model.snapshot.window;
    if model.label.tree() {
        return Line::from(vec![
            marker(model, here),
            Span::raw("  "),
            Span::styled(
                truncate(&label(pane), width.saturating_sub(3)),
                model.theme.dim,
            ),
        ]);
    }
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

/// `▌  2:mywork`, or `▌  2` for a window tmux named: above the window's Claude sessions.
pub fn heading(model: &Model, pane: &Pane, width: usize) -> Line<'static> {
    let here = Some(pane.window_id) == model.snapshot.window;
    Line::from(vec![
        marker(model, here),
        Span::raw("  "),
        Span::styled(
            truncate(&heading_text(pane), width.saturating_sub(3)),
            model.theme.muted,
        ),
    ])
}

/// Only a name set by hand says anything; tmux names a Claude Code window after its version.
fn heading_text(pane: &Pane) -> String {
    if pane.window_named {
        label(pane)
    } else {
        pane.window_index.to_string()
    }
}

/// `#I:#W`, as tmux names the window.
pub(super) fn label(pane: &Pane) -> String {
    format!("{}:{}", pane.window_index, pane.window_name)
}
