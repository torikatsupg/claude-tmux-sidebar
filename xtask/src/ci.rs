//! `cargo xtask ci`: every check CI runs, in one command.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Rust files longer than this should be split into smaller modules.
const MAX_MODULE_LINES: usize = 200;
const RUST_DIRS: [&str; 3] = ["src", "tests", "xtask/src"];
const SHELL_FILES: [&str; 4] = ["bin/cts", "bin/cts-hook", "sidebar.tmux", "install.sh"];
const SHELL_DIR: &str = "scripts";

pub fn run() -> Result<(), String> {
    let shell = shell_files();
    let shell: Vec<&str> = shell.iter().map(String::as_str).collect();
    let steps: [(&str, Vec<&str>); 5] = [
        ("cargo", vec!["fmt", "--all", "--check"]),
        (
            "cargo",
            vec![
                "clippy",
                "--workspace",
                "--all-targets",
                "--locked",
                "--",
                "-D",
                "warnings",
            ],
        ),
        ("cargo", vec!["test", "--workspace", "--locked"]),
        ("shellcheck", shell.clone()),
        ("shfmt", [vec!["--diff"], shell].concat()),
    ];
    steps
        .iter()
        .try_for_each(|(program, args)| step(program, args))?;
    module_sizes()
}

fn step(program: &str, args: &[&str]) -> Result<(), String> {
    eprintln!("==> {program} {}", args.join(" "));
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|e| format!("{program}: {e}"))?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| format!("{program} failed ({status})"))
}

fn shell_files() -> Vec<String> {
    let scripts = fs::read_dir(SHELL_DIR)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "sh"))
        .map(|p| p.to_string_lossy().into_owned());
    SHELL_FILES
        .iter()
        .map(|&f| f.to_owned())
        .chain(scripts)
        .collect()
}

fn module_sizes() -> Result<(), String> {
    eprintln!("==> module sizes (at most {MAX_MODULE_LINES} lines)");
    let too_long: Vec<String> = RUST_DIRS
        .iter()
        .flat_map(|dir| rust_files(Path::new(dir)))
        .filter_map(|path| {
            let lines = fs::read_to_string(&path).map_or(0, |s| s.lines().count());
            (lines > MAX_MODULE_LINES).then(|| format!("  {}: {lines} lines", path.display()))
        })
        .collect();
    if too_long.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "modules over {MAX_MODULE_LINES} lines; split them:\n{}",
            too_long.join("\n")
        ))
    }
}

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .flat_map(|p| {
            if p.is_dir() {
                rust_files(&p)
            } else {
                p.extension()
                    .is_some_and(|e| e == "rs")
                    .then_some(p)
                    .into_iter()
                    .collect()
            }
        })
        .collect()
}
