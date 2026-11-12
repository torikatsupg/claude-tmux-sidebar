//! Runs the effects that [`update`](super::update::update) asks for.

use super::preview::{self, Captured};
use super::update::{Effect, Msg, Target};
use crate::context::Context;
use crate::text::shell_quote;
use crate::tmux::{self, pane::PaneId};

/// What the event loop does next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Next {
    Continue { reload: bool },
    Stop,
}

/// Runs `effects`, and returns what to do next with the messages that answer them.
pub fn perform(ctx: &Context, me: Option<PaneId>, effects: &[Effect]) -> (Next, Vec<Msg>) {
    let next = effects
        .iter()
        .fold(Next::Continue { reload: false }, |next, effect| {
            match (next, effect) {
                (Next::Stop, _) | (_, Effect::Exit) => Next::Stop,
                (_, Effect::Reload) => Next::Continue { reload: true },
                // A jump moves the focus; reload right away so the selection follows it.
                (_, goto @ Effect::Goto { .. }) => {
                    run(ctx, me, goto);
                    Next::Continue { reload: true }
                }
                (next, effect) => {
                    run(ctx, me, effect);
                    next
                }
            }
        });
    (next, effects.iter().filter_map(answer).collect())
}

fn answer(effect: &Effect) -> Option<Msg> {
    match effect {
        Effect::Capture(pane) => Some(captured(*pane, &tmux::capture(*pane))),
        _ => None,
    }
}

/// What a capture answers; a gone pane or a failed `capture-pane` prints nothing, which empties the preview.
fn captured(pane: PaneId, output: &str) -> Msg {
    Msg::Captured(Captured {
        pane,
        lines: preview::clean(output),
    })
}

fn run(ctx: &Context, me: Option<PaneId>, effect: &Effect) {
    match effect {
        Effect::Goto {
            target,
            leaves_window,
        } => goto(me, target, *leaves_window),
        // Run detached by tmux, since closing every sidebar kills this pane too.
        Effect::CloseAll => {
            let command = format!("{} off", shell_quote(&ctx.exe.to_string_lossy()));
            tmux::run(&["run-shell", "-b", &command]);
        }
        Effect::KillSelf => {
            if let Some(me) = me {
                tmux::run(&["kill-pane", "-t", &me.to_string()]);
            }
        }
        Effect::PreviewPopup(pane) => {
            let sizes = "#{pane_width} #{pane_height} #{client_width} #{client_height}";
            let (width, height) = preview::popup_size(&tmux::display(&pane.to_string(), sizes));
            tmux::preview_popup(*pane, &width, &height);
        }
        Effect::Reload | Effect::Exit | Effect::Capture(_) => {}
    }
}

fn goto(me: Option<PaneId>, target: &Target, leaves_window: bool) {
    match target {
        Target::Pane(pane) => tmux::goto(*pane, None),
        Target::Session(name) => {
            tmux::run(&["switch-client", "-t", &format!("={name}")]);
        }
    }
    // Hand this window's focus back to the pane the user was in, so it isn't on the sidebar when they return.
    let me = me.map(|p| p.to_string()).filter(|_| leaves_window);
    if let Some(me) = me.filter(|id| tmux::display(id, "#{pane_active}") == "1") {
        tmux::run(&["last-pane", "-t", &me]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmux::pane::fixture::pane;

    #[test]
    fn a_failed_capture_leaves_the_preview_empty() {
        let id = pane(1, 0).id;
        assert_eq!(
            captured(id, ""),
            Msg::Captured(Captured {
                pane: id,
                lines: vec![]
            })
        );
    }

    #[test]
    fn odd_bytes_keep_the_rest_of_the_capture() {
        let id = pane(1, 0).id;
        let output = String::from_utf8_lossy(b"ok \xff\nnext");
        assert_eq!(
            captured(id, &output),
            Msg::Captured(Captured {
                pane: id,
                lines: vec!["ok \u{fffd}".to_owned(), "next".to_owned()]
            })
        );
    }
}
