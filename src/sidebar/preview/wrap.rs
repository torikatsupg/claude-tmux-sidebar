//! Text folded to the sidebar's width between words and wide characters, never cut.

use unicode_width::UnicodeWidthChar;

use crate::text;

/// `line` in lines of at most `width` cells. A break drops the spaces at it; a word wider than a line is
/// split. An empty line stays one empty line.
pub fn wrap(line: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let (lines, last) = atoms(line)
        .iter()
        .fold((Vec::new(), String::new()), |(lines, current), atom| {
            place(lines, current, atom, width)
        });
    lines.into_iter().chain([last]).collect()
}

fn cells(c: char) -> usize {
    c.width().unwrap_or(0)
}

/// Runs of spaces, runs of narrow characters (words), and single wide characters.
fn atoms(line: &str) -> Vec<String> {
    let kind = |c: char| match c {
        ' ' => 0,
        c if cells(c) > 1 => 2,
        _ => 1,
    };
    line.chars().fold(Vec::<String>::new(), |mut atoms, c| {
        let joins = atoms
            .last()
            .and_then(|a| a.chars().last())
            .is_some_and(|prev| kind(prev) == kind(c) && kind(c) != 2);
        match atoms.last_mut() {
            Some(atom) if joins => atom.push(c),
            _ => atoms.push(c.to_string()),
        }
        atoms
    })
}

fn place(lines: Vec<String>, current: String, atom: &str, width: usize) -> (Vec<String>, String) {
    let fits = text::width(&current) + text::width(atom) <= width;
    let space = atom.starts_with(' ');
    if fits && !(space && current.is_empty() && !lines.is_empty()) {
        return (lines, current + atom);
    }
    let lines = if current.trim().is_empty() {
        lines
    } else {
        lines
            .into_iter()
            .chain([current.trim_end().to_owned()])
            .collect()
    };
    match () {
        () if space => (lines, String::new()),
        () if text::width(atom) <= width => (lines, atom.to_owned()),
        () => split(lines, atom, width),
    }
}

/// A word wider than a line: whole lines of it, and the rest to go on with.
fn split(lines: Vec<String>, word: &str, width: usize) -> (Vec<String>, String) {
    let chunks = word.chars().fold(vec![String::new()], |mut chunks, c| {
        match chunks.last_mut() {
            Some(chunk) if text::width(chunk) + cells(c) <= width => chunk.push(c),
            _ => chunks.push(c.to_string()),
        }
        chunks
    });
    let rest = chunks.last().cloned().unwrap_or_default();
    let full = chunks.len().saturating_sub(1);
    (
        lines
            .into_iter()
            .chain(chunks.into_iter().take(full))
            .collect(),
        rest,
    )
}
