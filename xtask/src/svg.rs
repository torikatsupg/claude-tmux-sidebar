//! `cargo xtask ansi2svg`: composes captured panes into one SVG terminal screenshot.

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write as _};

use crate::ansi::{self, Cell};

const FG: &str = "#cdcecf";
const BG: &str = "#192330";
/// Cell size in px.
const CELL_WIDTH: f64 = 8.4;
const CELL_HEIGHT: u32 = 18;
const PADDING: u32 = 14;
const BASELINE: u32 = 14;

type Grid = HashMap<(usize, usize), Cell>;

pub fn run(args: &[String]) -> Result<(), String> {
    let [width, height, specs @ ..] = args else {
        return Err("usage: cargo xtask ansi2svg W H X:Y:FILE...".to_owned());
    };
    let size = |s: &str| s.parse::<usize>().map_err(|e| format!("{s}: {e}"));
    let (width, height) = (size(width)?, size(height)?);
    let panes: Vec<Grid> = specs.iter().map(|s| pane(s)).collect::<Result<_, _>>()?;
    // Later panes are drawn over earlier ones.
    let grid: Grid = panes
        .into_iter()
        .flatten()
        .filter(|((x, y), _)| *x < width && *y < height)
        .collect();
    io::stdout()
        .write_all(svg(&grid, width, height).as_bytes())
        .map_err(|e| e.to_string())
}

/// `X:Y:FILE`: a capture placed at cell offset (X, Y).
fn pane(spec: &str) -> Result<Grid, String> {
    let parts: Vec<&str> = spec.splitn(3, ':').collect();
    let [x, y, path] = parts.as_slice() else {
        return Err(format!("{spec}: expected X:Y:FILE"));
    };
    let offset = |s: &str| s.parse::<usize>().map_err(|e| format!("{spec}: {e}"));
    let (ox, oy) = (offset(x)?, offset(y)?);
    let text = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    Ok(ansi::lines(text.trim_end_matches('\n'))
        .into_iter()
        .enumerate()
        .flat_map(|(y, cells)| {
            cells
                .into_iter()
                .map(move |(x, cell)| ((ox + x, oy + y), cell))
        })
        .collect())
}

fn svg(grid: &Grid, width: usize, height: usize) -> String {
    let cells = (0..height).flat_map(|y| (0..width).map(move |x| (x, y)));
    let body: String = cells
        .filter_map(|(x, y)| grid.get(&(x, y)).map(|c| cell(x, y, c)))
        .collect();
    let (w, h) = (
        to_f64(width) * CELL_WIDTH + f64::from(PADDING * 2),
        to_u32(height) * CELL_HEIGHT + PADDING * 2,
    );
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.0}\" height=\"{h}\" \
         font-family=\"Menlo, Monaco, 'DejaVu Sans Mono', monospace\" font-size=\"14\">\n\
         <rect width=\"100%\" height=\"100%\" rx=\"8\" fill=\"{BG}\"/>\n{body}</svg>\n"
    )
}

fn cell(x: usize, y: usize, cell: &Cell) -> String {
    let style = &cell.style;
    let (fg, bg) = if style.reverse {
        (
            style.bg.as_deref().unwrap_or(BG),
            Some(style.fg.as_deref().unwrap_or(FG)),
        )
    } else {
        (style.fg.as_deref().unwrap_or(FG), style.bg.as_deref())
    };
    let (px, py) = (
        f64::from(PADDING) + to_f64(x) * CELL_WIDTH,
        PADDING + to_u32(y) * CELL_HEIGHT,
    );
    let rect = bg.map(|bg| {
        let w = CELL_WIDTH * to_f64(cell.width) + 0.5;
        format!("<rect x=\"{px:.1}\" y=\"{py}\" width=\"{w:.1}\" height=\"{CELL_HEIGHT}\" fill=\"{bg}\"/>\n")
    });
    let weight = if style.bold {
        " font-weight=\"bold\""
    } else {
        ""
    };
    let text = (!cell.ch.is_whitespace()).then(|| {
        let ch = escape(cell.ch);
        format!(
            "<text x=\"{px:.1}\" y=\"{}\" fill=\"{fg}\"{weight}>{ch}</text>\n",
            py + BASELINE
        )
    });
    rect.into_iter().chain(text).collect()
}

fn escape(ch: char) -> String {
    match ch {
        '&' => "&amp;".to_owned(),
        '<' => "&lt;".to_owned(),
        '>' => "&gt;".to_owned(),
        '"' => "&quot;".to_owned(),
        '\'' => "&#x27;".to_owned(),
        c => c.to_string(),
    }
}

/// Screen sizes are tiny, so these conversions never lose anything.
fn to_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

fn to_f64(n: usize) -> f64 {
    f64::from(to_u32(n))
}
