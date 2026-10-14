//! SGR (Select Graphic Rendition) codes: how escape sequences change the text style.

/// Nightfox-like palette: 0-7 normal, 8-15 bright.
const PALETTE: [&str; 16] = [
    "#393b44", "#c94f6d", "#81b29a", "#dbc074", "#719cd6", "#9d79d6", "#63cdcf", "#dfdfe0",
    "#575860", "#d16983", "#8ebaa4", "#e0c989", "#86abdc", "#baa1e2", "#7ad5d6", "#e4e4e5",
];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub bold: bool,
    pub reverse: bool,
}

pub fn apply(style: Style, codes: &[u16]) -> Style {
    match codes {
        [] => style,
        [38, 5, n, rest @ ..] => apply(
            Style {
                fg: Some(color256(*n)),
                ..style
            },
            rest,
        ),
        [48, 5, n, rest @ ..] => apply(
            Style {
                bg: Some(color256(*n)),
                ..style
            },
            rest,
        ),
        [code, rest @ ..] => apply(basic(style, *code), rest),
    }
}

/// One single-number SGR code: an attribute or a basic colour.
fn basic(style: Style, code: u16) -> Style {
    match code {
        30..=37 => Style {
            fg: palette(code - 30),
            ..style
        },
        90..=97 => Style {
            fg: palette(code - 82),
            ..style
        },
        39 => Style { fg: None, ..style },
        40..=47 => Style {
            bg: palette(code - 40),
            ..style
        },
        49 => Style { bg: None, ..style },
        _ => attribute(style, code),
    }
}

fn attribute(style: Style, code: u16) -> Style {
    match code {
        0 => Style::default(),
        1 => Style {
            bold: true,
            ..style
        },
        7 => Style {
            reverse: true,
            ..style
        },
        22 => Style {
            bold: false,
            ..style
        },
        27 => Style {
            reverse: false,
            ..style
        },
        _ => style,
    }
}

fn palette(index: u16) -> Option<String> {
    PALETTE.get(usize::from(index)).map(|&c| c.to_owned())
}

/// The xterm 256-colour palette.
fn color256(n: u16) -> String {
    match n {
        0..=15 => palette(n).unwrap_or_default(),
        16..=231 => {
            let level = |v: u16| if v == 0 { 0 } else { 55 + v * 40 };
            let n = n - 16;
            format!(
                "#{:02x}{:02x}{:02x}",
                level(n / 36),
                level(n / 6 % 6),
                level(n % 6)
            )
        }
        _ => {
            let v = 8 + (n.min(255) - 232) * 10;
            format!("#{v:02x}{v:02x}{v:02x}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_the_256_colour_cube_and_greys() {
        assert_eq!(color256(196), "#ff0000");
        assert_eq!(color256(238), "#444444");
    }
}
