# Recorded by scripts/demo.sh (run that, not this file: it builds the binary and cleans up afterwards).
Output docs/demo.gif

Set Shell bash
# Nightfox, the palette of docs/screenshot.svg
Set Theme { "name": "Nightfox", "background": "#192330", "foreground": "#cdcecf", "cursor": "#cdcecf", "selection": "#2b3b51", "black": "#393b44", "red": "#c94f6d", "green": "#81b29a", "yellow": "#dbc074", "blue": "#719cd6", "magenta": "#9d79d6", "cyan": "#63cdcf", "white": "#dfdfe0", "brightBlack": "#575860", "brightRed": "#d16983", "brightGreen": "#8ebaa4", "brightYellow": "#e0c989", "brightBlue": "#86abdc", "brightMagenta": "#baa1e2", "brightCyan": "#7ad5d6", "brightWhite": "#e4e4e5" }
Set FontFamily "JetBrains Mono,JetBrainsMono Nerd Font Mono,Menlo,DejaVu Sans Mono,monospace"
Set FontSize 16
Set Width 1160
Set Height 620
Set Padding 12
Set Framerate 12
Set PlaybackSpeed 1
Set CursorBlink false

Hide
Type "exec ./scripts/demo.sh scene"
Enter
Wait+Screen@10s /Claude/
Show
Sleep 16s
