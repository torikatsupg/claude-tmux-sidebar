//! How far each level of the tree is indented, chosen by the `@cts_indent` tmux option.

const OPTION: &str = "@cts_indent";
const DEFAULT_SPACES: usize = 1;
const MAX_SPACES: usize = 4;

/// Spaces per level of the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Indent(usize);

impl Default for Indent {
    fn default() -> Self {
        Self(DEFAULT_SPACES)
    }
}

impl Indent {
    /// The indent the option looked up with `option` asks for.
    pub fn from_options(option: impl Fn(&str) -> Option<String>) -> Self {
        Self::parse(option(OPTION).as_deref())
    }

    /// `0` to `4` spaces; the default for anything else or unset.
    pub fn parse(value: Option<&str>) -> Self {
        value
            .and_then(|v| v.trim().parse().ok())
            .filter(|&spaces| spaces <= MAX_SPACES)
            .map_or_else(Self::default, Self)
    }

    /// The spaces in front of a row `depth` levels deep.
    pub fn at(self, depth: usize) -> String {
        " ".repeat(self.0 * depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unset_is_one_space() {
        assert_eq!(Indent::from_options(|_| None).at(2), "  ");
    }

    #[test]
    fn zero_to_four_spaces_are_taken() {
        assert_eq!(Indent::parse(Some("0")).at(2), "");
        assert_eq!(Indent::parse(Some("2")).at(2), "    ");
        assert_eq!(Indent::parse(Some(" 4 ")).at(1), "    ");
    }

    #[test]
    fn values_out_of_range_fall_back_to_one_space() {
        for value in ["5", "-1", "two", "", "1.5"] {
            assert_eq!(Indent::parse(Some(value)), Indent::default(), "{value}");
        }
    }

    #[test]
    fn the_option_is_read_by_its_name() {
        let indent = Indent::from_options(|name| (name == "@cts_indent").then(|| "3".to_owned()));
        assert_eq!(indent.at(1), "   ");
    }
}
