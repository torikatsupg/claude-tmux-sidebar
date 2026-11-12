//! What the first line of a Claude session calls it: the directory and branch, or the tmux window.

use crate::session::live::Claude;
use crate::sidebar::label::Label;
use crate::sidebar::update::Model;
use crate::text::{self, truncate};

/// A branch squeezed below this many cells is dropped rather than cut to a stub.
const MIN_BRANCH_WIDTH: usize = 4;

pub(super) fn under_heading(model: &Model, claude: &Claude) -> bool {
    let pane = claude.pane.as_ref();
    pane.is_some_and(|p| model.label.window_heading(p))
}

/// The name and the muted text after it, cut to `room` cells together.
pub(super) fn label(model: &Model, claude: &Claude, room: usize) -> (String, String) {
    match model.label {
        Label::Window => (truncate(&window(claude), room), String::new()),
        Label::Dir | Label::Auto(_) => {
            let numbered = !model.label.tree();
            let name = truncate(&name(claude, numbered), room);
            let branch = branch(claude.branch(), room.saturating_sub(text::width(&name)));
            (name, branch)
        }
    }
}

/// ` main` after the name, cut to `room` cells; empty when there is no branch or no room for it.
fn branch(branch: &str, room: usize) -> String {
    if branch.is_empty() || room < MIN_BRANCH_WIDTH + 1 {
        return String::new();
    }
    format!(" {}", truncate(branch, room - 1))
}

/// `window:directory` when the tmux session has several windows and `numbered`, otherwise just the
/// directory.
pub(super) fn name(claude: &Claude, numbered: bool) -> String {
    let dir = std::path::Path::new(&claude.record.cwd)
        .file_name()
        .map_or_else(|| "?".to_owned(), |d| d.to_string_lossy().into_owned());
    claude
        .pane
        .as_ref()
        .filter(|p| numbered && p.session_windows > 1)
        .map_or_else(|| dir.clone(), |p| format!("{}:{dir}", p.window_index))
}

/// `#I:#W` of the session's window; the directory outside tmux.
fn window(claude: &Claude) -> String {
    claude
        .pane
        .as_ref()
        .map_or_else(|| name(claude, true), crate::sidebar::view::window::label)
}
