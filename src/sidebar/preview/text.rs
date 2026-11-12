//! `capture-pane` output made fit for the preview.

use itertools::Itertools;

/// An input box starts within this many lines of the bottom.
const BOX_REACH: usize = 8;
/// The most lines between the two rules of an input box.
const BOX_LINES: usize = 3;
/// The shortest run of rule characters that counts as a rule.
const RULE_WIDTH: usize = 8;

/// Plain lines that end in the pane's last text: no control or format characters, no blank lines below the
/// text, and no input box with its footer at the bottom.
pub fn clean(output: &str) -> Vec<String> {
    let lines = output
        .lines()
        .map(|line| plain(line).trim_end().to_owned())
        .collect();
    trimmed(without_input_box(trimmed(lines)))
}

/// `line` without control or format characters.
pub fn plain(line: &str) -> String {
    line.chars().filter(|&c| shown(c)).collect()
}

/// Not a control character (Cc) or an invisible format character (Cf: bidi controls, zero widths).
fn shown(c: char) -> bool {
    !c.is_control()
        && !matches!(c,
            '\u{ad}' | '\u{61c}' | '\u{180e}' | '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{206f}' | '\u{feff}' | '\u{fff9}'..='\u{fffb}')
}

fn trimmed(lines: Vec<String>) -> Vec<String> {
    let used = lines
        .iter()
        .rposition(|l| !l.is_empty())
        .map_or(0, |i| i + 1);
    lines.into_iter().take(used).collect()
}

/// Cuts a short input framed by two horizontal rules near the bottom, and the footer below it, as
/// interactive programs such as Claude Code draw them; it tells nothing about what the program did.
fn without_input_box(lines: Vec<String>) -> Vec<String> {
    let reach = lines.len().saturating_sub(BOX_REACH);
    let cut = lines
        .iter()
        .enumerate()
        .skip(reach)
        .filter(|(_, line)| rule(line))
        .map(|(i, _)| i)
        .tuple_windows()
        .filter(|&(top, bottom)| (2..=BOX_LINES + 1).contains(&(bottom - top)))
        .map(|(top, _)| top)
        .last();
    match cut {
        Some(top) => lines.into_iter().take(top).collect(),
        None => lines,
    }
}

/// A line of box-drawing rule characters only, such as `────` or `╭────╮`.
fn rule(line: &str) -> bool {
    let line = line.trim();
    line.chars().count() >= RULE_WIDTH
        && line.chars().all(|c| {
            matches!(
                c,
                '─' | '━' | '═' | '╭' | '╮' | '╰' | '╯' | '┌' | '┐' | '└' | '┘'
            )
        })
}
