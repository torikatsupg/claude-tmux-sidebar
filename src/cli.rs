//! Command-line parsing. tmux passes empty strings for missing format values, so empty means absent.

use crate::jump::Direction;
use crate::tmux::pane::{PaneId, WindowId};

pub const USAGE: &str = "usage: claude-tmux-sidebar <command>

tmux side:
  toggle [pane]              open/close the sidebar in every window (pane: where it was toggled from; gets the focus)
  on [pane] | off            open / close the sidebar
  ensure [window]            add the sidebar to a window if enabled and missing (tmux hooks)
  fix-width [window]         restore the sidebar width after a layout change (tmux hooks)
  popup [pane]               show the list in a popup instead (pane: where it was opened from)
  next|prev [pane] [client]  go to the next / previous done or waiting Claude

internal:
  sidebar                    the sidebar pane itself
  popup-ui [pane]            the list inside the popup
  hook                       Claude Code hook (payload on stdin)

  --version";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Toggle {
        origin: Option<PaneId>,
    },
    On {
        origin: Option<PaneId>,
    },
    Off,
    Ensure {
        window: Option<WindowId>,
    },
    FixWidth {
        window: Option<WindowId>,
    },
    Popup {
        origin: Option<PaneId>,
    },
    Jump {
        direction: Direction,
        from: Option<PaneId>,
        client: Option<String>,
    },
    Sidebar,
    PopupUi {
        origin: Option<PaneId>,
    },
    Hook,
    Version,
}

/// Parses the arguments after the program name.
pub fn parse(args: &[String]) -> Option<Command> {
    let arg = |i: usize| args.get(i).map(String::as_str).filter(|a| !a.is_empty());
    let pane = |i| arg(i).and_then(PaneId::parse);
    let window = |i| arg(i).and_then(WindowId::parse);
    let jump = |direction| Command::Jump {
        direction,
        from: pane(1),
        client: arg(2).map(str::to_owned),
    };
    Some(match arg(0)? {
        "toggle" => Command::Toggle { origin: pane(1) },
        "on" => Command::On { origin: pane(1) },
        "off" => Command::Off,
        "ensure" => Command::Ensure { window: window(1) },
        "fix-width" => Command::FixWidth { window: window(1) },
        "popup" => Command::Popup { origin: pane(1) },
        "next" => jump(Direction::Next),
        "prev" => jump(Direction::Prev),
        "sidebar" => Command::Sidebar,
        "popup-ui" => Command::PopupUi { origin: pane(1) },
        "hook" => Command::Hook,
        "--version" | "-V" => Command::Version,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(args: &[&str]) -> Option<Command> {
        parse(&args.iter().map(|&a| a.to_owned()).collect::<Vec<_>>())
    }

    #[test]
    fn reads_targets_and_treats_empty_as_absent() {
        assert_eq!(
            parsed(&["toggle", "%3"]),
            Some(Command::Toggle {
                origin: PaneId::parse("%3")
            })
        );
        assert_eq!(
            parsed(&["ensure", ""]),
            Some(Command::Ensure { window: None })
        );
        assert_eq!(
            parsed(&["next", "%1", "/dev/ttys001"]),
            Some(Command::Jump {
                direction: Direction::Next,
                from: PaneId::parse("%1"),
                client: Some("/dev/ttys001".to_owned())
            })
        );
    }

    #[test]
    fn rejects_unknown_commands() {
        assert_eq!(parsed(&["bogus"]), None);
        assert_eq!(parsed(&[]), None);
    }
}
