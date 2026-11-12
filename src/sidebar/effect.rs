//! Runs the effects that [`update`](super::update::update) asks for.

pub mod git;

use std::sync::mpsc::Sender;

use super::preview::{self, GitAsk};
use super::update::{Effect, Target};
use crate::context::Context;
use crate::text::shell_quote;
use crate::tmux::{self, pane::PaneId};

/// What the event loop does next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Next {
    Continue { reload: bool },
    Stop,
}

/// Runs `effects`, and returns what to do next. git runs on its own thread and answers later.
pub fn perform(
    ctx: &Context,
    me: Option<PaneId>,
    git: &Sender<GitAsk>,
    effects: &[Effect],
) -> Next {
    effects
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
                (next, Effect::Git(ask)) => {
                    let _ = git.send(ask.clone());
                    next
                }
                (next, effect) => {
                    run(ctx, me, effect);
                    next
                }
            }
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
        Effect::Reload | Effect::Exit | Effect::Git(_) => {}
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
