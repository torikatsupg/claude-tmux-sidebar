#!/usr/bin/env bash
# tmux plugin entry point (TPM compatible, or `run-shell /path/to/sidebar.tmux`).
CURRENT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CTS="$CURRENT_DIR/bin/cts"

# display-popup, set -p and hook arrays need tmux 3.2+. Unparsable versions (e.g. "master") are let through.
ver=$(tmux -V | sed -E 's/^[^0-9]*([0-9]+)\.([0-9]+).*/\1 \2/')
read -r major minor <<<"$ver"
if [ "${major:-}" -eq "${major:-}" ] 2>/dev/null && { [ "$major" -lt 3 ] || { [ "$major" -eq 3 ] && [ "$minor" -lt 2 ]; }; }; then
  tmux display-message "claude-tmux-sidebar: requires tmux 3.2+ (found $(tmux -V))"
  exit 0
fi

opt() { local v; v=$(tmux show -gqv "$1"); echo "${v:-$2}"; }

# Space-separated, e.g. "b C-b" (note that C-b C-b is send-prefix by default).
for key in $(opt @cts_toggle_key b); do
  tmux bind-key "$key" run-shell -b "'$CTS' toggle"
done
tmux bind-key "$(opt @cts_popup_key B)" run-shell -b "'$CTS' popup '#{pane_id}'"
tmux bind-key "$(opt @cts_next_key C-n)" run-shell -b "'$CTS' next '#{pane_id}' '#{client_name}'"
tmux bind-key "$(opt @cts_prev_key C-p)" run-shell -b "'$CTS' prev '#{pane_id}' '#{client_name}'"

# Fixed hook indexes keep re-sourcing idempotent and leave the user's own hooks alone.
idx=$(opt @cts_hook_index 77)
for h in after-new-window after-new-session session-window-changed client-session-changed; do
  tmux set-hook -g "${h}[$idx]" "run-shell -b \"'$CTS' ensure '#{window_id}'\""
done
tmux set-hook -g "window-layout-changed[${idx}]" "run-shell -b \"'$CTS' fix-width '#{window_id}'\""
tmux set-hook -g "client-resized[${idx}]" "run-shell -b \"'$CTS' fix-width\""

[ "$(opt @cts_default off)" = "on" ] && [ -z "$(tmux show -gqv @cts_enabled)" ] && "$CTS" on
exit 0
