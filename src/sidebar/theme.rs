//! The sidebar's colors, one per role, read from tmux options in tmux's color syntax.

use ratatui::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub accent: Color,
    pub cursor: Color,
    pub active: Color,
    pub muted: Color,
    pub dim: Color,
    pub waiting: Color,
    pub running: Color,
    pub done: Color,
    pub warning: Color,
    pub critical: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Theme {
    pub const DEFAULT: Self = Self {
        accent: Color::Magenta,
        cursor: Color::Indexed(237),
        active: Color::Indexed(235),
        // Readable but quieter than the default text, on dark and light terminal themes.
        muted: Color::Indexed(245),
        dim: Color::DarkGray,
        waiting: Color::Yellow,
        running: Color::Green,
        done: Color::Cyan,
        warning: Color::Yellow,
        critical: Color::Red,
    };

    /// The `@cts_color_*` options looked up with `option`; an unset or unparsable one keeps its default.
    pub fn from_options(option: impl Fn(&str) -> Option<String>) -> Self {
        let pick = |role: &str, default: Color| {
            option(&format!("@cts_color_{role}"))
                .as_deref()
                .and_then(color)
                .unwrap_or(default)
        };
        let d = Self::DEFAULT;
        Self {
            accent: pick("accent", d.accent),
            cursor: pick("cursor", d.cursor),
            active: pick("active", d.active),
            muted: pick("muted", d.muted),
            dim: pick("dim", d.dim),
            waiting: pick("waiting", d.waiting),
            running: pick("running", d.running),
            done: pick("done", d.done),
            warning: pick("warning", d.warning),
            critical: pick("critical", d.critical),
        }
    }
}

/// A tmux color: a name, `brightNAME`, `colourN` / `colorN` or `#rrggbb`, in any case.
pub fn color(value: &str) -> Option<Color> {
    let value = value.trim().to_ascii_lowercase();
    if let Some(hex) = value.strip_prefix('#') {
        return rgb(hex);
    }
    if let Some(n) = value
        .strip_prefix("colour")
        .or_else(|| value.strip_prefix("color"))
    {
        return index(n);
    }
    named(&value)
}

fn rgb(hex: &str) -> Option<Color> {
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let [_, r, g, b] = u32::from_str_radix(hex, 16).ok()?.to_be_bytes();
    Some(Color::Rgb(r, g, b))
}

fn index(n: &str) -> Option<Color> {
    // u8::from_str also takes a leading `+`, which tmux does not.
    let digits = !n.is_empty() && n.chars().all(|c| c.is_ascii_digit());
    digits.then(|| n.parse().ok().map(Color::Indexed)).flatten()
}

/// ratatui's `Gray` is ANSI white and `White` is bright white, so tmux's `white` maps to `Gray`.
fn named(name: &str) -> Option<Color> {
    let color = match name {
        "default" => Color::Reset,
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::Gray,
        "brightblack" => Color::DarkGray,
        "brightred" => Color::LightRed,
        "brightgreen" => Color::LightGreen,
        "brightyellow" => Color::LightYellow,
        "brightblue" => Color::LightBlue,
        "brightmagenta" => Color::LightMagenta,
        "brightcyan" => Color::LightCyan,
        "brightwhite" => Color::White,
        _ => return None,
    };
    Some(color)
}

#[cfg(test)]
mod tests;
