//! The process environment, read once at startup so the rest of the crate never touches `std::env`.

use std::env;
use std::path::PathBuf;

use crate::tmux::pane::PaneId;

const APP_DIR: &str = "claude-tmux-sidebar";

/// Where this process runs and where the state files live.
#[derive(Clone, Debug)]
pub struct Context {
    pub state_dir: PathBuf,
    /// The pane this process runs in (`TMUX_PANE`).
    pub pane: Option<PaneId>,
    /// The tmux server socket (`TMUX` up to the first comma); empty outside tmux.
    pub socket: String,
    /// This executable, for commands that tmux runs later.
    pub exe: PathBuf,
    /// `CTS_DEBUG` is set: the hook logs raw payloads.
    pub debug: bool,
}

impl Context {
    pub fn from_env() -> Self {
        let var = |name: &str| env::var(name).ok().filter(|v| !v.is_empty());
        Self {
            state_dir: var("CTS_STATE_DIR")
                .map_or_else(|| cache_dir(var).join(APP_DIR), PathBuf::from),
            pane: var("TMUX_PANE").and_then(|p| PaneId::parse(&p)),
            socket: var("TMUX")
                .and_then(|t| t.split(',').next().map(str::to_owned))
                .unwrap_or_default(),
            exe: env::current_exe().unwrap_or_else(|_| PathBuf::from(APP_DIR)),
            debug: var("CTS_DEBUG").is_some(),
        }
    }
}

fn cache_dir(var: impl Fn(&str) -> Option<String>) -> PathBuf {
    var("XDG_CACHE_HOME").map_or_else(
        || PathBuf::from(var("HOME").unwrap_or_default()).join(".cache"),
        PathBuf::from,
    )
}

/// Seconds since the Unix epoch.
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}
