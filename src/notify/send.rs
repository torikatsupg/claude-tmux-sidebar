//! Sends the notifications: the side effects of `@cts_notify`.

use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::process::{Command, Stdio};

use super::place::{self, CLIENT_FORMAT, PLACE_FORMAT, Place};
use super::{Mode, os_command, text, tmux_text, worth};
use crate::context::Context;
use crate::i18n;
use crate::session::{Record, store};
use crate::tmux;

const BELL: &[u8] = b"\x07";

/// Opening the pane's tty must not make it this process's controlling terminal.
#[cfg(target_os = "macos")]
const O_NOCTTY: i32 = 0x2_0000;
#[cfg(target_os = "linux")]
const O_NOCTTY: i32 = 0o400;
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
const O_NOCTTY: i32 = 0;

/// Notifies about the hook's change from `prev` to `record`, unless the user already looks at its pane.
/// The mode comes from a file, so with `off` the hook never calls tmux.
pub fn run(ctx: &Context, prev: Option<&Record>, record: &Record, by_notification: bool) {
    store::clear_alerts(&ctx.state_dir, &record.session_id, record.state);
    let Some(pane) = ctx.pane.filter(|_| worth(prev, record, by_notification)) else {
        return;
    };
    let mode = Mode::read(&ctx.state_dir);
    if mode == Mode::Off || !store::claim_alert(&ctx.state_dir, &record.session_id, record.state) {
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
            .custom_flags(O_NOCTTY)
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
