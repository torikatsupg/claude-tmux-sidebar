//! How a row and its window are named, chosen by the `@cts_label` and `@cts_single_window` tmux options.

use crate::tmux::pane::Pane;

const OPTION: &str = "@cts_label";
const SINGLE_WINDOW_OPTION: &str = "@cts_single_window";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Label {
    /// The directory and the git branch.
    #[default]
    Dir,
    /// The tmux window as `#I:#W`.
    Window,
    /// The directory and the git branch, under a heading for a window named by hand.
    Auto(SingleWindow),
}

/// The heading of a named window that is alone in its tmux session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SingleWindow {
    /// One line with the session: `work/mywork`.
    Inline,
    /// Its own line, as with several windows.
    Lines,
}

impl Label {
    /// The label the options looked up with `option` ask for.
    pub fn from_options(option: impl Fn(&str) -> Option<String>) -> Self {
        Self::parse(
            option(OPTION).as_deref(),
            option(SINGLE_WINDOW_OPTION).as_deref(),
        )
    }

    /// The label for the option values; the directory for anything unknown or unset.
    pub fn parse(label: Option<&str>, single_window: Option<&str>) -> Self {
        match label {
            Some("window") => Self::Window,
            Some("auto") if single_window == Some("lines") => Self::Auto(SingleWindow::Lines),
            Some("auto") => Self::Auto(SingleWindow::Inline),
            _ => Self::Dir,
        }
    }

    /// The pane's window gets a heading line, and its Claude rows go under it.
    pub fn window_heading(self, pane: &Pane) -> bool {
        match self {
            Self::Auto(single) => {
                pane.window_named && (pane.session_windows > 1 || single == SingleWindow::Lines)
            }
            Self::Dir | Self::Window => false,
        }
    }

    /// The pane's window joins the line of its tmux session.
    pub fn inline(self, pane: &Pane) -> bool {
        self == Self::Auto(SingleWindow::Inline) && pane.window_named && pane.session_windows == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tmux::pane::fixture::pane;

    fn named(session_windows: u32) -> Pane {
        Pane {
            window_named: true,
            session_windows,
            ..pane(1, 0)
        }
    }

    #[test]
    fn reads_the_options() {
        let inline = Label::Auto(SingleWindow::Inline);
        let lines = Label::Auto(SingleWindow::Lines);
        assert_eq!(Label::parse(None, None), Label::Dir);
        assert_eq!(Label::parse(Some("other"), Some("lines")), Label::Dir);
        assert_eq!(Label::parse(Some("window"), Some("lines")), Label::Window);
        assert_eq!(Label::parse(Some("auto"), None), inline);
        assert_eq!(Label::parse(Some("auto"), Some("other")), inline);
        assert_eq!(Label::parse(Some("auto"), Some("lines")), lines);
    }

    #[test]
    fn a_named_window_among_others_gets_its_own_heading() {
        for label in [SingleWindow::Inline, SingleWindow::Lines].map(Label::Auto) {
            assert!(label.window_heading(&named(2)));
            assert!(!label.inline(&named(2)));
        }
    }

    #[test]
    fn a_named_window_alone_goes_by_the_option() {
        let inline = Label::Auto(SingleWindow::Inline);
        let lines = Label::Auto(SingleWindow::Lines);
        assert!(inline.inline(&named(1)) && !inline.window_heading(&named(1)));
        assert!(!lines.inline(&named(1)) && lines.window_heading(&named(1)));
    }

    #[test]
    fn only_a_named_window_counts() {
        let label = Label::Auto(SingleWindow::Inline);
        let automatic = Pane {
            window_named: false,
            ..named(2)
        };
        assert!(!label.window_heading(&automatic) && !label.inline(&automatic));
        assert!(!Label::Dir.window_heading(&named(2)) && !Label::Window.inline(&named(1)));
    }
}
