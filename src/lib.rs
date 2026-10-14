//! A tmux sidebar listing every Claude Code session with live status.
//!
//! Pure modules turn values into values; side effects live at the edges: [`tmux`] (the tmux server),
//! [`session::store`] (state files), [`process`] (other processes) and the `run` functions of
//! [`control`], [`hook`], [`jump`] and [`sidebar`].

pub mod cli;
pub mod context;
pub mod control;
pub mod hook;
pub mod i18n;
pub mod jump;
pub mod listing;
pub mod process;
pub mod session;
pub mod sidebar;
pub mod text;
pub mod tmux;

use std::io;
use std::process::ExitCode;

use cli::Command;
use context::Context;
use sidebar::Mode;

/// Runs one command and reports how it went.
pub fn run(command: Command, ctx: &Context) -> ExitCode {
    match command {
        Command::Toggle { origin } => control::toggle(ctx, origin),
        Command::On { origin } => control::on(ctx, origin),
        Command::Off => control::off(),
        Command::Ensure { window } => control::ensure(ctx, window),
        Command::FixWidth { window } => control::fix_width(window),
        Command::Popup { origin } => control::popup(ctx, origin),
        Command::Jump {
            direction,
            from,
            client,
        } => jump::run(ctx, direction, from, client.as_deref()),
        Command::Hook => hook::run(ctx, &read_stdin()),
        Command::Version => print_version(),
        Command::Sidebar => return interactive(ctx, Mode::Sidebar),
        Command::PopupUi { origin } => return interactive(ctx, Mode::Popup { origin }),
    }
    ExitCode::SUCCESS
}

fn interactive(ctx: &Context, mode: Mode) -> ExitCode {
    match sidebar::run(ctx, mode) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("claude-tmux-sidebar: {error}");
            ExitCode::FAILURE
        }
    }
}

fn read_stdin() -> String {
    io::read_to_string(io::stdin()).unwrap_or_default()
}

#[expect(
    clippy::print_stdout,
    reason = "--version prints to stdout by convention"
)]
fn print_version() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
