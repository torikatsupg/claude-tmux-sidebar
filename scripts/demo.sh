#!/usr/bin/env bash
# shellcheck disable=SC2088 # the fake prompts show a literal ~/
# Records docs/demo.gif: this checkout's sidebar in a throwaway tmux server, fed by simulated Claude Code
# hook events (the real `hook` command, fake session ids). Needs vhs (`brew install vhs`, which brings ttyd
# and ffmpeg), jq and cargo. Nothing touches your own tmux server or Claude state.
#   scripts/demo.sh           record docs/demo.gif
#   scripts/demo.sh scene     set up the tmux server and attach to it (docs/demo.tape runs this)
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SELF="$REPO/scripts/demo.sh"
BIN="$REPO/bin/claude-tmux-sidebar"
SOCKET_NAME=cts-gif
SIDEBAR_W=36
t() { tmux -L "$SOCKET_NAME" -f /dev/null "$@"; }

# --- record: build, run vhs, clean up -------------------------------------------------------------------
record() {
    command -v vhs >/dev/null || {
        echo "vhs not found: brew install vhs" >&2
        exit 1
    }
    "$REPO/scripts/fetch-binary.sh" --build # the demo shows this checkout's code
    CTS_DEMO_DIR=$(mktemp -d)
    export CTS_DEMO_DIR
    trap 't kill-server 2>/dev/null; rm -rf "$CTS_DEMO_DIR"' EXIT
    (cd "$REPO" && vhs docs/demo.tape)
    echo "wrote $REPO/docs/demo.gif"
}

# --- scene: the tmux server with fake Claude panes ------------------------------------------------------
pane_file() { echo "$CTS_DEMO_DIR/panes/$1"; }

say() { # pane-name line...
    printf '%s\n' "${@:2}" >>"$(pane_file "$1")"
}

pane_id() { t display -p -t "$1" '#{pane_id}'; }

hook() { # target event [tool] [notification_type]
    local pane cwd
    pane=$(pane_id "$1")
    cwd="/home/you/src/$(t display -p -t "$1" '#{window_name}')"
    jq -nc --arg e "$2" --arg tool "${3:-}" --arg kind "${4:-}" --arg id "demo-${pane#%}" --arg cwd "$cwd" \
        '{hook_event_name:$e, session_id:$id, tool_name:$tool, notification_type:$kind, cwd:$cwd}' |
        TMUX="$(t display -p '#{socket_path}'),0,0" TMUX_PANE="$pane" "$BIN" hook
}

backdate() { # target seconds: makes the current status look older, so the ages differ
    local file
    file="$CTS_STATE_DIR/demo-$(pane_id "$1" | tr -d %).json"
    jq --argjson d "$2" '.since -= $d' "$file" >"$file.tmp" && mv "$file.tmp" "$file"
}

window() { # session window lines...
    local file
    file=$(pane_file "$1-$2")
    printf '%s\n' "${@:3}" >"$file"
    if t has-session -t "=$1" 2>/dev/null; then
        t new-window -d -t "$1:" -n "$2" "'$SELF' pane '$file'"
    else
        t new-session -d -s "$1" -n "$2" -x "$COLS" -y "$LINES_" "'$SELF' pane '$file'"
    fi
}

scene() {
    : "${CTS_DEMO_DIR:?run scripts/demo.sh instead}"
    export CTS_STATE_DIR="$CTS_DEMO_DIR/state"
    mkdir -p "$CTS_STATE_DIR" "$CTS_DEMO_DIR/panes"
    t kill-server 2>/dev/null || true
    COLS=$(tput cols) LINES_=$(tput lines)

    window work api '~/src/api' '' '> Add retry with backoff to the HTTP client' '' \
        '● Read(client/http.go)' '● Edit(client/http.go)' '  ⎿  Added 18 lines'
    t set -g default-size "${COLS}x${LINES_}"
    t set -g base-index 1
    t set -g status-style bg=colour235,fg=colour244
    t set -g status-left '#[fg=colour252,bold] #S #[default]'
    t set -g status-left-length 20
    t set -g status-right ''
    t set -g window-status-current-style fg=colour252,bold
    t set -g pane-border-style fg=colour238
    t set -g pane-active-border-style fg=colour244
    t move-window -s work:0 -t work:1
    window work web '~/src/web' '' '> Upgrade the bundler and fix the build' '' \
        '● Edit(package.json)' '  ⎿  Updated 3 lines'
    window work docs '~/src/docs' '' '> Proofread the install guide' '' '● Edit(install.md)' '' \
        'All typos fixed.'
    window oss cli '~/src/cli' '' '> Add a --json flag to the list command' '' '● Read(src/list.rs)'
    window oss site '~/src/site'

    hook work:api SessionStart && hook work:api UserPromptSubmit && hook work:api PreToolUse Edit
    hook work:web SessionStart && hook work:web UserPromptSubmit && hook work:web PreToolUse Edit
    hook work:docs SessionStart && hook work:docs UserPromptSubmit && hook work:docs Stop
    hook oss:cli SessionStart && hook oss:cli UserPromptSubmit && hook oss:cli PreToolUse Read
    hook oss:site SessionStart
    backdate work:api 41 && backdate work:web 12 && backdate oss:cli 95
    backdate work:docs 260 && backdate oss:site 1500

    t set -g @cts_width "$SIDEBAR_W"
    t set -g @cts_default on
    t run-shell "$REPO/sidebar.tmux"
    t select-window -t work:api
    t set-hook -g client-attached "run-shell -b \"'$SELF' play\""
    exec tmux -L "$SOCKET_NAME" -f /dev/null attach -t work
}

# --- pane: a fake Claude Code screen that shows its file as it grows -------------------------------------
pane() {
    stty -echo -icanon 2>/dev/null || true
    printf '\033[?25l'
    exec tail -n +1 -f "$1"
}

# --- play: the timeline, started when the recorder attaches ----------------------------------------------
play() {
    export CTS_STATE_DIR="$CTS_DEMO_DIR/state"
    local sidebar
    sidebar=$(t list-panes -t work:api -F '#{pane_id} #{@cts_sidebar}' | awk '$2 == "1" { print $1 }')
    sleep 2
    say work-api '' '● Bash(go test ./client/...)'
    hook work:api PreToolUse Bash
    sleep 1.5
    say work-web '' '● Bash(npm run build)' '' '  Allow this command?' '    npm run build' '' \
        '  › 1. Yes' '    2. No'
    hook work:web PreToolUse Bash
    hook work:web PermissionRequest Bash # web now waits for you
    sleep 1.5
    say oss-cli '● Edit(src/list.rs)'
    hook oss:cli PreToolUse Edit
    sleep 0.8
    t select-pane -t "$sidebar" # prefix + b focuses the sidebar
    sleep 1.2
    t send-keys -t "$sidebar" j
    sleep 1.5
    t send-keys -t "$sidebar" Enter # jumps to work:web
    sleep 2.5
    printf '\033[8A\033[J' >>"$(pane_file work-web)" # the prompt goes away once answered
    say work-web '' '● Bash(npm run build)' '  ⎿  Running…'
    hook work:web PostToolUse Bash # approved: running again
    sleep 1
    say work-api '  ⎿  ok  example.com/api/client 0.38s'
    hook work:api Stop
}

case "${1:-}" in
"") record ;;
scene) scene ;;
pane) pane "$2" ;;
play) play ;;
*) echo "usage: scripts/demo.sh [scene]" >&2 && exit 2 ;;
esac
