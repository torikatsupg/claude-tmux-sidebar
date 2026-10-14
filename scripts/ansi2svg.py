#!/usr/bin/env python3
"""Renders `tmux capture-pane -e` output of several panes into one SVG terminal screenshot.

usage: ansi2svg.py WIDTH HEIGHT X:Y:FILE... > out.svg   (X/Y: pane offset in cells)
"""

import re
import sys
import unicodedata
from html import escape

# Nightfox-like palette: 0-7 normal, 8-15 bright.
PALETTE = ["#393b44", "#c94f6d", "#81b29a", "#dbc074", "#719cd6", "#9d79d6", "#63cdcf", "#dfdfe0",
           "#575860", "#d16983", "#8ebaa4", "#e0c989", "#86abdc", "#baa1e2", "#7ad5d6", "#e4e4e5"]
FG, BG = "#cdcecf", "#192330"
CW, CH = 8.4, 18  # cell size in px


def color256(n):
    if n < 16:
        return PALETTE[n]
    if n < 232:
        n -= 16
        return "#%02x%02x%02x" % tuple(0 if v == 0 else 55 + v * 40 for v in (n // 36, n // 6 % 6, n % 6))
    v = 8 + (n - 232) * 10
    return "#%02x%02x%02x" % (v, v, v)


def parse(text, grid, ox, oy):
    for y, line in enumerate(text.split("\n")):
        x, st = 0, {"fg": None, "bg": None, "bold": False, "rev": False}
        for tok in re.split(r"(\x1b\[[0-9;]*m)", line):
            if tok.startswith("\x1b["):
                codes = [int(c) if c else 0 for c in tok[2:-1].split(";")]
                i = 0
                while i < len(codes):
                    c = codes[i]
                    if c == 0:
                        st = {"fg": None, "bg": None, "bold": False, "rev": False}
                    elif c == 1:
                        st["bold"] = True
                    elif c == 7:
                        st["rev"] = True
                    elif c == 22:
                        st["bold"] = False
                    elif c == 27:
                        st["rev"] = False
                    elif 30 <= c <= 37:
                        st["fg"] = PALETTE[c - 30]
                    elif 90 <= c <= 97:
                        st["fg"] = PALETTE[c - 82]
                    elif c == 39:
                        st["fg"] = None
                    elif 40 <= c <= 47:
                        st["bg"] = PALETTE[c - 40]
                    elif c == 49:
                        st["bg"] = None
                    elif c in (38, 48) and i + 2 < len(codes) and codes[i + 1] == 5:
                        st["fg" if c == 38 else "bg"] = color256(codes[i + 2])
                        i += 2
                    i += 1
                continue
            for ch in tok:
                w = 2 if unicodedata.east_asian_width(ch) in "WF" else 1
                if (oy + y) in grid and ox + x < len(grid[oy + y]):
                    grid[oy + y][ox + x] = (ch, dict(st), w)
                x += w


def main():
    width, height = int(sys.argv[1]), int(sys.argv[2])
    grid = {y: [(" ", {"fg": None, "bg": None, "bold": False, "rev": False}, 1)] * width for y in range(height)}
    for spec in sys.argv[3:]:
        x, y, path = spec.split(":", 2)
        with open(path) as f:
            parse(f.read().rstrip("\n"), grid, int(x), int(y))

    pad = 14
    out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{width * CW + pad * 2:.0f}" height="{height * CH + pad * 2}" '
           f'font-family="Menlo, Monaco, \'DejaVu Sans Mono\', monospace" font-size="14">',
           f'<rect width="100%" height="100%" rx="8" fill="{BG}"/>']
    for y in range(height):
        for x, (ch, st, w) in enumerate(grid[y]):
            fg, bg = st["fg"] or FG, st["bg"]
            if st["rev"]:
                fg, bg = bg or BG, fg
            px, py = pad + x * CW, pad + y * CH
            if bg:
                out.append(f'<rect x="{px:.1f}" y="{py}" width="{CW * w + 0.5:.1f}" height="{CH}" fill="{bg}"/>')
            if ch.strip():
                weight = ' font-weight="bold"' if st["bold"] else ""
                out.append(f'<text x="{px:.1f}" y="{py + 14}" fill="{fg}"{weight}>{escape(ch)}</text>')
    out.append("</svg>")
    print("\n".join(out))


if __name__ == "__main__":
    main()
