#!/usr/bin/env bash
# tmux plugin entry point (TPM compatible, or `run-shell /path/to/sidebar.tmux`).
CURRENT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CTS="$CURRENT_DIR/bin/cts"

opt() { local v; v=$(tmux show -gqv "$1"); echo "${v:-$2}"; }

tmux bind-key "$(opt @cts_toggle_key b)" run-shell -b "'$CTS' toggle"
tmux bind-key "$(opt @cts_popup_key B)" run-shell -b "'$CTS' popup"

# Fixed hook indexes keep re-sourcing idempotent and leave the user's own hooks alone.
idx=$(opt @cts_hook_index 77)
for h in after-new-window after-new-session session-window-changed client-session-changed; do
  tmux set-hook -g "$h[$idx]" "run-shell -b \"'$CTS' ensure '#{window_id}'\""
done
tmux set-hook -g "window-layout-changed[$idx]" "run-shell -b \"'$CTS' fix-width '#{window_id}'\""
tmux set-hook -g "client-resized[$idx]" "run-shell -b \"'$CTS' fix-width\""

[ "$(opt @cts_default on)" = "on" ] && [ -z "$(tmux show -gqv @cts_enabled)" ] && "$CTS" on
exit 0
