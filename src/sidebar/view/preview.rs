//! The preview below the list: a rule naming the cursor row, then the end of what it last showed.

use ratatui::text::{Line, Span};

use super::{session_name, window};
use crate::sidebar::preview::{self, Subject};
use crate::sidebar::update::Model;
use crate::text::{self, truncate};

/// `height` lines: nothing when the preview is off, unfocused, or the pane is too short for it.
pub fn block(model: &Model, width: usize, height: u16) -> Vec<Line<'static>> {
    let Some(room) = usize::from(model.preview_height(height)).checked_sub(1) else {
        return Vec::new();
    };
    let subject = preview::subject(&model.snapshot, model.selection.selected.as_ref());
    let wrapped: Vec<String> = subject
        .map(|s| body(model, s))
        .unwrap_or_default()
        .iter()
        .flat_map(|line| preview::wrap(line, width))
        .collect();
    let tail = wrapped.iter().skip(wrapped.len().saturating_sub(room));
    let body: Vec<Line<'static>> = tail.map(|l| Line::raw(l.clone())).collect();
    let filler = room.saturating_sub(body.len());
    [rule(model, subject, width)]
        .into_iter()
        .chain(body)
        .chain(std::iter::repeat_n(Line::default(), filler))
        .collect()
}

/// A Claude row's last words, or the screen of a window's pane while its capture is current.
fn body(model: &Model, subject: Subject<'_>) -> Vec<String> {
    match subject {
        Subject::Claude(claude) => preview::said(claude),
        Subject::Window(pane) => model
            .captured
            .as_ref()
            .filter(|c| c.pane == pane.id)
            .map(|c| c.lines.clone())
            .unwrap_or_default(),
    }
}

/// `─ session/name ───`, in the color of tmux's pane borders; the name is cut to leave the line's ends.
fn rule(model: &Model, subject: Option<Subject<'_>>, width: usize) -> Line<'static> {
    let color = model.preview.rule.unwrap_or(model.theme.dim);
    let name = subject.map(|s| name(s)).unwrap_or_default();
    let head = if name.is_empty() {
        String::new()
    } else {
        format!("─ {} ", truncate(&name, width.saturating_sub(4)))
    };
    let rest = "─".repeat(width.saturating_sub(text::width(&head)));
    Line::from(Span::styled(
        truncate(&format!("{head}{rest}"), width),
        color,
    ))
}

/// The row's name as the list writes it, after its tmux session.
fn name(subject: Subject<'_>) -> String {
    let (session, name) = match subject {
        Subject::Claude(claude) => (
            claude.pane.as_ref().map(|p| p.session.as_str()),
            session_name::name(claude, true),
        ),
        Subject::Window(pane) => (Some(pane.session.as_str()), window::label(pane)),
    };
    session.map_or_else(|| name.clone(), |s| format!("{s}/{name}"))
}
