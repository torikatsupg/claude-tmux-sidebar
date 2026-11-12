//! The settings a sidebar reads once, when it opens.

use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Instant;

use super::preview::{self, Preview};
use super::update::Msg;
use super::{Mode, indent::Indent, label::Label, snapshot, theme::Theme};
use crate::context::Context;
use crate::i18n::{self, Messages};
use crate::tmux;

/// Everything a step needs that never changes.
pub struct Env<'a> {
    pub ctx: &'a Context,
    pub scope: snapshot::Scope,
    pub help: bool,
    pub indent: Indent,
    pub theme: Theme,
    pub preview: Preview,
    pub messages: &'static Messages,
    /// `$HOME`, read once.
    pub home: String,
    /// Where effects that take time send their answers, and where the loop picks them up.
    pub answers: Sender<Msg>,
    pub answered: Receiver<Msg>,
    /// The spinner runs on the clock, not on the number of events.
    pub started: Instant,
}

impl<'a> Env<'a> {
    pub fn new(ctx: &'a Context, mode: Mode) -> Self {
        let (me, popup) = match mode {
            Mode::Sidebar => (ctx.pane, false),
            Mode::Popup { origin } => (origin, true),
        };
        let lang = tmux::global_option("@cts_lang");
        let (answers, answered) = mpsc::channel();
        Env {
            ctx,
            scope: snapshot::Scope {
                me,
                popup,
                all_windows: tmux::global_option("@cts_all_windows").as_deref() == Some("on"),
                label: Label::from_options(tmux::global_option),
            },
            help: tmux::global_option("@cts_help").as_deref() != Some("off"),
            indent: Indent::from_options(tmux::global_option),
            theme: Theme::from_options(tmux::global_option),
            preview: Preview {
                rule: preview::border_color(
                    tmux::global_window_option("pane-border-style").as_deref(),
                ),
                ..in_popup(Preview::from_options(tmux::global_option), popup)
            },
            messages: i18n::messages(lang.as_deref()),
            home: std::env::var("HOME").unwrap_or_default(),
            answers,
            answered,
            started: Instant::now(),
        }
    }

    /// The key that opens the preview popup, when that is the preview.
    pub fn popup_key(&self) -> Option<char> {
        (self.preview.mode == preview::Mode::Popup).then_some(self.preview.key)
    }
}

/// tmux opens no popup over another, so the list's own popup shows the preview below the list instead.
fn in_popup(preview: Preview, popup: bool) -> Preview {
    match preview.mode {
        preview::Mode::Popup if popup => Preview {
            mode: preview::Mode::Pane,
            ..preview
        },
        _ => preview,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preview(mode: preview::Mode) -> Preview {
        Preview {
            mode,
            ..Preview::default()
        }
    }

    #[test]
    fn the_list_popup_shows_a_popup_preview_below_the_list() {
        let popup = preview(preview::Mode::Popup);
        assert_eq!(in_popup(popup, true).mode, preview::Mode::Pane);
        assert_eq!(in_popup(popup, false).mode, preview::Mode::Popup);
        let off = preview(preview::Mode::Off);
        assert_eq!(in_popup(off, true).mode, preview::Mode::Off);
    }
}
