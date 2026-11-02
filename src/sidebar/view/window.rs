//! Window rows: a window without Claude (`@cts_all_windows`), and the heading of a window in the tree.

use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

use crate::sidebar::update::Model;
use crate::text::truncate;
use crate::tmux::pane::Pane;

/// A window without Claude: `· 2:zsh` among flat rows, or a faint `2:zsh` at the window level of the
/// tree, where the column already tells it is a window.
pub fn line(model: &Model, pane: &Pane, width: usize, selected: bool) -> Line<'static> {
    let here = Some(pane.window_id) == model.snapshot.window;
    let tree = model.label.tree();
    let indent = model.indent.at(usize::from(tree));
    let (mark, color) = if tree {
        (Vec::new(), model.theme.dim)
    } else {
        let mark = vec![Span::styled("·", model.theme.dim), Span::raw(" ")];
        (mark, model.theme.muted)
    };
    let width = width.saturating_sub(indent.len() + 2 * usize::from(!tree));
    let label = truncate(&label(pane), width);
    let style = Style::new().fg(color);
    // With the cursor on the window you are in, the background is the cursor's, so the number tells.
    let name = if here && selected {
        number_first(&label, pane.window_index, style, model.theme.accent)
    } else {
        vec![Span::styled(label, style)]
    };
    Line::from([vec![Span::raw(indent)], mark, name].concat())
}

/// ` 2:mywork`, or ` 2` for a window tmux named: above the window's Claude sessions.
pub fn heading(model: &Model, pane: &Pane, width: usize) -> Line<'static> {
    let indent = model.indent.at(1);
    let width = width.saturating_sub(indent.len());
    Line::from(vec![
        Span::raw(indent),
        Span::styled(truncate(&heading_text(pane), width), model.theme.muted),
    ])
}

/// `2:` of `2:api` in the `accent` color over `style`, and the rest in `style`; all of it in `accent`
/// when it does not start with the window number.
pub(super) fn number_first(
    name: &str,
    window_index: u32,
    style: Style,
    accent: Color,
) -> Vec<Span<'static>> {
    let number = format!("{window_index}:");
    match name.strip_prefix(&number) {
        Some(rest) => vec![
            Span::styled(number, style.fg(accent)),
            Span::styled(rest.to_owned(), style),
        ],
        None => vec![Span::styled(name.to_owned(), style.fg(accent))],
    }
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
