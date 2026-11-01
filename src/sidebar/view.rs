//! Rendering: the model becomes lines of styled text. Only [`render`] touches the terminal frame.

use ratatui::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::rows::{self, Row};
use super::theme::Theme;
use super::update::Model;
use crate::i18n::Messages;
use crate::listing::GroupName;
use crate::session::Status;

mod session;
mod window;

const TITLE: &str = "Claude";
/// Statuses counted in the title bar, in order.
const COUNTED: [Status; 3] = [Status::Running, Status::Waiting, Status::Done];

pub fn render(model: &Model, messages: &Messages, frame: &mut Frame) {
    let area = frame.area();
    frame.render_widget(
        Paragraph::new(lines(model, messages, area.width, area.height)),
        area,
    );
}

/// The whole screen, one line per terminal row.
pub fn lines(model: &Model, messages: &Messages, width: u16, height: u16) -> Vec<Line<'static>> {
    let width = usize::from(width);
    let end = model.list_end(height);
    let theme = &model.theme;
    let body: Vec<Line<'static>> = if model.snapshot.rows.is_empty() {
        vec![Line::styled(
            format!(" {}", messages.no_sessions),
            theme.muted,
        )]
    } else {
        rows::placed(&model.snapshot.rows, end)
            .map(|(_, row)| row_line(model, messages, row, width))
            .collect()
    };
    let filler = usize::from(end.saturating_sub(rows::FIRST_ROW)).saturating_sub(body.len());
    [title(model), Line::styled("─".repeat(width), theme.dim)]
        .into_iter()
        .chain(body)
        .chain(std::iter::repeat_n(Line::default(), filler))
        .chain(model.help.then(|| help(messages, theme)))
        .collect()
}

/// ` ⏎ jump  j/k move  q close`: bold keys, muted labels.
fn help(messages: &Messages, theme: &Theme) -> Line<'static> {
    let spans = messages
        .help
        .iter()
        .enumerate()
        .flat_map(|(i, (key, label))| {
            let gap = if i == 0 { " " } else { "  " };
            [
                Span::raw(gap),
                Span::styled(*key, Modifier::BOLD),
                Span::raw(" "),
                Span::styled(*label, theme.muted),
            ]
        });
    Line::from(spans.collect::<Vec<_>>())
}

fn title(model: &Model) -> Line<'static> {
    let count = |status: Status| {
        model
            .snapshot
            .rows
            .iter()
            .filter(|r| matches!(r, Row::Session(c) if c.record.state == status))
            .count()
    };
    let counts = COUNTED.into_iter().filter_map(|status| {
        let (icon, color) = session::look(&model.theme, status);
        Some(count(status))
            .filter(|&n| n > 0)
            .map(|n| Span::styled(format!("{icon}{n} "), color))
    });
    let title = Span::styled(
        TITLE,
        Style::new()
            .fg(model.theme.accent)
            .add_modifier(Modifier::BOLD),
    );
    Line::from(
        [Span::raw(" "), title, Span::raw(" ")]
            .into_iter()
            .chain(counts)
            .collect::<Vec<_>>(),
    )
}

fn row_line(model: &Model, messages: &Messages, row: &Row, width: usize) -> Line<'static> {
    let selected = rows::owner(row).is_some_and(|k| model.selection.selected.as_ref() == Some(&k));
    let line = match row {
        Row::Gap => Line::default(),
        Row::Empty(_) => Line::styled(
            format!("   {}", messages.none),
            Style::new()
                .fg(model.theme.muted)
                .add_modifier(Modifier::ITALIC),
        ),
        Row::Heading(name) => heading(name, messages),
        Row::Session(claude) => session::line(model, claude, width),
        Row::Detail(claude) => session::detail(model, claude, width),
        Row::Window(pane) => window::line(model, pane, width),
    };
    if selected {
        highlight(line, width, model.theme.cursor)
    } else {
        line
    }
}

fn heading(name: &GroupName, messages: &Messages) -> Line<'static> {
    let text = match name {
        GroupName::Session(s) => s.as_str(),
        GroupName::Outside => messages.outside,
    };
    Line::from(vec![
        Span::raw(" "),
        Span::styled(format!("▾ {text}"), Modifier::BOLD),
    ])
}

/// The cursor: a background across the full width, under the row's own colors.
fn highlight(line: Line<'static>, width: usize, cursor: Color) -> Line<'static> {
    let pad = " ".repeat(width.saturating_sub(line.width()));
    let spans = line
        .spans
        .into_iter()
        .chain([Span::raw(pad)])
        .map(|span| span.patch_style(Style::new().bg(cursor)))
        .collect::<Vec<_>>();
    Line::from(spans)
}

#[cfg(test)]
mod tests;
