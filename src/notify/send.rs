//! Sends the notifications: the side effects of `@cts_notify`.

use std::fs::OpenOptions;
use std::io::Write;
use std::process::{Command, Stdio};

use super::place::{self, CLIENT_FORMAT, PLACE_FORMAT, Place};
use super::{Mode, OPTION, os_command, text, tmux_text, worth};
use crate::context::Context;
use crate::i18n;
use crate::session::{Record, Status};
use crate::tmux;

const BELL: &[u8] = b"\x07";

/// Notifies about the hook's change from `prev` to `record`, unless the user already looks at its pane.
/// Nothing touches tmux before the change is known to be worth it, so `off` costs nothing.
pub fn run(ctx: &Context, prev: Option<Status>, record: &Record) {
    if !worth(prev, record.state) {
        return;
    }
    let Some(pane) = ctx.pane else {
        return;
    };
    let mode = Mode::parse(tmux::global_option(OPTION).as_deref());
    if mode == Mode::Off {
        return;
    }
    let Some(place) = place::parse_place(&tmux::display(&pane.to_string(), PLACE_FORMAT)) else {
        return;
    };
    let clients = place::parse_clients(&tmux::output(&["list-clients", "-F", CLIENT_FORMAT]));
    let watcher = place::watcher(&clients);
    if place::seen(&place, watcher) {
        return;
    }
    let lang = tmux::global_option("@cts_lang");
    let text = text(record.state, &place, i18n::messages(lang.as_deref()));
    if mode.tmux() {
        tmux::message(watcher.map(|c| c.name.as_str()), &tmux_text(&text));
        ring(&place);
    }
    if mode.os() {
        os(&text);
    }
}

/// Rings the bell in the pane's window, as if Claude had: tmux flags the window and passes the bell on.
fn ring(place: &Place) {
    if place.tty.starts_with("/dev/") {
        let _ = OpenOptions::new()
            .write(true)
            .open(&place.tty)
            .and_then(|mut tty| tty.write_all(BELL));
    }
}

/// Shows an OS notification without waiting for it, since the hook must not hold Claude up; nothing
/// happens when the command is missing.
fn os(text: &str) {
    let (program, args) = os_command(text, cfg!(target_os = "macos"));
    let _ = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}
