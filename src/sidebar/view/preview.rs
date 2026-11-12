//! The preview below the list: a rule naming the cursor row, then its status panel.

use std::collections::BTreeMap;

use ratatui::text::{Line, Span};

use super::{panel, session_name, window};
use crate::i18n::Messages;
use crate::sidebar::preview::{self, Context, Subject};
use crate::sidebar::rows::Row;
use crate::sidebar::update::Model;
use crate::text::{self, truncate};

/// `height` lines: nothing when the preview is off, unfocused, or the pane is too short for it.
pub fn block(model: &Model, messages: &Messages, width: usize, height: u16) -> Vec<Line<'static>> {
    let Some(room) = usize::from(model.preview_height(height)).checked_sub(1) else {
        return Vec::new();
    };
    let subject = preview::subject(&model.snapshot, model.selection.selected.as_ref());
    let body: Vec<Line<'static>> = subject
        .map(|s| preview::items(s, &context(model, s, width), room))
        .unwrap_or_default()
        .iter()
        .map(|item| panel::line(model, messages, item, width))
        .collect();
    let filler = room.saturating_sub(body.len());
    [rule(model, subject, width)]
        .into_iter()
        .chain(body)
        .chain(std::iter::repeat_n(Line::default(), filler))
        .collect()
}

/// The git answer while it is about the subject's directory, and whether another session shares it.
fn context<'a>(model: &'a Model, subject: Subject<'_>, width: usize) -> Context<'a> {
    let dir = preview::directory(&model.snapshot, model.selection.selected.as_ref());
    let git = model.git.as_ref().filter(|g| Some(&g.dir) == dir.as_ref());
    let shared = match subject {
        Subject::Claude(me) => model.snapshot.rows.iter().any(|row| {
            matches!(row, Row::Session(c) if c.record.session_id != me.record.session_id
                && same_tree(&model.tops, &c.record.cwd, &me.record.cwd))
        }),
        Subject::Window(_) => false,
    };
    Context {
        changes: git.and_then(|g| g.changes),
        branch: git.map_or("", |g| g.branch.as_str()),
        shared,
        now: model.now,
        width,
        home: &model.home,
    }
}

/// Two directories in the same repository (by the top directories git told), or, when either is not known
/// to be in one, the same directory.
fn same_tree(tops: &BTreeMap<String, String>, a: &str, b: &str) -> bool {
    let top = |dir: &str| tops.get(dir).filter(|t| !t.is_empty());
    match (top(a), top(b)) {
        (Some(x), Some(y)) => x == y,
        _ => !a.is_empty() && a == b,
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
