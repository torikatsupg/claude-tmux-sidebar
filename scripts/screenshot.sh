#!/usr/bin/env bash
# shellcheck disable=SC2088 # the fake prompts show a literal ~/
# Regenerates docs/screenshot.svg from a throwaway tmux server filled with fake sessions.
#   scripts/screenshot.sh [en|ja]
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LANG_OPT="${1:-en}"
OUT="$REPO/docs/screenshot${LANG_OPT/#en/}.svg"
OUT="${OUT/screenshotja/screenshot-ja}"
W=100 H=16 SIDEBAR_W=34
TMP=$(mktemp -d)
export CTS_STATE_DIR="$TMP/state"
mkdir -p "$CTS_STATE_DIR"
t() { tmux -L cts-demo -f /dev/null "$@"; }
trap 't kill-server 2>/dev/null; rm -rf "$TMP"' EXIT

body() { # fake content for a main pane
  printf "printf '%s\\\\n' %s; exec sleep 3600" "%s" "$(printf "'%s' " "$@")"
}

t new-session -d -s work -n api -x "$W" -y "$H" "$(body '~/src/api' '' '> Add retry to the HTTP client' '' '● Bash(go test ./client/...)' '  ⎿  ok  example.com/api/client 0.41s')"
t set -g default-size "${W}x${H}"
t set -g status off
t set -g pane-border-style fg=colour238
t set -g pane-active-border-style fg=colour238
t new-window -d -t work: -n web "$(body '~/src/web')"
t new-session -d -s personal -n blog "$(body '~/src/blog')"
t new-session -d -s scratch -n zsh "$(body '~/tmp')"

socket=$(t display -p '#{socket_path}')
now=$(date +%s)
state() { # pane state detail cwd age_seconds
  jq -n --arg pane "$1" --arg state "$2" --arg detail "$3" --arg cwd "$4" --arg socket "$socket" \
    --argjson since $((now - $5)) --argjson now "$now" \
    '{session_id:$pane, state:$state, detail:$detail, cwd:$cwd, pane:$pane, socket:$socket, pid:null, since:$since, updated_at:$now}' \
    >"$CTS_STATE_DIR/${1#%}.json"
}
state "$(t display -p -t work:api '#{pane_id}')" running Bash /src/api 2
state "$(t display -p -t work:web '#{pane_id}')" waiting Edit /src/web 64
state "$(t display -p -t personal:blog '#{pane_id}')" "done" "" /src/blog 300

t set -g @cts_lang "$LANG_OPT"
t set -g @cts_width "$SIDEBAR_W"
t set -g @cts_default on
t run-shell "$REPO/sidebar.tmux"
t select-window -t work:api
sleep 1.5

sidebar=$(t list-panes -t work:api -F '#{pane_id} #{@cts_sidebar}' | awk '$2 == "1" { print $1 }')
main=$(t list-panes -t work:api -F '#{pane_id} #{@cts_sidebar}' | awk '$2 != "1" { print $1 }')
t capture-pane -e -p -t "$sidebar" >"$TMP/sidebar.ansi"
t capture-pane -e -p -t "$main" >"$TMP/main.ansi"
printf '│\n%.0s' $(seq "$H") >"$TMP/border.ansi"
printf '\033[38;5;238m' | cat - "$TMP/border.ansi" >"$TMP/border2.ansi"
python3 "$REPO/scripts/ansi2svg.py" "$W" "$H" "0:0:$TMP/sidebar.ansi" "$SIDEBAR_W:0:$TMP/border2.ansi" \
  "$((SIDEBAR_W + 1)):0:$TMP/main.ansi" >"$OUT"
echo "wrote $OUT"
