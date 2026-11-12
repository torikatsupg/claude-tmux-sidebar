//! The status panel's lines, styled.

use ratatui::style::Style;
use ratatui::text::{Line, Span};

use super::session;
use crate::i18n::Messages;
use crate::session::Status;
use crate::sidebar::preview::{Changes, Group, Item};
use crate::sidebar::update::Model;
use crate::text::{self, age, truncate};

pub fn line(model: &Model, messages: &Messages, item: &Item, width: usize) -> Line<'static> {
    let theme = &model.theme;
    match item {
        Item::Lead { status, text } => {
            let text = Some(text.as_str())
                .filter(|t| !t.is_empty())
                .unwrap_or(word(messages, *status));
            lead(session::icon(model, *status), text, width)
        }
        Item::More(text) => Line::raw(truncate(&format!("  {text}"), width)),
        Item::Count { group, count } => {
            let label = match group {
                Group::Agents => messages.agents,
                Group::Background => messages.background,
            };
            lead(
                session::icon(model, Status::Running),
                &format!("{label} {count}"),
                width,
            )
        }
        Item::Task { .. } => task(model, item, width),
        Item::Changes { changes, shared } => self::changes(model, messages, *changes, *shared),
        Item::Text(text) => Line::raw(truncate(text, width)),
        Item::Muted(text) => Line::styled(truncate(text, width), theme.muted),
    }
}

fn word(messages: &Messages, status: Status) -> &'static str {
    match status {
        Status::Waiting => messages.waiting,
        _ => messages.running,
    }
}

fn lead(icon: Span<'static>, text: &str, width: usize) -> Line<'static> {
    Line::from(vec![
        icon,
        Span::raw(format!(" {}", truncate(text, width.saturating_sub(2)))),
    ])
}

/// `  Explore  docsを調査          1m `: the age lines up with the list's ages.
fn task(model: &Model, item: &Item, width: usize) -> Line<'static> {
    let Item::Task {
        name,
        name_width,
        text,
        age,
    } = item
    else {
        return Line::default();
    };
    let right = format!("{} ", self::age(*age));
    let name = if name.is_empty() {
        String::new()
    } else {
        let name = truncate(name, *name_width);
        let pad = " ".repeat(name_width.saturating_sub(text::width(&name)));
        format!("{name}{pad}  ")
    };
    let used = 2 + text::width(&name) + text::width(&right);
    let text = truncate(text, width.saturating_sub(used + 1));
    let gap = width.saturating_sub(used + text::width(&text));
    Line::from(vec![
        Span::raw(format!("  {name}")),
        Span::raw(text),
        Span::raw(" ".repeat(gap)),
        Span::styled(right, model.theme.muted),
    ])
}

/// `± +128 -14 (3 files)`, with `, shared` when another session works in the same directory.
fn changes(model: &Model, messages: &Messages, changes: Changes, shared: bool) -> Line<'static> {
    let theme = &model.theme;
    let unit = if changes.files == 1 {
        messages.file
    } else {
        messages.files
    };
    let shared = if shared {
        format!(", {}", messages.shared)
    } else {
        String::new()
    };
    Line::from(vec![
        Span::styled("± ", theme.muted),
        Span::styled(
            format!("+{}", changes.added),
            Style::new().fg(theme.running),
        ),
        Span::raw(" "),
        Span::styled(
            format!("-{}", changes.removed),
            Style::new().fg(theme.critical),
        ),
        Span::styled(format!(" ({} {unit}{shared})", changes.files), theme.muted),
    ])
}
