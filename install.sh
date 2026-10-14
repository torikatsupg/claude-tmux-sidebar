#!/usr/bin/env bash
# Registers the Claude Code hooks (~/.claude/settings.json) and the tmux plugin (~/.tmux.conf).
# Idempotent; backs up settings.json before editing. `./install.sh --uninstall` reverts both.
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOOK="$REPO/bin/cts-hook"
SETTINGS="${CLAUDE_SETTINGS:-$HOME/.claude/settings.json}"
TMUX_CONF="${TMUX_CONF:-$HOME/.tmux.conf}"
TMUX_LINE="run-shell '$REPO/sidebar.tmux'"
EVENTS='["SessionStart","SessionEnd","UserPromptSubmit","PreToolUse","PostToolUse","PermissionRequest","Notification","Stop"]'

for cmd in jq python3 tmux; do
  command -v "$cmd" >/dev/null || { echo "missing dependency: $cmd" >&2; exit 1; }
done

# Removes our hook entries (matched by command path) and drops events left empty.
strip_filter='
  .hooks //= {} |
  .hooks |= with_entries(
    .value |= map(.hooks |= map(select(.command != $hook)) | select(.hooks | length > 0))
  ) |
  .hooks |= with_entries(select(.value | length > 0)) |
  if .hooks == {} then del(.hooks) else . end'
add_filter='
  reduce ($events[]) as $e (.;
    .hooks[$e] = ((.hooks[$e] // []) + [{matcher: "*", hooks: [{type: "command", command: $hook, timeout: 5}]}]))'

mkdir -p "$(dirname "$SETTINGS")"
[ -f "$SETTINGS" ] || echo '{}' >"$SETTINGS"
cp "$SETTINGS" "$SETTINGS.bak.$(date +%Y%m%d%H%M%S)"

if [ "${1:-}" = "--uninstall" ]; then
  jq --arg hook "$HOOK" "$strip_filter" "$SETTINGS" >"$SETTINGS.tmp" && mv "$SETTINGS.tmp" "$SETTINGS"
  if [ -f "$TMUX_CONF" ]; then
    grep -vF -e "$TMUX_LINE" -e "# Claude Code sidebar" "$TMUX_CONF" >"$TMUX_CONF.tmp" || true
    mv "$TMUX_CONF.tmp" "$TMUX_CONF"
  fi
  if [ -n "${TMUX:-}" ]; then
    "$REPO/bin/cts" off
    idx=$(tmux show -gqv @cts_hook_index); idx=${idx:-77}
    for h in after-new-window after-new-session session-window-changed client-session-changed window-layout-changed client-resized; do
      tmux set-hook -gu "$h[$idx]" 2>/dev/null || true
    done
  fi
  echo "uninstalled (tmux key bindings remain until the tmux server restarts)"
  exit 0
fi

jq --arg hook "$HOOK" --argjson events "$EVENTS" "$strip_filter | $add_filter" "$SETTINGS" >"$SETTINGS.tmp" &&
  mv "$SETTINGS.tmp" "$SETTINGS"
echo "Claude Code hooks registered in $SETTINGS"

touch "$TMUX_CONF"
if ! grep -qF "$TMUX_LINE" "$TMUX_CONF"; then
  printf '\n# Claude Code sidebar\n%s\n' "$TMUX_LINE" >>"$TMUX_CONF"
  echo "tmux plugin added to $TMUX_CONF"
fi
if [ -n "${TMUX:-}" ]; then
  tmux run-shell "$REPO/sidebar.tmux"
  echo "loaded into the running tmux server"
fi
echo "Restart running Claude Code sessions to pick up the hooks."
