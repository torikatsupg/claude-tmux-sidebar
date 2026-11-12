//! Runs the effects that [`update`](super::update::update) asks for.

use std::process::{Command, Stdio};

use super::preview::{self, GitRead};
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
        Effect::Git(dir) => Some(read(
            dir,
            git(dir, &["rev-parse", "--abbrev-ref", "HEAD"]),
            git(dir, &["diff", "--shortstat", "HEAD"]).as_deref(),
        )),
        _ => None,
    }
}

/// `git -C DIR ARGS`, `None` when it fails (not a repository, no commits yet). It takes no optional
/// locks, so that it never gets in the way of the user's own git.
fn git(dir: &str, args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(["--no-optional-locks", "-C", dir])
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
}

/// What git answered about `dir`; nothing outside a repository.
fn read(dir: &str, branch: Option<String>, stat: Option<&str>) -> Msg {
    Msg::Git(GitRead {
        dir: dir.to_owned(),
        branch: branch.unwrap_or_default(),
        changes: stat.and_then(preview::shortstat),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sidebar::preview::Changes;

    #[test]
    fn outside_a_repository_git_answers_nothing() {
        assert_eq!(
            read("/tmp", None, None),
            Msg::Git(GitRead {
                dir: "/tmp".to_owned(),
                branch: String::new(),
                changes: None
            })
        );
    }

    #[test]
    fn a_repository_answers_its_branch_and_changes() {
        let stat = " 2 files changed, 5 insertions(+), 1 deletion(-)".to_owned();
        assert_eq!(
            read("/src", Some("main".to_owned()), Some(&stat)),
            Msg::Git(GitRead {
                dir: "/src".to_owned(),
                branch: "main".to_owned(),
                changes: Some(Changes {
                    files: 2,
                    added: 5,
                    removed: 1
                })
            })
        );
    }
}
