//! What the first line of a session row names, chosen by the `@cts_label` tmux option.

pub const OPTION: &str = "@cts_label";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Label {
    /// The directory and the git branch.
    #[default]
    Dir,
    /// The tmux window as `#I:#W`.
    Window,
    /// The tmux window when it was named by hand, otherwise the directory and the git branch.
    Auto,
}

impl Label {
    /// The label for an option value; the directory for anything unknown or unset.
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("window") => Self::Window,
            Some("auto") => Self::Auto,
            _ => Self::Dir,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_option() {
        assert_eq!(Label::parse(Some("dir")), Label::Dir);
        assert_eq!(Label::parse(Some("window")), Label::Window);
        assert_eq!(Label::parse(Some("pane")), Label::Dir);
        assert_eq!(Label::parse(None), Label::Dir);
    }
}
