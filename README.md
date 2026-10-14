# claude-tmux-sidebar

[English](README.md) | [日本語](README.ja.md)

A tmux sidebar that lists every tmux session and the [Claude Code](https://claude.com/claude-code) sessions running in it, with live status.

![screenshot](docs/screenshot.svg)

- See at a glance which Claude is working, which one is waiting for your approval, and which one has finished.
- Jump to any session with `Enter` or a click.
- tmux sessions without Claude are listed too, so the sidebar doubles as a session switcher.

| Icon | Status | Claude Code hook |
|---|---|---|
| ⠹ (green) | running | `UserPromptSubmit` / `PreToolUse` / `PostToolUse` |
| ◆ (yellow) | waiting for you | `PermissionRequest` / `Notification(permission_prompt)` / `PreToolUse(AskUserQuestion)` |
| ✓ (cyan) | done | `Stop` |
| ○ (gray) | idle | `SessionStart`, 60 s without input (`Notification(idle_prompt)`) |

## Requirements

- tmux 3.2+
- python3, jq, bash
- Claude Code

## Install

There are two parts: the **tmux plugin** (draws the sidebar) and the **Claude Code hooks** (report each session's status).

### 1. tmux plugin

With [TPM](https://github.com/tmux-plugins/tpm), add to `~/.tmux.conf`, then press `prefix + I`:

```tmux
set -g @plugin 'torikatsupg/claude-tmux-sidebar'
```

To pin a release, use `set -g @plugin 'torikatsupg/claude-tmux-sidebar#v0.1.2'`.

### 2. Claude Code hooks

Install the `tmux-sidebar` plugin from inside Claude Code:

```
/plugin install tmux-sidebar --marketplace torikatsupg/claude-tmux-sidebar
```

On older Claude Code versions:

```
/plugin marketplace add torikatsupg/claude-tmux-sidebar
/plugin install tmux-sidebar@torikatsupg
```

Restart running Claude Code sessions to pick up the hooks.

If you prefer not to use the Claude Code plugin, register the hooks in `~/.claude/settings.json` instead (a backup `settings.json.bak.<timestamp>` is made first):

```sh
~/.tmux/plugins/claude-tmux-sidebar/install.sh --hooks-only
```

Use only one of the two, or every hook runs twice.

### Without TPM

```sh
git clone https://github.com/torikatsupg/claude-tmux-sidebar ~/.tmux/plugins/claude-tmux-sidebar
~/.tmux/plugins/claude-tmux-sidebar/install.sh
```

This registers the hooks and appends `run-shell '.../sidebar.tmux'` to `~/.tmux.conf` (and loads it right away when run inside tmux).

### Uninstall

- TPM: remove the `@plugin` line and press `prefix + alt + u`.
- Claude Code plugin: `/plugin uninstall tmux-sidebar@torikatsupg`.
- `install.sh`: run `./install.sh --uninstall`.

## Usage

| Key | Action |
|---|---|
| `prefix + b` | Toggle the sidebar (in every window) |
| `prefix + B` | Show the list in a popup (jumps and closes on select) |
| `prefix + C-n` / `prefix + C-p` | Go to the next / previous Claude that is done or waiting for you |
| `j` / `k` / `g` / `G` | Move the selection |
| `n` / `p` | Select the next / previous done or waiting Claude |
| `Enter` / click | Go to that pane (or that tmux session, on a session row) |
| `q` | Close |

Sessions, windows and panes are listed in tmux's own order. The selection follows the pane you are focused on.

## Options

Set them in `~/.tmux.conf` **before** the plugin is loaded (before `run '~/.tmux/plugins/tpm/tpm'`).

```tmux
set -g @cts_width 34          # sidebar width
set -g @cts_toggle_key b      # several keys allowed, e.g. 'b C-s'
set -g @cts_popup_key B
set -g @cts_next_key C-n
set -g @cts_prev_key C-p
set -g @cts_default off       # open the sidebar when tmux starts
set -g @cts_lang en           # en / ja
set -g @cts_hook_index 77     # tmux hook array index (change only on conflicts)
```

Avoid keys tmux already binds (e.g. `C-b` is `send-prefix`): the plugin's binding replaces them.

## How it works

- `bin/cts-hook` runs on each Claude Code hook and writes the status, `$TMUX_PANE` and Claude's PID to `~/.cache/claude-tmux-sidebar/<session_id>.json`. `SessionEnd` removes the file.
- `bin/cts` controls tmux: tmux hooks such as `after-new-window` add a full-height pane on the left of each window and restore its width after layout changes.
- `bin/cts-sidebar` draws the list with curses and cleans up state files whose pane or process is gone. It closes itself in a window where only the sidebar is left.

## Privacy

Everything stays on your machine. State files hold the session ID, working directory, tool name and tmux pane, and are readable only by you.

For debugging, start Claude with `CTS_DEBUG=1` to append raw hook payloads to `~/.cache/claude-tmux-sidebar/debug.log`. **The log contains your prompts and tool inputs** — delete it when done and check it before sharing.

## Limitations

- A turn interrupted with Esc does not fire `Stop`, so it stays "running" until `idle_prompt` turns it idle 60 seconds later.
- The sidebar is not added to a zoomed window (it would unzoom it); it is added on the next visit.

## Development

```sh
shellcheck bin/cts bin/cts-hook sidebar.tmux install.sh scripts/*.sh
scripts/screenshot.sh        # regenerate docs/screenshot.svg (scripts/screenshot.sh ja for the Japanese one)
```

To release, bump `version` in `.claude-plugin/plugin.json` and `.claude-plugin/marketplace.json`, add the section to `CHANGELOG.md`, merge to `main`, and run the **Release** workflow from the Actions tab. It tags `v<version>` and publishes a GitHub release; only the repository owner can run it.

## License

[MIT](LICENSE)
