//! Terminal text helpers: display width, truncation, durations and shell quoting.

use unicode_width::UnicodeWidthStr;

const ELLIPSIS: char = '…';

/// Display width in terminal cells (wide CJK characters count as two).
pub fn width(s: &str) -> usize {
    s.width()
}

/// Cuts `s` to at most `max` cells, marking the cut with an ellipsis.
pub fn truncate(s: &str, max: usize) -> String {
    if width(s) <= max {
        return s.to_owned();
    }
    let Some(room) = max.checked_sub(1) else {
        return String::new();
    };
    let kept = s
        .char_indices()
        .map(|(start, c)| start + c.len_utf8())
        .map_while(|end| s.get(..end).filter(|prefix| width(prefix) <= room))
        .last()
        .unwrap_or_default();
    format!("{kept}{ELLIPSIS}")
}

/// Compact age such as `42s`, `5m`, `3h` or `2d`.
pub fn age(seconds: u64) -> String {
    const UNITS: [(u64, &str); 3] = [(86_400, "d"), (3_600, "h"), (60, "m")];
    UNITS
        .iter()
        .find(|&&(size, _)| seconds >= size)
        .map_or_else(
            || format!("{seconds}s"),
            |&(size, unit)| format!("{}{unit}", seconds / size),
        )
}

/// Quotes `s` as a single shell word.
pub fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_keeps_short_text() {
        assert_eq!(truncate("abc", 3), "abc");
    }

    #[test]
    fn truncate_marks_the_cut() {
        assert_eq!(truncate("abcdef", 4), "abc…");
        assert_eq!(truncate("abc", 0), "");
    }

    #[test]
    fn truncate_counts_wide_characters_twice() {
        assert_eq!(truncate("承認待ち", 5), "承認…");
    }

    #[test]
    fn age_picks_the_largest_unit() {
        assert_eq!(age(59), "59s");
        assert_eq!(age(60), "1m");
        assert_eq!(age(7_200), "2h");
        assert_eq!(age(172_800), "2d");
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("it's"), r"'it'\''s'");
    }
}
