//! Parsing `tmux capture-pane -e` output: text with SGR escape sequences, into styled cells.

mod sgr;

use unicode_width::UnicodeWidthChar;

pub use sgr::Style;
use sgr::apply;

/// One character on screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub style: Style,
    /// Cells it covers (2 for wide characters).
    pub width: usize,
}

/// The cells of each line, with the column each starts at. The style carries over line ends: tmux
/// does not repeat an attribute that is still in effect at the start of the next line.
pub fn lines(text: &str) -> Vec<Vec<(usize, Cell)>> {
    let start = (Style::default(), Vec::new());
    let (_, lines) = text.split('\n').fold(start, |(style, lines), text| {
        let (style, cells) = line(style, text);
        (style, [lines, vec![cells]].concat())
    });
    lines
}

/// The cells of one line drawn from `style`, and the style in effect at its end.
fn line(style: Style, text: &str) -> (Style, Vec<(usize, Cell)>) {
    let parts: Vec<&str> = text.split("\x1b[").collect();
    let Some((first, sequences)) = parts.split_first() else {
        return (style, Vec::new());
    };
    let chunks = std::iter::once((Vec::new(), *first)).chain(sequences.iter().map(|s| sgr(s)));
    let (style, _, cells) = chunks.fold(
        (style, 0, Vec::new()),
        |(style, x, cells), (codes, text)| {
            let style = apply(style, &codes);
            let placed = place(text, &style, x);
            let x = placed.last().map_or(x, |(at, cell)| at + cell.width);
            (style, x, [cells, placed].concat())
        },
    );
    (style, cells)
}

/// The cells of `text` drawn from column `x`.
fn place(text: &str, style: &Style, x: usize) -> Vec<(usize, Cell)> {
    let widths = text.chars().map(|ch| ch.width().unwrap_or(0).max(1));
    // Prefix sums of the widths: the column of each character.
    let columns = widths.clone().scan(x, |column, width| {
        let at = *column;
        *column += width;
        Some(at)
    });
    text.chars()
        .zip(widths)
        .zip(columns)
        .map(|((ch, width), at)| {
            (
                at,
                Cell {
                    ch,
                    style: style.clone(),
                    width,
                },
            )
        })
        .collect()
}

/// Splits `1;31mtext` into its codes and the text after them.
fn sgr(chunk: &str) -> (Vec<u16>, &str) {
    chunk
        .split_once('m')
        .map_or((Vec::new(), chunk), |(codes, text)| {
            (
                codes.split(';').map(|c| c.parse().unwrap_or(0)).collect(),
                text,
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styles_follow_escape_codes() {
        let (_, cells) = line(Style::default(), "a\x1b[1;32mb\x1b[0mc");
        let styles: Vec<(usize, char, bool, Option<&str>)> = cells
            .iter()
            .map(|(x, c)| (*x, c.ch, c.style.bold, c.style.fg.as_deref()))
            .collect();
        assert_eq!(
            styles,
            [
                (0, 'a', false, None),
                (1, 'b', true, Some("#81b29a")),
                (2, 'c', false, None)
            ]
        );
    }

    #[test]
    fn wide_characters_take_two_columns() {
        let (_, cells) = line(Style::default(), "承x");
        let columns: Vec<usize> = cells.iter().map(|(x, _)| *x).collect();
        assert_eq!(columns, [0, 2]);
    }

    #[test]
    fn styles_carry_over_line_ends() {
        let lines = lines("\x1b[1ma\nb\x1b[0m\nc");
        let bold: Vec<Vec<bool>> = lines
            .iter()
            .map(|cells| cells.iter().map(|(_, c)| c.style.bold).collect())
            .collect();
        assert_eq!(bold, [vec![true], vec![true], vec![false]]);
    }
}
