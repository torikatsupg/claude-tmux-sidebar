# claude-tmux-sidebar

[English](README.md) | [日本語](README.ja.md)

Run several Claude Code sessions in parallel across tmux and you soon lose track of which one is waiting for your approval and which one has already finished. claude-tmux-sidebar keeps a sidebar on the left of every tmux window that lists all your tmux sessions and the [Claude Code](https://claude.com/claude-code) sessions running in them, with live status.

![Demo: one of several Claude Code sessions in the sidebar turns yellow (waiting for approval), and Enter in the sidebar jumps to it](docs/demo.gif)

- See at a glance which Claude is working, which one is waiting for your approval, and which one has finished.
- Jump to any session with `Enter` or a click, or straight to the next Claude that is done or waiting with `prefix + C-n`.
- tmux sessions without Claude are listed too, so the sidebar doubles as a session switcher.

## Quick start

1. With [TPM](https://github.com/tmux-plugins/tpm), add the plugin to `~/.tmux.conf` and press `prefix + I`:

   ```tmux
   set -g @plugin 'torikatsupg/claude-tmux-sidebar#v0.12.1'
   ```

2. Inside Claude Code, install the hooks, then restart running Claude Code sessions:

   ```
   /plugin install tmux-sidebar --marketplace torikatsupg/claude-tmux-sidebar
   ```

Press `prefix + b` to open the sidebar. For other ways to install, see [Install](#install).

## Status

![The sidebar showing sessions in each status](docs/screenshot.svg)

| Icon | Status | When |
|---|---|---|
| ⠹ (green) | running | Claude is working, or subagents or shells it started in the background still are |
| ◆ (yellow) | waiting for you | Claude asks for approval or asks you a question |
| ✓ (cyan) | done | The turn has ended and nothing is left running |
| ○ (gray) | idle | Just started, or 60 s without input |

Each session shows its directory and git branch, with the tool in use and how long it has been in that status on the right. The line below shows the last prompt you sent and how full the context window is (yellow from 50%, red from 80%). The window index (`1:api`) appears only when the tmux session has more than one window. The rows of the window you are in have a slightly lighter background; when the cursor is on one of them, the window index is drawn in the accent color instead.

## Usage

| Key | Action |
|---|---|
| `prefix + b` | Open the sidebar (in every window) and focus it; when it is already open, focus it; when it has the focus, close it |
| `prefix + B` | Show the list in a popup (jumps and closes on select) |
| `prefix + C-n` / `prefix + C-p` | Go to the next / previous Claude that is done or waiting for you |
| `j` / `k` / `g` / `G` | Move the selection |
| `n` / `p` | Select the next / previous done or waiting Claude |
| `Enter` / click | Go to that pane (or that window or tmux session, on a window or session row) |
| `q` | Close |

Sessions, windows and panes are listed in tmux's own order. The selection follows the pane you are focused on.

## Options

Set them in `~/.tmux.conf` **before** the plugin is loaded (before `run '~/.tmux/plugins/tpm/tpm'`).

```tmux
set -g @cts_width 34          # sidebar width
set -g @cts_toggle_key b      # several keys allowed, space-separated
set -g @cts_popup_key B
set -g @cts_next_key C-n
set -g @cts_prev_key C-p
set -g @cts_default off       # open the sidebar when tmux starts
set -g @cts_lang en           # en / ja
set -g @cts_help on           # show the key help at the bottom
set -g @cts_label auto        # rows: auto (sessions > windows > Claude) / dir (flat) / window (#I:#W)
set -g @cts_single_window inline  # with auto, a named window alone in its session: inline / lines
set -g @cts_all_windows off   # also list windows without Claude
set -g @cts_indent 1          # spaces per level of the tree: 0-4
set -g @cts_notify off        # notify when Claude waits for you or is done: off / tmux / os / all
set -g @cts_hook_index 77     # tmux hook array index (change only on conflicts)
set -g @cts_color_accent '#D97757'  # colors: see below
set -g @cts_color_active colour235  # background of your window's rows
set -g @cts_context_windows 'claude-opus-5-5=1000000'  # context window size per model
```

Avoid keys tmux already binds (e.g. `C-b` is `send-prefix`): the plugin's binding replaces them.

With `@cts_label window`, each row shows its tmux window as `index:name` instead of the directory and branch, and follows a renamed window on the next refresh. Sessions outside tmux keep the directory.

By default (`@cts_label auto`), the list is a tree of sessions, windows and Claude rows, each row one level under its parent and showing its directory and branch. In a session with several windows, each window is a heading: `index:name` for a window you named yourself, and just `index` for any other, since tmux names a window after what runs in it, which for Claude Code is its version number. A window counts as named when its `automatic-rename` is off, which `rename-window` and `new-window -n` do for you. Set `@cts_label dir` for the flat list of earlier versions, with each row showing its window number when the session has several.

A session with a single window needs no window heading: its rows go under the session, which reads `session/name` when you named the window. Set `@cts_single_window lines` to give a named window its own heading instead.

With `@cts_all_windows on`, every tmux window is listed in tmux's order. A window without Claude is a dim `index:name` row: `Enter` or a click goes there, and the cursor stops on it, but `n` / `p` and `prefix + C-n` / `prefix + C-p` skip it. A window with Claude in one of its panes shows only the Claude row. A window linked into several sessions (`link-window`) is listed once, under the last of those sessions in tmux's order, as a Claude in it already is. The sidebar reads the option when it opens.

Each color has its own option and takes one of these forms of a tmux color: a name (`black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, their `bright` variants such as `brightred`, or `default`), `colour0`–`colour255` (or `color0`–`color255`), or `#rrggbb`. An unset or unrecognized value keeps the default, and that includes tmux's other color names such as `coral` or `grey50`: write those as `#rrggbb`. The sidebar reads them when it opens, so reopen it after a change.

| Option | Colors | Default |
|---|---|---|
| `@cts_color_accent` | the title, and the window index on the cursor row of your window | `magenta` |
| `@cts_color_cursor` | the cursor row's background | `colour237` |
| `@cts_color_active` | the background of the rows of the window you are in | `colour235` |
| `@cts_color_muted` | secondary text: branch, prompt, help labels | `colour245` |
| `@cts_color_dim` | faint text: separator line, idle mark | `brightblack` |
| `@cts_color_waiting` | the waiting mark, also in `cts status --color` | `yellow` |
| `@cts_color_running` | the running mark, also in `cts status --color` | `green` |
| `@cts_color_done` | the done mark, also in `cts status --color` | `cyan` |
| `@cts_color_warning` | context window usage from 50% | `yellow` |
| `@cts_color_critical` | context window usage from 80% | `red` |

`@cts_context_windows` takes space-separated `model=tokens` pairs, where the model is the id Claude Code writes to the transcript (such as `claude-opus-5-5`). Without it, or for a model it does not list, the size follows [Claude Code's documented rules](https://code.claude.com/docs/en/model-config#extended-context): 1M for Fable, Mythos, Opus 4.7 and later, Sonnet 5 and later, and Haiku 5.5 and later, for a model ending in `[1m]`, or with more than 200k tokens in use, and 200k otherwise. List a model only when those rules miss, such as with `CLAUDE_CODE_DISABLE_1M_CONTEXT=1`. To find a model's size, run `claude -p --model <model> --output-format json "hi"` and read `modelUsage.<model id>.contextWindow`.

You can also let Claude Code report the size: its status line input has each session's real size, which goes before `@cts_context_windows` and the rules. This is optional and useful only when the rules miss. Add one line to your [status line](https://code.claude.com/docs/en/statusline) script, after it reads its input, and restart Claude:

```bash
input=$(cat)
echo "$input" | ~/.tmux/plugins/claude-tmux-sidebar/bin/cts statusline
```

The command prints nothing, so the status line stays as it is.

## Status line and notifications

With the sidebar closed or another window in front, you can still see what needs you. `cts status` prints how many Claude sessions are waiting for you, running and done, such as `◆2 ⠹1 ✓3`. A status no session is in is left out, and with none of them it prints nothing. Put it in `status-right`:

```tmux
set -g status-right '#(~/.tmux/plugins/claude-tmux-sidebar/bin/cts status) %H:%M'
set -g status-interval 5   # how often tmux reruns it, in seconds (default 15)
```

To color it, wrap it in a tmux style, or add `--color` to color each count with `@cts_color_waiting`, `@cts_color_running` and `@cts_color_done` (yellow, green and cyan by default). Each colored count ends with `#[fg=default]`. These options take the same colors as in the sidebar, and an unrecognized value keeps the default.

```tmux
set -g status-right '#[fg=yellow]#(~/.tmux/plugins/claude-tmux-sidebar/bin/cts status)#[default] %H:%M'
set -g status-right '#(~/.tmux/plugins/claude-tmux-sidebar/bin/cts status --color) %H:%M'
```

`@cts_notify` also tells you when a session starts waiting for you or is done:

| Value | Notification |
|---|---|
| `off` (default) | none |
| `tmux` | a message such as `◆ api:2 waiting for you` on the client you use (the tmux session, with the window index when it has several windows, and the status), and a bell in the session's window |
| `os` | a desktop notification, with `osascript` on macOS or `notify-send` on Linux; nothing when the command is missing |
| `all` | both |

There is no notification when the client you used last shows the session's window, unless that window is zoomed into another pane. A session that already had the status is not notified again. Claude Code sends its own prompt notification about six seconds after a prompt opens, so it can arrive after you answered; one that arrives within five seconds of the session's previous hook is not notified. Only sessions running in tmux are notified.

The hooks read the value from `~/.cache/claude-tmux-sidebar/notify`, which the plugin writes when tmux loads it, so that a hook never waits on tmux. After changing `@cts_notify`, reload the config with `tmux source ~/.tmux.conf`.

## Requirements

- tmux 3.2+
- Claude Code
- macOS or Linux (arm64 / x86_64), with `curl` to download the prebuilt binary — or a [Rust toolchain](https://rustup.rs) to build it
- `jq`, only for `install.sh`

On first run, the plugin downloads the binary for your platform from the matching GitHub release and checks its SHA-256, or builds it with `cargo` when no release binary fits. Release binaries carry a [build provenance attestation](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations), which you can check with:

```sh
gh attestation verify ~/.tmux/plugins/claude-tmux-sidebar/bin/claude-tmux-sidebar --repo torikatsupg/claude-tmux-sidebar
```

## Install

There are two parts: the **tmux plugin** draws the sidebar, and the **Claude Code hooks** report each session's status. The [Quick start](#quick-start) installs both; for other setups:

- **Follow `main` instead of a release**: `set -g @plugin 'torikatsupg/claude-tmux-sidebar'`. Pinning a tag is recommended: nothing changes until you change the tag, and release tags cannot be moved or deleted.
- **Pin the Claude Code plugin to a release**: `/plugin marketplace add torikatsupg/claude-tmux-sidebar#v0.12.1`, then `/plugin install tmux-sidebar@torikatsupg`. The same two commands without the tag also work on Claude Code versions without `--marketplace`.
- **Hooks without the Claude Code plugin**: `~/.tmux/plugins/claude-tmux-sidebar/install.sh --hooks-only` registers them in `~/.claude/settings.json` (backed up first as `settings.json.bak.<timestamp>`). Use this or the plugin, not both, or every hook runs twice.
- **Without TPM**:

  ```sh
  git clone --branch v0.12.1 https://github.com/torikatsupg/claude-tmux-sidebar ~/.tmux/plugins/claude-tmux-sidebar
  ~/.tmux/plugins/claude-tmux-sidebar/install.sh
  ```

  This registers the hooks and appends `run-shell '.../sidebar.tmux'` to `~/.tmux.conf` (and loads it right away when run inside tmux).

Restart running Claude Code sessions to pick up the hooks.

### Uninstall

- TPM: remove the `@plugin` line and press `prefix + alt + u`.
- Claude Code plugin: `/plugin uninstall tmux-sidebar@torikatsupg`.
- `install.sh`: run `./install.sh --uninstall`.

## Privacy

Everything stays on your machine. Each Claude session's status is kept in `~/.cache/claude-tmux-sidebar/<session_id>.json`, readable only by you and removed when the session ends. It holds the session ID, working directory, tool name, tmux pane, Claude's process ID, the path of the session's transcript and the first 200 characters of your last prompt. The sidebar reads the git branch and context usage from the end of that transcript and does not store them.

For debugging, start Claude with `CTS_DEBUG=1` to append raw hook payloads to `~/.cache/claude-tmux-sidebar/debug.log`. **The log contains your prompts and tool inputs** — delete it when done and check it before sharing.

## Limitations

- A turn interrupted with Esc, or by denying a prompt, stays "running" or "waiting" until it turns idle 60 seconds later.
- The sidebar is not added to a zoomed window (it would unzoom it); it is added on the next visit.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for how it works, the development setup and the release steps.

## License

[MIT](LICENSE)
