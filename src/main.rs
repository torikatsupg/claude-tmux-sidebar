use std::process::ExitCode;

use claude_tmux_sidebar::cli::{self, USAGE};
use claude_tmux_sidebar::context::Context;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = cli::parse(&args) else {
        eprintln!("{USAGE}");
        return ExitCode::FAILURE;
    };
    claude_tmux_sidebar::run(command, &Context::from_env())
}
