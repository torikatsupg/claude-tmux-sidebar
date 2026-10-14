//! Development tasks, run as `cargo xtask <task>`.

mod ansi;
mod ci;
mod svg;

use std::process::ExitCode;

const USAGE: &str = "usage: cargo xtask <task>

  ci                         formatting, lints, tests and module size limits (what CI runs)
  ansi2svg W H X:Y:FILE...   render `tmux capture-pane -e` output of panes into one SVG on stdout";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.split_first() {
        Some((task, rest)) if task == "ci" && rest.is_empty() => ci::run(),
        Some((task, rest)) if task == "ansi2svg" => svg::run(rest),
        _ => Err(USAGE.to_owned()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
